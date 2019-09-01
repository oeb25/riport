use actix::*;
use pandoc_types::definition::Block;

use serde::{Deserialize, Serialize};

use std::time::SystemTime;

use crate::client::{Client, Server2Client, Server2Client_Project, Server2Client_Project_File};
use crate::project::ProjectId;

pub type Doc = Vec<Block>;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FileId {
    pub file_id: i64,
}

pub struct File {
    pub id: FileId,
    pub project_id: ProjectId,
    pub name: String,
    pub src: String,
    pub doc: Option<Doc>,
    pub listeners: Vec<Option<WeakAddr<Client>>>,
}

impl File {
    pub fn new(id: FileId, project_id: ProjectId, name: String) -> File {
        File {
            id,
            project_id,
            name,
            src: String::new(),
            doc: None,
            listeners: vec![],
        }
    }
    // pub fn compile(
    //     &mut self,
    //     ctx: &mut Context<Self>,
    // ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = ()> {
    //     let doc = crate::doc::compile(&self.src, &std::path::PathBuf::from("./tmp"))
    //         .expect("failed to compile");
    //     self.doc = Some(doc.1);
    //     fut::ok(())
    // }
    pub fn compile(&mut self) {
        let tmp = std::path::PathBuf::from("./tmp");
        std::fs::create_dir_all(&tmp);

        let doc = crate::doc::compile(&self.src, &tmp).expect("failed to compile");
        self.doc = Some(doc.1);
    }
}

impl Actor for File {
    type Context = Context<Self>;
}

#[derive(Message)]
struct ChangeFile {
    src: String,
}

impl Handler<ChangeFile> for File {
    type Result = ();

    fn handle(&mut self, msg: ChangeFile, ctx: &mut Context<Self>) {
        self.src = msg.src;
    }
}

pub struct GetInfo;
impl Message for GetInfo {
    type Result = FileInfo;
}

impl Handler<GetInfo> for File {
    type Result = FileInfo;

    fn handle(&mut self, _: GetInfo, ctx: &mut Self::Context) -> FileInfo {
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
    pub addr: Addr<Client>,
}

impl Message for JoinFile {
    type Result = usize;
}

impl Handler<JoinFile> for File {
    type Result = usize;

    fn handle(&mut self, join: JoinFile, ctx: &mut Self::Context) -> usize {
        let id = self.listeners.len();
        self.listeners.push(Some(join.addr.downgrade()));

        join.addr.do_send(Server2Client::Project {
            id: self.project_id,
            msg: Server2Client_Project::File {
                id: self.id,
                msg: Server2Client_Project_File::FileSource {
                    src: self.src.clone(),
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

    fn handle(&mut self, leave: LeaveFile, ctx: &mut Self::Context) {
        self.listeners[leave.id] = None;
    }
}

pub struct EditFile {
    pub src: String,
}

impl Message for EditFile {
    type Result = ();
}

impl Handler<EditFile> for File {
    type Result = ();

    fn handle(&mut self, edit: EditFile, ctx: &mut Self::Context) {
        self.src = edit.src;
        for l in self.listeners.iter().filter_map(|f| f.as_ref()) {
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
        for l in self.listeners.iter().filter_map(|f| f.as_ref()) {
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
