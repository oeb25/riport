use actix::*;

use serde::{Deserialize, Serialize};

use futures::future::{join_all, Future};

use std::collections::HashMap;
use std::time::SystemTime;

use crate::client::Client;
use crate::file::{self, File, FileId, FileInfo};

use crate::s2c::*;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProjectId {
    pub project_id: i64,
}

pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub last_changed: SystemTime,
    pub order: Vec<FileId>,
    pub files: HashMap<FileId, Addr<File>>,
    pub listeners: Vec<Option<WeakAddr<Client>>>,
}

impl Project {
    pub fn new(id: ProjectId, name: String) -> Addr<Project> {
        Project::create(move |ctx| {
            let mut project = Project {
                id,
                name,
                last_changed: SystemTime::now(),
                order: vec![],
                files: HashMap::new(),
                listeners: vec![],
            };
            project.new_file("index.md".to_string(), "# Index".to_string(), ctx);
            project.new_file("abstract.md".to_string(), "# Abstract".to_string(), ctx);
            project.new_file("conlusion.md".to_string(), "# Conlusion".to_string(), ctx);
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
            file_id: self.files.len() as i64,
        };
        let file = File::new(id, self.id, ctx.address().downgrade(), name, src);
        let file_addr = file.start();
        self.files.insert(id, file_addr.clone());
        self.order.push(id);

        (id, file_addr)
    }
    pub fn generate_file_info(
        &mut self,
        _: &mut Context<Self>,
    ) -> impl Future<Item = Vec<FileInfo>> {
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

        self.generate_file_info(ctx)
            .into_actor(self)
            .then(move |res, _, _| match res {
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

    fn handle(&mut self, _: FileChanged, _: &mut Self::Context) {
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
                msg: Server2Client_Project::UpdateInfo { info: info.clone() },
            });
        }
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
                msg: Server2Client_Project::UpdateInfo { info: info.clone() },
            });
        }
    }
}
