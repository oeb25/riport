use actix::*;
use futures::future::{err, join_all, ok};
use futures::prelude::*;
use std::collections::HashMap;

use crate::client::{Client, Server2Client};
use crate::project::{GetInfo, Project, ProjectId, ProjectInfo};

pub struct Hub {
    connections: Vec<Option<Addr<Client>>>,
    projects: HashMap<ProjectId, Addr<Project>>,
}

impl Default for Hub {
    fn default() -> Hub {
        let mut hub = Hub {
            connections: vec![],
            projects: HashMap::new(),
        };
        hub.create_project("Sample Project".to_string());
        hub.create_project("Another Project".to_string());
        hub
    }
}

impl Hub {
    fn generate_project_info_list(
        &mut self,
        ctx: &mut Context<Self>,
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
        let project = Project::new(id, name);
        let project_addr = project.start();
        self.projects.insert(id, project_addr.clone());

        (id, project_addr)
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

        println!("bount to generate");
        self.generate_project_info_list(ctx)
            .into_actor(self)
            .then(move |res, act, ctx| {
                println!("generated");
                match res {
                    Ok(list) => msg.adder.do_send(Server2Client::Projects { list }),
                    _ => {}
                }

                fut::ok(())
            })
            .wait(ctx);
        println!("after");

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
    fn handle(&mut self, msg: CreateProject, ctx: &mut Context<Self>) -> CreateProjectRes {
        let (id, project) = self.create_project(msg.name);

        CreateProjectRes { id }
    }
}

pub struct GetProject {
    pub id: ProjectId,
}

impl Message for GetProject {
    type Result = Option<Addr<Project>>;
}

#[derive(MessageResponse)]
pub struct GetProjectRes {
    pub addr: Addr<Project>,
}

impl Handler<GetProject> for Hub {
    type Result = Option<Addr<Project>>;
    fn handle(&mut self, msg: GetProject, ctx: &mut Context<Self>) -> Option<Addr<Project>> {
        self.projects.get(&msg.id).cloned()
    }
}

// impl Handler<Msg> for Hub {
//     type Result = MsgResponse;

//     fn handle(&mut self, msg: Msg, ctx: &mut Context<Self>) -> MsgResponse {
//         println!("Hub got message");
//         match msg {
//             Msg::Connect { addr } => {
//                 let id = self.connections.len();
//                 println!("New connection: {}", id);
//                 self.connections.push(Some(addr));
//                 MsgResponse::AssignID { id }
//             }
//             Msg::Disconnect { id } => {
//                 println!("Disconnected: {}", id);
//                 self.connections[id] = None;
//                 MsgResponse::None
//             }
//             Msg::Message { contents } => {
//                 println!("Message recived: {}", contents);
//                 for con in self.connections.iter().filter_map(|x| x.as_ref()) {
//                     con.do_send(HubMsg::Message {
//                         contents: contents.clone(),
//                     });
//                 }
//                 MsgResponse::None
//             }
//         }
//     }
// }
