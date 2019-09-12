use actix::*;

use serde::{Deserialize, Serialize};

use futures::future::{join_all, Future};

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::client::{Client, ClientId};
use crate::project::file::FileId;
use crate::project::{Project, ProjectId, ProjectInfo};

use crate::s2c::*;

type ListenerMap = HashMap<ClientId, WeakAddr<Client>>;

pub struct ProjectActor {
    pub project: Project,
    pub last_changed: SystemTime,
    pub project_listeners: ListenerMap,
    pub file_src_listeners: HashMap<FileId, ListenerMap>,
    pub file_doc_listeners: HashMap<FileId, ListenerMap>,
}

impl ProjectActor {
    pub fn new_(project: Project) -> Addr<ProjectActor> {
        ProjectActor::create(move |ctx| ProjectActor {
            project,
            last_changed: SystemTime::now(),
            project_listeners: HashMap::new(),
            file_src_listeners: HashMap::new(),
            file_doc_listeners: HashMap::new(),
        })
    }
    pub fn new(id: ProjectId, name: String, path: PathBuf, tmpdir: PathBuf) -> Addr<ProjectActor> {
        let mut project = Project::empty(id, name, path, tmpdir);
        project.new_file("index.md".to_string(), "# Index".to_string());
        project.new_file("abstract.md".to_string(), "# Abstract".to_string());
        project.new_file("conlusion.md".to_string(), "# Conlusion".to_string());

        project
            .write_to_disk(project.path.clone())
            .expect("failed to write to disk");

        ProjectActor::new_(project)
    }
    pub fn read_from_disk(path: PathBuf, tmpdir: PathBuf) -> Addr<ProjectActor> {
        let mut project = Project::read_from_disk(path, tmpdir).expect("failed to read from disk");

        ProjectActor::new_(project)
    }
    fn build_update_event(&self, file_id: FileId, kind: ListenKind) -> Server2Client {
        let file = self.project.files.get(&file_id).unwrap();
        let msg = match kind {
            ListenKind::Src => Server2ClientProjectFile::FileSource {
                src: file.src.clone(),
            },
            ListenKind::Doc => Server2ClientProjectFile::FileDoc {
                doc: file.doc.clone().unwrap(),
            },
        };

        Server2Client::Project {
            id: self.project.id,
            msg: Server2ClientProject::File { id: file_id, msg },
        }
    }
    fn notify(&mut self, file_id: FileId, kind: ListenKind, ignore_listener: Option<ClientId>) {
        let msg = self.build_update_event(file_id, kind);

        let listernes = match kind {
            ListenKind::Src => self.file_src_listeners.get_mut(&file_id),
            ListenKind::Doc => self.file_doc_listeners.get_mut(&file_id),
        };

        let listernes = if let Some(listernes) = listernes {
            listernes
        } else {
            unimplemented!();
        };

        let mut to_remove = vec![];
        for (client_id, listener) in listernes.iter() {
            if let Some(addr) = listener.upgrade() {
                if Some(*client_id) != ignore_listener {
                    addr.do_send(msg.clone());
                }
            } else {
                to_remove.push(*client_id);
            }
        }
        for client_id in to_remove {
            listernes.remove(&client_id);
        }
    }
}

impl Actor for ProjectActor {
    type Context = Context<Self>;
}

pub struct GetInfo;
impl Message for GetInfo {
    type Result = ProjectInfoResponse;
}

#[derive(MessageResponse)]
pub struct ProjectInfoResponse(pub ProjectInfo);

impl Handler<GetInfo> for ProjectActor {
    type Result = ProjectInfoResponse;

    fn handle(&mut self, _: GetInfo, _: &mut Self::Context) -> ProjectInfoResponse {
        ProjectInfoResponse(self.project.generate_info())
    }
}

pub struct JoinProject {
    pub addr: Addr<Client>,
    pub client_id: ClientId,
}

impl Message for JoinProject {
    type Result = ();
}

impl Handler<JoinProject> for ProjectActor {
    type Result = ();

    fn handle(&mut self, join: JoinProject, ctx: &mut Self::Context) {
        self.project_listeners
            .insert(join.client_id, join.addr.downgrade());

        let project_id = self.project.id;

        let list = self.project.generate_file_info();

        join.addr.do_send(Server2Client::Project {
            id: project_id,
            msg: Server2ClientProject::Files { list },
        });
    }
}

pub struct LeaveProject {
    pub client_id: ClientId,
}

impl Message for LeaveProject {
    type Result = ();
}

impl Handler<LeaveProject> for ProjectActor {
    type Result = ();

    fn handle(&mut self, leave: LeaveProject, _: &mut Self::Context) {
        self.project_listeners.remove(&leave.client_id);
    }
}

pub struct ReorderFile {
    pub id: FileId,
    pub new_index: usize,
}
impl Message for ReorderFile {
    type Result = ();
}

impl Handler<ReorderFile> for ProjectActor {
    type Result = ();

    fn handle(&mut self, reorder: ReorderFile, _: &mut Self::Context) {
        self.project.reorder_file(reorder.id, reorder.new_index);

        self.last_changed = SystemTime::now();

        let info = self.project.generate_info();

        for l in self.project_listeners.values().filter_map(|f| f.upgrade()) {
            l.do_send(Server2Client::Project {
                id: self.project.id,
                msg: Server2ClientProject::UpdateInfo { info: info.clone() },
            });
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ListenKind {
    Src,
    Doc,
}

#[derive(Message)]
pub struct JoinFile {
    pub file_id: FileId,
    pub client_id: ClientId,
    pub kind: ListenKind,
    pub addr: Addr<Client>,
}

impl Handler<JoinFile> for ProjectActor {
    type Result = ();
    fn handle(&mut self, msg: JoinFile, _: &mut Context<Self>) {
        let map = match msg.kind {
            ListenKind::Src => &mut self.file_src_listeners,
            ListenKind::Doc => &mut self.file_doc_listeners,
        };
        map.entry(msg.file_id)
            .or_insert_with(|| HashMap::new())
            .insert(msg.client_id, msg.addr.downgrade());

        msg.addr
            .do_send(self.build_update_event(msg.file_id, msg.kind));
    }
}

#[derive(Message)]
pub struct LeaveFile {
    pub file_id: FileId,
    pub client_id: ClientId,
    pub kind: ListenKind,
}

impl Handler<LeaveFile> for ProjectActor {
    type Result = ();
    fn handle(&mut self, msg: LeaveFile, _: &mut Context<Self>) {
        let map = match msg.kind {
            ListenKind::Src => &mut self.file_src_listeners,
            ListenKind::Doc => &mut self.file_doc_listeners,
        };
        map.entry(msg.file_id)
            .or_insert_with(|| HashMap::new())
            .remove(&msg.client_id);
    }
}

#[derive(Message)]
pub struct EditFile {
    pub file_id: FileId,
    pub src: String,
    pub ignore_listener: ClientId,
}

impl Handler<EditFile> for ProjectActor {
    type Result = ();
    fn handle(&mut self, msg: EditFile, _: &mut Context<Self>) {
        if let Some(file) = self.project.files.get_mut(&msg.file_id) {
            file.update_src(msg.src);
            self.notify(msg.file_id, ListenKind::Src, Some(msg.ignore_listener));
            self.notify(msg.file_id, ListenKind::Doc, None);
        }
    }
}
