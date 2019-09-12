use actix::*;

use pandoc_types::definition::Block;

use serde::{Deserialize, Serialize};

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::client::Client;
use crate::project::{Project, ProjectId};

pub type Doc = Vec<Block>;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FileId {
    pub file_id: u64,
}

pub struct File {
    pub id: FileId,
    pub project_id: ProjectId,
    pub name: String,
    pub src: String,
    pub doc: Option<Doc>,
    pub tmpdir: PathBuf,
}

impl File {
    pub fn new(
        id: FileId,
        project_id: ProjectId,
        name: String,
        src: String,
        tmpdir: PathBuf,
    ) -> File {
        let mut file = File {
            id,
            project_id,
            name,
            src,
            doc: None,
            tmpdir,
        };
        file.compile();
        file
    }
    pub fn compile(&mut self) {
        let doc = crate::doc::compile(&self.src, &self.tmpdir).expect("failed to compile");
        self.doc = Some(doc.1);
    }
    pub fn write_to_disk(&self, dir: &Path) -> io::Result<()> {
        unimplemented!()
    }
    pub fn get_info(&self) -> FileInfo {
        FileInfo {
            name: self.name.clone(),
            id: self.id,
        }
    }
    pub fn update_src(&mut self, src: String) {
        self.src = src;
        self.compile()
    }
}

#[derive(Serialize, Clone, MessageResponse)]
pub struct FileInfo {
    pub name: String,
    pub id: FileId,
}
