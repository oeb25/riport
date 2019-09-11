use actix::*;

use serde::{Deserialize, Serialize};

use futures::future::{join_all, Future};

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::client::Client;
use crate::file::{self, File, FileId, FileInfo};

use crate::s2c::*;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProjectId {
    pub project_id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectConfig {
    pub id: ProjectId,
    pub name: String,
    pub order: Vec<PathBuf>,
}

pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub path: PathBuf,
    pub last_changed: SystemTime,
    pub order: Vec<FileId>,
    pub files: HashMap<FileId, Addr<File>>,
    pub listeners: Vec<Option<WeakAddr<Client>>>,
    pub tmpdir: PathBuf,
}

impl Project {
    pub fn empty(id: ProjectId, name: String, path: PathBuf, tmpdir: PathBuf) -> Project {
        Project {
            id,
            name,
            path,
            last_changed: SystemTime::now(),
            order: vec![],
            files: HashMap::new(),
            listeners: vec![],
            tmpdir,
        }
    }
    pub fn new(id: ProjectId, name: String, path: PathBuf, tmpdir: PathBuf) -> Addr<Project> {
        Project::create(move |ctx| {
            let mut project = Project::empty(id, name, path, tmpdir);
            project.new_file("index.md".to_string(), "# Index".to_string(), ctx);
            project.new_file("abstract.md".to_string(), "# Abstract".to_string(), ctx);
            project.new_file("conlusion.md".to_string(), "# Conlusion".to_string(), ctx);

            ctx.wait(
                project
                    .write_to_disk(project.path.clone())
                    .into_actor(&project)
                    .map(|_, _, _| ())
                    .map_err(|_, _, _| ()),
            );

            project
        })
    }
    pub fn new_file(
        &mut self,
        name: String,
        src: String,
        ctx: &mut Context<Self>,
    ) -> (FileId, Addr<File>) {
        let id = FileId {
            file_id: self.files.len() as _,
        };
        let file_tmpdir = self.tmpdir.join(&format!("{}", id.file_id));
        fs::create_dir_all(&file_tmpdir).unwrap();
        let file = File::new(
            id,
            self.id,
            ctx.address().downgrade(),
            name,
            src,
            file_tmpdir,
        );
        let file_addr = file.start();
        self.files.insert(id, file_addr.clone());
        self.order.push(id);

        (id, file_addr)
    }
    pub fn generate_file_info(&self) -> impl Future<Item = Vec<FileInfo>> {
        let mut files = vec![];

        for file in self.files.values() {
            let res = file.send(file::GetInfo);
            files.push(res);
        }

        join_all(files)
    }
    pub fn generate_info(&self) -> ProjectInfo {
        ProjectInfo {
            name: self.name.clone(),
            id: self.id,
            last_changed: self.last_changed,
            files: self.order.clone(),
        }
    }
    pub fn generate_config(&self) -> impl Future<Item = ProjectConfig> {
        let name = self.name.clone();
        let id = self.id;
        self.generate_file_info().map(move |infos| ProjectConfig {
            name,
            id,
            order: infos
                .into_iter()
                .map(|info| PathBuf::from(info.name))
                .collect(),
        })
    }
    pub fn write_to_disk(&self, dir: PathBuf) -> impl Future<Item = io::Result<()>> {
        let name = self.name.clone();
        let id = self.id;

        fs::create_dir_all(&dir).unwrap();

        let dir2 = dir.clone();

        let writes = self
            .order
            .iter()
            .map(|id| self.files[id].send(file::WriteToDisk { dir: dir2.clone() }))
            .collect::<Vec<_>>();

        Future::join(
            join_all(writes).map_err(|_| ()),
            self.generate_config().map_err(|_| ()),
        )
        .map(move |(writes, config)| {
            let mut order = vec![];
            for w in writes {
                match w {
                    Ok(p) => {
                        order.push(p.strip_prefix(&dir).unwrap().to_owned());
                    }
                    Err(e) => return Err(e),
                }
            }

            let config = ProjectConfig { name, id, order };

            fs::write(
                dir.join("config.json"),
                serde_json::to_string(&config).unwrap(),
            )?;

            Ok(())
        })
    }
    pub fn read_from_disk(dir: PathBuf, tmpdir: PathBuf) -> Addr<Project> {
        pub fn helper(
            dir: PathBuf,
            ctx: &mut Context<Project>,
            tmpdir: PathBuf,
        ) -> io::Result<Project> {
            let config = fs::read_to_string(dir.join("config.json"))?;
            let config: ProjectConfig =
                serde_json::from_str(&config).expect("failed to parse project config");

            let mut project = Project::empty(config.id, config.name, dir, tmpdir);

            for path in config.order {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                let path = project.path.join(path);
                if path.is_dir() {
                    // TODO: Sub directories
                } else {
                    let src = fs::read_to_string(&path)?;
                    let (id, _) = project.new_file(name.to_string(), src, ctx);
                }
            }

            let pdf_path_dir = PathBuf::from("./");
            let pdf_path_dir = pdf_path_dir.canonicalize().unwrap();
            let pdf_path = pdf_path_dir.join(format!("./project-{}.pdf", project.id.project_id));

            ctx.wait(
                project
                    .create_pdf(pdf_path)
                    .into_actor(&project)
                    .map(|_, _, _| ())
                    .map_err(|_, _, _| ()),
            );

            Ok(project)
        }

        Project::create(move |ctx| helper(dir, ctx, tmpdir).unwrap())
    }
    fn create_pdf(&self, pdf_path: PathBuf) -> impl Future<Item = PathBuf> {
        let root = self.tmpdir.clone();

        join_all(
            self.order
                .iter()
                .cloned()
                .map(|file_id| {
                    self.files[&file_id]
                        .send(file::GetDoc)
                        .map(move |x| (file_id, x))
                })
                .collect::<Vec<_>>(),
        )
        .map(move |res| {
            let meta = pandoc_types::definition::Meta(HashMap::new());
            let doc = pandoc_types::definition::Pandoc(
                meta,
                res.into_iter().flat_map(|(_, xs)| xs.doc).collect(),
            );
            crate::doc::to_pdf(&doc, &root, &pdf_path).expect("failed to create pdf");
            pdf_path
        })
    }
}

