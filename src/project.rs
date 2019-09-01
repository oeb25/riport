use actix::*;

use serde::{Deserialize, Serialize};

use futures::future::{join_all, Future};

use std::collections::HashMap;
use std::time::SystemTime;

use crate::client::{Client, Server2Client, Server2Client_Project};
use crate::file::{self, File, FileId, FileInfo};

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProjectId {
    pub project_id: i64,
}

pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub files: HashMap<FileId, Addr<File>>,
    pub listeners: Vec<Option<WeakAddr<Client>>>,
}

impl Project {
    pub fn new(id: ProjectId, name: String) -> Project {
        let mut project = Project {
            id,
            name,
            files: HashMap::new(),
            listeners: vec![],
        };
        project.new_file("index.md".to_string());
        project.new_file("abstract.md".to_string());
        project
    }
    pub fn new_file(&mut self, name: String) -> (FileId, Addr<File>) {
        let id = FileId {
            file_id: self.files.len() as i64,
        };
        let file = File::new(id, self.id, name);
        let file_addr = file.start();
        self.files.insert(id, file_addr.clone());

        (id, file_addr)
    }
    pub fn generate_file_info(
        &mut self,
        ctx: &mut Context<Self>,
    ) -> impl Future<Item = Vec<FileInfo>> {
        let mut files = vec![];

        for file in self.files.values() {
            let res = file.send(file::GetInfo);
            files.push(res);
        }

        join_all(files)
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

    fn handle(&mut self, _: GetInfo, ctx: &mut Self::Context) -> ProjectInfo {
        ProjectInfo {
            name: self.name.clone(),
            id: self.id,
            last_changed: SystemTime::now(),
            files: self.files.keys().cloned().collect(),
        }
    }
}

#[derive(Serialize, MessageResponse)]
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

        self.generate_file_info(ctx)
            .into_actor(self)
            .then(move |res, act, ctx| match res {
                Ok(list) => {
                    join.addr.do_send(Server2Client::Project {
                        id: project_id,
                        msg: Server2Client_Project::Files { list },
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

    fn handle(&mut self, leave: LeaveProject, ctx: &mut Self::Context) {
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

    fn handle(&mut self, get_file: GetFile, ctx: &mut Self::Context) -> Option<Addr<File>> {
        self.files.get(&get_file.id).cloned()
    }
}
