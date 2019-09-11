use actix::*;
use pandoc_types::definition::Block;

use serde::{Deserialize, Serialize};

use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::client::Client;
use crate::project::{FileChanged, Project, ProjectId};

use crate::s2c::*;

pub type Doc = Vec<Block>;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FileId {
    pub file_id: u64,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ListenKind {
    Doc,
    Src,
}

pub struct File {
    pub id: FileId,
    pub project_id: ProjectId,
    pub project: WeakAddr<Project>,
    pub name: String,
    pub src: String,
    pub doc: Option<Doc>,
    pub listeners: Vec<Option<(ListenKind, WeakAddr<Client>)>>,
    pub tmpdir: PathBuf,
}

impl File {
    pub fn new(
        id: FileId,
        project_id: ProjectId,
        project: WeakAddr<Project>,
        name: String,
        src: String,
        tmpdir: PathBuf,
    ) -> File {
        let mut file = File {
            id,
            project_id,
            project,
            name,
            src,
            doc: None,
            listeners: vec![],
            tmpdir,
        };
        file.compile();
        file
    }
    pub fn compile(&mut self) {
        let doc = crate::doc::compile(&self.src, &self.tmpdir).expect("failed to compile");
        self.doc = Some(doc.1);
    }
    fn msg(&self, msg: Server2ClientProjectFile) -> Server2Client {
        Server2Client::Project {
            id: self.project_id,
            msg: Server2ClientProject::File { id: self.id, msg },
        }
    }
}

impl Actor for File {
    type Context = Context<Self>;
}

pub struct GetInfo;
impl Message for GetInfo {
    type Result = FileInfo;
}

impl Handler<GetInfo> for File {
    type Result = FileInfo;

    fn handle(&mut self, _: GetInfo, _: &mut Self::Context) -> FileInfo {
        FileInfo {
            name: self.name.clone(),
            id: self.id,
            last_changed: SystemTime::now(),
        }
    }
}

pub struct GetDoc;
impl Message for GetDoc {
    type Result = GetDocResponce;
}

#[derive(MessageResponse)]
pub struct GetDocResponce {
    pub doc: Doc,
}

impl Handler<GetDoc> for File {
    type Result = GetDocResponce;

    fn handle(&mut self, _: GetDoc, _: &mut Self::Context) -> GetDocResponce {
        // TODO: Don't recompile if not needed
        self.compile();
        let doc = self.doc.clone().unwrap();
        GetDocResponce { doc }
    }
}

#[derive(Serialize, MessageResponse)]
pub struct FileInfo {
    pub name: String,
    pub last_changed: SystemTime,
    pub id: FileId,
}

pub struct JoinFile {
    pub kind: ListenKind,
    pub addr: Addr<Client>,
}

impl Message for JoinFile {
    type Result = usize;
}

impl Handler<JoinFile> for File {
    type Result = usize;

    fn handle(&mut self, join: JoinFile, _: &mut Self::Context) -> usize {
        let id = self.listeners.len();
        self.listeners
            .push(Some((join.kind, join.addr.downgrade())));

        join.addr.do_send(self.msg(if join.kind == ListenKind::Src {
            Server2ClientProjectFile::FileSource {
                src: self.src.clone(),
            }
        } else {
            Server2ClientProjectFile::FileDoc {
                doc: self.doc.clone().unwrap_or(vec![]),
            }
        }));

        id
    }
}

pub struct LeaveFile {
    pub id: usize,
}

impl Message for LeaveFile {
    type Result = ();
}

impl Handler<LeaveFile> for File {
    type Result = ();

    fn handle(&mut self, leave: LeaveFile, _: &mut Self::Context) {
        self.listeners[leave.id] = None;
    }
}

pub struct EditFile {
    pub src: String,
    pub ignore: Option<usize>,
}

impl Message for EditFile {
    type Result = ();
}

impl Handler<EditFile> for File {
    type Result = ();

    fn handle(&mut self, edit: EditFile, _: &mut Self::Context) {
        if let Some(p) = self.project.upgrade() {
            p.do_send(FileChanged { id: self.id });
        }
        self.src = edit.src;
        let ignore = edit.ignore;
        for (kind, l) in self.listeners.iter().enumerate().filter_map(|(i, f)| {
            if Some(i) != ignore {
                f.as_ref()
            } else {
                None
            }
        }) {
            if *kind != ListenKind::Src {
                continue;
            }
            let msg = self.msg(Server2ClientProjectFile::FileSource {
                src: self.src.clone(),
            });
            if let Some(l) = l.upgrade() {
                l.do_send(msg)
            } else {
                // TODO: Remove dead listeners
            }
        }
        self.compile();
        for (kind, l) in self.listeners.iter().enumerate().filter_map(|(i, f)| {
            if Some(i) != ignore {
                f.as_ref()
            } else {
                None
            }
        }) {
            if *kind != ListenKind::Doc {
                continue;
            }
            let msg = self.msg(Server2ClientProjectFile::FileDoc {
                doc: self.doc.clone().unwrap(),
            });
            if let Some(l) = l.upgrade() {
                l.do_send(msg)
            } else {
                // TODO: Remove dead listeners
            }
        }
    }
}

pub struct WriteToDisk {
    pub dir: PathBuf,
}

impl Message for WriteToDisk {
    type Result = io::Result<PathBuf>;
}

impl Handler<WriteToDisk> for File {
    type Result = io::Result<PathBuf>;

    fn handle(&mut self, wtd: WriteToDisk, _: &mut Context<Self>) -> io::Result<PathBuf> {
        let p = wtd.dir.join(&self.name).with_extension("md");
        fs::write(&p, &self.src)?;
        Ok(p)
    }
}
