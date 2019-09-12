pub mod file;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::client::Client;
use crate::project::file::{File, FileId, FileInfo};

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
    pub order: Vec<FileId>,
    pub files: HashMap<FileId, File>,
    pub tmpdir: PathBuf,
}

impl Project {
    pub fn empty(id: ProjectId, name: String, path: PathBuf, tmpdir: PathBuf) -> Project {
        Project {
            id,
            name,
            path,
            order: vec![],
            files: HashMap::new(),
            tmpdir,
        }
    }
    pub fn new_file(&mut self, name: String, src: String) -> FileId {
        let id = FileId {
            file_id: self.files.len() as _,
        };
        let file_tmpdir = self.tmpdir.join(&format!("{}", id.file_id));
        fs::create_dir_all(&file_tmpdir).unwrap();
        let file = File::new(id, self.id, name, src, file_tmpdir);
        self.files.insert(id, file);
        self.order.push(id);

        id
    }
    pub fn generate_file_info(&self) -> Vec<FileInfo> {
        self.files.values().map(|f| f.get_info()).collect()
    }
    pub fn generate_info(&self) -> ProjectInfo {
        ProjectInfo {
            name: self.name.clone(),
            id: self.id,
            files: self.order.clone(),
        }
    }
    pub fn generate_config(&self) -> ProjectConfig {
        let name = self.name.clone();
        let id = self.id;
        let infos = self.generate_file_info();
        ProjectConfig {
            name,
            id,
            order: infos
                .into_iter()
                .map(|info: FileInfo| PathBuf::from(info.name))
                .collect(),
        }
    }
    pub fn write_to_disk(&self, dir: PathBuf) -> io::Result<()> {
        let name = self.name.clone();
        let id = self.id;

        fs::create_dir_all(&dir).unwrap();

        let mut order = vec![];
        for id in &self.order {
            let file = &self.files[id];
            file.write_to_disk(&dir)?;
            // TODO
            // order.push(file.path.strip_prefix(&dir).unwrap().to_owned());
        }

        let config = ProjectConfig { name, id, order };

        fs::write(
            dir.join("config.json"),
            serde_json::to_string(&config).unwrap(),
        )?;

        Ok(())
    }
    pub fn read_from_disk(dir: PathBuf, tmpdir: PathBuf) -> io::Result<Project> {
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
                let id = project.new_file(name.to_string(), src);
            }
        }

        // let pdf_path_dir = PathBuf::from("./");
        // let pdf_path_dir = pdf_path_dir.canonicalize().unwrap();
        // let pdf_path = pdf_path_dir.join(format!("./project-{}.pdf", project.id.project_id));

        // ctx.wait(
        //     project
        //         .create_pdf(pdf_path)
        //         .into_actor(&project)
        //         .map(|_, _, _| ())
        //         .map_err(|_, _, _| ()),
        // );

        Ok(project)
    }
    fn create_pdf(&self, pdf_path: PathBuf) -> io::Result<PathBuf> {
        let root = &self.tmpdir;

        let doc: Vec<pandoc_types::definition::Block> = self
            .order
            .iter()
            // TODO: Dont clone here
            .flat_map(|file_id| self.files[file_id].doc.iter().flat_map(|x| x))
            .cloned()
            .collect();

        let meta = pandoc_types::definition::Meta(HashMap::new());
        let doc = pandoc_types::definition::Pandoc(meta, doc);
        crate::doc::to_pdf(&doc, &root, &pdf_path)?;
        Ok(pdf_path)
    }
    pub fn reorder_file(&mut self, file_id: FileId, new_index: usize) {
        let old_index = self
            .order
            .iter()
            .position(|f| *f == file_id)
            .expect("file was in the order");
        if old_index == new_index {
            return;
        }

        if old_index > new_index {
            self.order.remove(old_index);
            self.order.insert(new_index, file_id);
        } else {
            self.order.insert(new_index, file_id);
            self.order.remove(old_index);
        }
    }
}

#[derive(Serialize, Clone)]
pub struct ProjectInfo {
    name: String,
    id: ProjectId,
    files: Vec<FileId>,
}
