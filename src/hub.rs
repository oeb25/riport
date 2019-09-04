use actix::*;
use futures::future::join_all;
use futures::prelude::*;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::client::Client;
use crate::project::{GetInfo, Project, ProjectId, ProjectInfo};

use crate::s2c::*;

pub struct Hub {
    projects_path: PathBuf,
    connections: Vec<Option<Addr<Client>>>,
    projects: HashMap<ProjectId, Addr<Project>>,
}

impl Hub {
    pub fn new(projects_path: PathBuf) -> io::Result<Hub> {
        let mut hub = Hub {
            projects_path,
            connections: vec![],
            projects: HashMap::new(),
        };

        fs::create_dir_all(&hub.projects_path)?;

        for e in fs::read_dir(&hub.projects_path)? {
            let e = e?;
            let path = e.path();
            if path.is_dir() {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let path = hub.projects_path.join(&name);
                hub.load_project(name, path);
            } else {
                // TODO
            }
        }

        // hub.create_project("Sample Project".to_string());
        // // hub.create_project("Another Project".to_string());
        // hub.load_project(
        //     "Another Project".to_string(),
        //     PathBuf::from("./tmp/Another Project"),
        // );
        Ok(hub)
    }
    fn generate_project_info_list(
        &mut self,
        _: &mut Context<Self>,
    ) -> impl Future<Item = Vec<ProjectInfo>> {
        let mut projects = vec![];

        for project in self.projects.values() {
            let res = project.send(GetInfo);
            projects.push(res);
        }

        join_all(projects)
    }
    fn create_project(&mut self, name: String) -> (ProjectId, Addr<Project>) {
        let id = ProjectId {
            project_id: self.projects.len() as i64,
        };
        let path = self.projects_path.join(&name);
        let project = Project::new(id, name, path);
        self.projects.insert(id, project.clone());

        (id, project)
    }
    fn load_project(&mut self, name: String, path: PathBuf) -> (ProjectId, Addr<Project>) {
        println!("loading project at {:?}", path);
        let id = ProjectId {
            project_id: self.projects.len() as i64,
        };
        let project = Project::read_from_disk(path.to_owned());
        self.projects.insert(id, project.clone());

        (id, project)
    }
}

impl Actor for Hub {
    type Context = Context<Self>;
}

pub struct Connect {
    pub adder: Addr<Client>,
}

impl Message for Connect {
    type Result = ConnectRes;
}

#[derive(MessageResponse)]
pub struct ConnectRes {
    pub id: usize,
}

impl Handler<Connect> for Hub {
    type Result = ConnectRes;
    fn handle(&mut self, msg: Connect, ctx: &mut Context<Self>) -> ConnectRes {
        let id = self.connections.len();
        self.connections.push(Some(msg.adder.clone()));

        self.generate_project_info_list(ctx)
            .into_actor(self)
            .then(move |res, _act, _| {
                match res {
                    Ok(list) => msg.adder.do_send(Server2Client::Projects { list }),
                    _ => {}
                }

                fut::ok(())
            })
            .wait(ctx);

        ConnectRes { id }
    }
}

pub struct CreateProject {
    pub name: String,
}

impl Message for CreateProject {
    type Result = CreateProjectRes;
}

#[derive(MessageResponse)]
pub struct CreateProjectRes {
    pub id: ProjectId,
}

impl Handler<CreateProject> for Hub {
    type Result = CreateProjectRes;
    fn handle(&mut self, msg: CreateProject, _: &mut Context<Self>) -> CreateProjectRes {
        let (id, _) = self.create_project(msg.name);

        CreateProjectRes { id }
    }
}

pub struct GetProject {
    pub id: ProjectId,
}

impl Message for GetProject {
    type Result = Option<Addr<Project>>;
}

impl Handler<GetProject> for Hub {
    type Result = Option<Addr<Project>>;
    fn handle(&mut self, msg: GetProject, _: &mut Context<Self>) -> Option<Addr<Project>> {
        self.projects.get(&msg.id).cloned()
    }
}