impl Actor for Project {
    type Context = Context<Self>;
}

pub struct GetInfo;
impl Message for GetInfo {
    type Result = ProjectInfo;
}

impl Handler<GetInfo> for Project {
    type Result = ProjectInfo;

    fn handle(&mut self, _: GetInfo, _: &mut Self::Context) -> ProjectInfo {
        self.generate_info()
    }
}

#[derive(Serialize, Clone, MessageResponse)]
pub struct ProjectInfo {
    name: String,
    last_changed: SystemTime,
    id: ProjectId,
    files: Vec<FileId>,
}

pub struct JoinProject {
    pub addr: Addr<Client>,
}

impl Message for JoinProject {
    type Result = usize;
}

impl Handler<JoinProject> for Project {
    type Result = usize;

    fn handle(&mut self, join: JoinProject, ctx: &mut Self::Context) -> usize {
        let id = self.listeners.len();
        self.listeners.push(Some(join.addr.downgrade()));

        let project_id = self.id;

        self.generate_file_info()
            .into_actor(self)
            .then(move |res, _, _| match res {
                Ok(list) => {
                    join.addr.do_send(Server2Client::Project {
                        id: project_id,
                        msg: Server2ClientProject::Files { list },
                    });
                    fut::ok(())
                }
                _ => fut::ok(()),
            })
            .wait(ctx);

        id
    }
}

pub struct LeaveProject {
    pub id: usize,
}

impl Message for LeaveProject {
    type Result = ();
}

impl Handler<LeaveProject> for Project {
    type Result = ();

    fn handle(&mut self, leave: LeaveProject, _: &mut Self::Context) {
        self.listeners[leave.id] = None;
    }
}

pub struct GetFile {
    pub id: FileId,
}
impl Message for GetFile {
    type Result = Option<Addr<File>>;
}

impl Handler<GetFile> for Project {
    type Result = Option<Addr<File>>;

    fn handle(&mut self, get_file: GetFile, _: &mut Self::Context) -> Option<Addr<File>> {
        self.files.get(&get_file.id).cloned()
    }
}

pub struct FileChanged {
    pub id: FileId,
}
impl Message for FileChanged {
    type Result = ();
}

impl Handler<FileChanged> for Project {
    type Result = ();

    fn handle(&mut self, _: FileChanged, ctx: &mut Self::Context) {
        self.last_changed = SystemTime::now();

        let info = self.generate_info();

        for l in self
            .listeners
            .iter()
            .filter_map(|x| x.as_ref())
            .filter_map(|f| f.upgrade())
        {
            l.do_send(Server2Client::Project {
                id: self.id,
                msg: Server2ClientProject::UpdateInfo { info: info.clone() },
            });
        }

        ctx.wait(
            self.write_to_disk(PathBuf::from("./tmp").join(&self.name))
                .into_actor(self)
                .map(|_, _, _| ())
                .map_err(|_, _, _| ()),
        );

        // self.write_to_disk(PathBuf::from("./tmp/").join(&self.name)).into_actor()
    }
}

pub struct ReorderFile {
    pub id: FileId,
    pub new_index: usize,
}
impl Message for ReorderFile {
    type Result = ();
}

impl Handler<ReorderFile> for Project {
    type Result = ();

    fn handle(&mut self, reorder: ReorderFile, _: &mut Self::Context) {
        let old_index = self
            .order
            .iter()
            .position(|f| *f == reorder.id)
            .expect("file was in the order");
        if old_index == reorder.new_index {
            return;
        }

        if old_index > reorder.new_index {
            self.order.remove(old_index);
            self.order.insert(reorder.new_index, reorder.id);
        } else {
            self.order.insert(reorder.new_index, reorder.id);
            self.order.remove(old_index);
        }

        self.last_changed = SystemTime::now();

        let info = self.generate_info();

        for l in self
            .listeners
            .iter()
            .filter_map(|x| x.as_ref())
            .filter_map(|f| f.upgrade())
        {
            l.do_send(Server2Client::Project {
                id: self.id,
                msg: Server2ClientProject::UpdateInfo { info: info.clone() },
            });
        }
    }
}
