use actix::*;
use pandoc_types::definition::Block;

use serde::{Deserialize, Serialize};

use std::time::SystemTime;

use crate::client::Client;
use crate::project::{FileChanged, Project, ProjectId};

use crate::s2c::*;

pub type Doc = Vec<Block>;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FileId {
    pub file_id: i64,
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
}

impl File {
    pub fn new(
        id: FileId,
        project_id: ProjectId,
        project: WeakAddr<Project>,
        name: String,
        src: String,
    ) -> File {
        let mut file = File {
            id,
            project_id,
            project,
            name,
            src,
            doc: None,
            listeners: vec![],
        };
        file.compile();
        file
    }
    pub fn compile(&mut self) {
        let tmp = std::path::PathBuf::from("./tmp");
        std::fs::create_dir_all(&tmp).expect("failed to create run dir");

        let doc = crate::doc::compile(&self.src, &tmp).expect("failed to compile");
        self.doc = Some(doc.1);
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

#[derive(Serialize, MessageResponse)]
pub struct FileInfo {
    name: String,
    last_changed: SystemTime,
    id: FileId,
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

        join.addr.do_send(Server2Client::Project {
            id: self.project_id,
            msg: Server2Client_Project::File {
                id: self.id,
                msg: if join.kind == ListenKind::Src {
                    Server2Client_Project_File::FileSource {
                        src: self.src.clone(),
                    }
                } else {
                    Server2Client_Project_File::FileDoc {
                        doc: self.doc.clone().unwrap_or(vec![]),
                    }
                },
            },
        });

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
            let msg = Server2Client::Project {
                id: self.project_id,
                msg: Server2Client_Project::File {
                    id: self.id,
                    msg: Server2Client_Project_File::FileSource {
                        src: self.src.clone(),
                    },
                },
            };
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
            let msg = Server2Client::Project {
                id: self.project_id,
                msg: Server2Client_Project::File {
                    id: self.id,
                    msg: Server2Client_Project_File::FileDoc {
                        doc: self.doc.clone().unwrap(),
                    },
                },
            };
            if let Some(l) = l.upgrade() {
                l.do_send(msg)
            } else {
                // TODO: Remove dead listeners
            }
        }
    }
}
