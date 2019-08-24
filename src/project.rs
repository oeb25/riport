use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone, Copy)]
pub struct EditorId {
    pub editor_id: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Lock {
    pub by: EditorId,
    pub locked_at: time::SystemTime,
    pub duration: time::Duration,
}

impl Lock {
    pub fn new(now: time::SystemTime, by: EditorId) -> Lock {
        Lock {
            by,
            locked_at: now,
            duration: time::Duration::from_secs(1),
        }
    }
    pub fn is_expired(&self) -> bool {
        let now = time::SystemTime::now();
        (self.locked_at + self.duration) < now
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Change {
    by: EditorId,
    time: time::SystemTime,
}

impl Change {
    fn new(by: EditorId) -> Change {
        Change {
            by,
            time: time::SystemTime::now(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
pub struct FileId {
    path: PathBuf,
}

#[derive(Debug, Serialize, Clone)]
pub enum Compile {
    None,
    Stale(String),
    UpToDate(String),
}

impl Compile {
    fn needs_compile(&self) -> bool {
        match self {
            Compile::None | Compile::Stale(_) => true,
            Compile::UpToDate(s) => false,
        }
    }
    fn stale(&self) -> Compile {
        match self {
            Compile::None | Compile::Stale(_) => self.clone(),
            Compile::UpToDate(s) => Compile::Stale(s.clone()),
        }
    }
    fn get_str(&self) -> Option<&str> {
        match self {
            Compile::None => None,
            Compile::Stale(s) | Compile::UpToDate(s) => Some(s),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct File {
    lock: Option<Lock>,
    last_source_change: Option<Change>,
    last_compiled_time: Option<time::SystemTime>,
    source: String,
    compiled: Compile,
}

#[derive(Debug)]
pub enum EditError {
    Locked(Lock),
    FileNotFound,
    FileNotCompiled,
    IO(io::Error),
}

impl EditError {
    pub fn locked(&self) -> Option<&Lock> {
        match self {
            EditError::Locked(lock) => Some(lock),
            _ => None,
        }
    }
}

impl std::error::Error for EditError {}
impl std::fmt::Display for EditError {
    fn fmt(&self, writer: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(writer, "{:?}", self)
    }
}
// impl Into<EditError> for io::Error {
//     fn into(self) -> EditError {
//         EditError::IO(self)
//     }
// }
impl From<io::Error> for EditError {
    fn from(x: io::Error) -> EditError {
        EditError::IO(x)
    }
}

const SAMPLE_FILE: &str = r#"# Live preview

All input is sent to the server, compiled, and then sent back to you and displayed.

# Realtime collaboration

Working together with multiple people on the same project, has become a requirement for almost any situation.

# Do Math

Either inline $-1 = e^{i\pi}$, or block

$$
\int_a^b 1 = b - a
$$

# Execute code

```python
def fac(a):
    if a < 2:
        return a
    else:
        return a * fac(a - 1)

print(fac(12))
```

# Draw graphs

```graphviz
graph G {
    Browser -- Server
    Server -- Project
    Project -- Filter
    Pandoc -- Filter
    Project -- Pandoc
}
```

"#;

impl File {
    fn new() -> File {
        File {
            lock: None,
            last_source_change: None,
            last_compiled_time: None,
            source: SAMPLE_FILE.to_string(),
            compiled: Compile::None,
        }
    }
    fn empty() -> File {
        File {
            lock: None,
            last_source_change: None,
            last_compiled_time: None,
            source: String::new(),
            compiled: Compile::None,
        }
    }
    fn can_edit(&self, editor: EditorId) -> bool {
        if let Some(lock) = self.lock.as_ref() {
            lock.by == editor || lock.is_expired()
        } else {
            true
        }
    }
    fn lock(&mut self, now: time::SystemTime, editor: EditorId) -> Result<(), EditError> {
        if self.can_edit(editor) {
            self.lock = Some(Lock::new(now, editor));
            Ok(())
        } else {
            Err(EditError::Locked(self.lock.clone().unwrap()))
        }
    }
    fn edit(
        &mut self,
        now: time::SystemTime,
        editor: EditorId,
        source: &str,
    ) -> Result<(), EditError> {
        self.lock(now, editor)?;
        self.source = source.to_owned();
        self.last_source_change = Some(Change::new(editor));
        self.compiled = self.compiled.stale();
        Ok(())
    }
    fn compile(&mut self, root: &Path) -> io::Result<()> {
        if !self.compiled.needs_compile() {
            return Ok(());
        }

        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut cmd = Command::new("pandoc")
            .args(&["-f", "markdown", "-t", "json"])
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        write!(
            cmd.stdin.as_mut().expect("failed to get stdin"),
            "{}",
            &self.source
        )?;
        let out = cmd.wait_with_output()?;
        let compiled = String::from_utf8_lossy(&out.stdout).to_string();

        let parsed: pandoc_types::definition::Pandoc =
            serde_json::from_str(&compiled).expect("failed to parse pandoc output");

        use pandoc_types::definition::{Attr, Block, Inline, Target};

        struct RunPython<'a> {
            run_dir: &'a Path,
        }
        impl<'a> crate::walk_pandoc::Walk for RunPython<'a> {
            fn block(&mut self, block: Block) -> Vec<Block> {
                match block {
                    Block::CodeBlock(attr, src) => {
                        let lang = &attr.1[0];
                        if lang == "python" {
                            let mut cmd = Command::new("python")
                                .current_dir(self.run_dir)
                                .stdin(Stdio::piped())
                                .stdout(Stdio::piped())
                                .spawn()
                                .unwrap();
                            write!(cmd.stdin.as_mut().expect("failed to get stdin"), "{}", &src)
                                .unwrap();
                            let out = cmd.wait_with_output().unwrap();
                            let compiled = String::from_utf8_lossy(&out.stdout).to_string();

                            vec![
                                Block::CodeBlock(attr.clone(), src),
                                Block::CodeBlock(
                                    Attr("".to_string(), vec!["".to_string()], vec![]),
                                    compiled,
                                ),
                            ]
                        } else if lang == "graphviz" {
                            let output_name = PathBuf::from("graph.png");
                            let output_path = self.run_dir.join(&output_name);

                            let mut cmd = Command::new("dot")
                                .args(&[
                                    "-Tpng",
                                    &format!("-o{}", output_path.to_string_lossy().to_string()),
                                ])
                                .current_dir(self.run_dir)
                                .stdin(Stdio::piped())
                                .spawn()
                                .unwrap();
                            write!(cmd.stdin.as_mut().expect("failed to get stdin"), "{}", &src)
                                .unwrap();
                            let _ = cmd.wait_with_output().unwrap();

                            vec![Block::Para(vec![Inline::Image(
                                Attr::null(),
                                vec![],
                                Target(
                                    output_name.to_string_lossy().to_string(),
                                    output_name.to_string_lossy().to_string(),
                                ),
                            )])]
                        } else {
                            vec![Block::CodeBlock(attr, src)]
                        }
                    }
                    _ => vec![block],
                }
            }
        }

        let transformed = crate::walk_pandoc::walk_pandoc(&mut RunPython { run_dir: root }, parsed);

        let compiled = serde_json::to_string(&transformed).unwrap();

        self.compiled = Compile::UpToDate(compiled);
        self.last_compiled_time = Some(time::SystemTime::now());
        Ok(())
    }
    fn index(&self) -> FileIndex {
        FileIndex {
            last_source_change: self.last_source_change.clone(),
            last_compiled_time: self.last_compiled_time.clone(),
            lock: self.lock.clone(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ProjectId {
    pub project_id: usize,
}

#[derive(Debug)]
pub struct Project {
    id: ProjectId,
    root: PathBuf,
    tempdir: tempdir::TempDir,
    order: Vec<FileId>,
    files: HashMap<FileId, File>,
}

impl Project {
    pub fn init(id: ProjectId, root: PathBuf) -> io::Result<Project> {
        fs::create_dir_all(&root)?;

        Ok(Project {
            id,
            root,
            tempdir: tempdir::TempDir::new("riport")?,
            order: vec![],
            files: HashMap::new(),
        })
    }
    pub fn idk(&mut self) {}
    pub fn new_file(&mut self, name: &str) -> FileId {
        let id = FileId {
            path: PathBuf::from(name),
        };
        self.order.push(id.clone());
        self.files.insert(id.clone(), File::new());
        self.compile()
            .expect("failed to compile newly created file");
        id
    }
    pub fn edit_file(
        &mut self,
        editor: EditorId,
        file: &FileId,
        src: &str,
    ) -> Result<time::SystemTime, EditError> {
        let now = time::SystemTime::now();
        self.files
            .get_mut(file)
            .ok_or(EditError::FileNotFound)?
            .edit(now, editor, src)?;
        self.compile()?;
        Ok(now)
    }
    pub fn get_source(&self, file: &FileId) -> Result<&str, EditError> {
        Ok(&self.files.get(file).ok_or(EditError::FileNotFound)?.source)
    }
    pub fn get_compiled(&self, file: &FileId) -> Result<&str, EditError> {
        self.files
            .get(file)
            .ok_or(EditError::FileNotFound)?
            .compiled
            .get_str()
            .ok_or(EditError::FileNotCompiled)
    }
    pub fn list_files(&self) -> Vec<FileId> {
        self.files.keys().cloned().collect()
    }
    pub fn compile(&mut self) -> io::Result<()> {
        for file in self.files.values_mut() {
            file.compile(self.tempdir.path())?;
        }
        Ok(())
    }
    pub fn static_file_path(&self, path: &Path) -> PathBuf {
        self.tempdir.path().join(path)
    }
    pub fn index(&self) -> ProjectIndex {
        ProjectIndex {
            order: self.order.clone(),
            files: self
                .files
                .iter()
                .map(|(id, file)| (id.path.clone(), file.index()))
                .collect(),
        }
    }
    pub fn index_delta(&self, editor: EditorId, index: ProjectIndex) -> ProjectIndexDelta {
        let mut a = index;
        let b = self.index();

        let mut new_files = vec![];
        let mut removed_files = vec![];
        let mut changed_source_files = vec![];
        let mut changed_compiled_files = vec![];

        for (path, new) in b.files {
            let id = FileId { path };

            let delta_item = |id: &FileId, index: &FileIndex| ProjectIndexDeltaItem {
                id: id.clone(),
                index: index.clone(),
            };
            if let Some(old) = a.files.remove(&id.path) {
                match (&old.last_source_change, &new.last_source_change) {
                    (None, Some(change)) if change.by != editor => {
                        changed_source_files.push(delta_item(&id, &new));
                    }
                    (Some(a), Some(b)) if b.by != editor && a.time < b.time => {
                        changed_source_files.push(delta_item(&id, &new));
                    }
                    _ => {}
                }
                match (&old.last_compiled_time, &new.last_compiled_time) {
                    (None, Some(change)) => {
                        changed_compiled_files.push(delta_item(&id, &new));
                    }
                    (Some(a), Some(b)) if a < b => {
                        changed_compiled_files.push(delta_item(&id, &new));
                    }
                    _ => {}
                }
            } else {
                new_files.push(delta_item(&id, &new));
            }
        }

        // TODO: Test
        for (path, _) in a.files {
            let id = FileId { path };

            removed_files.push(id)
        }

        ProjectIndexDelta {
            new_files,
            removed_files,
            changed_source_files,
            changed_compiled_files,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileIndex {
    lock: Option<Lock>,
    last_source_change: Option<Change>,
    last_compiled_time: Option<time::SystemTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectIndex {
    order: Vec<FileId>,
    files: HashMap<PathBuf, FileIndex>,
}

#[derive(Debug, Serialize)]
pub struct ProjectIndexDeltaItem {
    id: FileId,
    index: FileIndex,
}

#[derive(Debug, Serialize)]
pub struct ProjectIndexDelta {
    new_files: Vec<ProjectIndexDeltaItem>,
    removed_files: Vec<FileId>,
    changed_source_files: Vec<ProjectIndexDeltaItem>,
    changed_compiled_files: Vec<ProjectIndexDeltaItem>,
}

#[test]
fn run_python_code() {
    let editor = EditorId { editor_id: 0 };
    let mut project = Project::init(ProjectId { project_id: 0 }, PathBuf::from("./test")).unwrap();
    let file = project.new_file("test.md");
    project
        .edit_file(
            editor,
            &file,
            r#"
```python
print(123)
```
"#,
        )
        .unwrap();
    let compiled = project.get_compiled(&file).unwrap();
    assert_eq!(compiled, "");
}
