use actix::fut::Either;
use actix::*;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;

use futures::prelude::*;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::file::{Doc, EditFile, File, FileId, FileInfo, JoinFile, LeaveFile};
use crate::hub::{self, CreateProject, GetProject, Hub};
use crate::project::{GetFile, JoinProject, LeaveProject, Project, ProjectId, ProjectInfo};

pub struct Client {
    id: usize,
    hub: Addr<Hub>,
    projects: HashMap<ProjectId, (Option<usize>, Addr<Project>)>,
    files: HashMap<(ProjectId, FileId), (Option<usize>, Addr<File>)>,
}

impl Client {
    pub fn new(hub: Addr<Hub>) -> Client {
        Client {
            id: 0,
            hub,
            projects: HashMap::new(),
            files: HashMap::new(),
        }
    }
    pub fn send(msg: Server2Client, ctx: &mut ws::WebsocketContext<Self>) {
        let text = serde_json::to_string(&msg).unwrap();
        ctx.text(text);
    }
    pub fn get_project(
        &mut self,
        id: ProjectId,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = (Option<usize>, Addr<Project>)>
    {
        if let Some(p) = self.projects.get(&id) {
            Either::A(fut::ok(p.clone()))
        } else {
            let f = self
                .hub
                .send(GetProject { id })
                .into_actor(self)
                .map(move |res, act, ctx| match res {
                    Some(project) => {
                        act.projects.insert(id, (None, project.clone()));
                        (None, project)
                    }
                    _ => unimplemented!(),
                })
                .map_err(|e, act, ctx| {
                    panic!("{:?}", e);
                    ()
                });
            Either::B(f)
        }
    }
    pub fn get_file(
        &mut self,
        project_id: ProjectId,
        file_id: FileId,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = (Option<usize>, Addr<File>)> {
        if let Some(p) = self.files.get(&(project_id, file_id)) {
            Either::A(fut::ok(p.clone()))
        } else {
            let f = self
                .get_project(project_id, ctx)
                .then(move |res, act, ctx| match res {
                    Ok((_, project)) => project.send(GetFile { id: file_id }).into_actor(act),
                    _ => unimplemented!(),
                })
                .map(move |file, act, ctx| {
                    let file = file.expect("file not found");
                    act.files
                        .insert((project_id, file_id), (None, file.clone()));
                    (None, file)
                })
                .map_err(|e, act, ctx| {
                    panic!("{:?}", e);
                    ()
                });;
            Either::B(f)
        }
    }
    pub fn join_project(
        &mut self,
        project_id: ProjectId,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = ()> {
        let my_adder = ctx.address();
        self.get_project(project_id, ctx)
            .then(move |res, act, ctx| match res {
                Ok((id, project)) => {
                    if let Some(id) = id {
                        Either::A(fut::ok(()))
                    } else {
                        let f = project
                            .send(JoinProject { addr: my_adder })
                            .into_actor(act)
                            .map(move |id, act, ctx| {
                                act.projects.insert(project_id, (Some(id), project));
                            })
                            .map_err(|e, act, ctx| {
                                panic!("{:?}", e);
                                ()
                            });
                        Either::B(f)
                    }
                }
                _ => unimplemented!(),
            })
    }
    pub fn leave_project(
        &mut self,
        project_id: ProjectId,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = ()> {
        self.get_project(project_id, ctx)
            .map(move |(con, project), act, ctx| {
                project.do_send(LeaveProject { id: con.unwrap() });
                act.projects.remove(&project_id);
            })
    }
    pub fn join_file_source(
        &mut self,
        project_id: ProjectId,
        file_id: FileId,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = ()> {
        let my_adder = ctx.address();
        self.get_file(project_id, file_id, ctx)
            .then(move |res, act, ctx| match res {
                Ok((id, file)) => {
                    if let Some(id) = id {
                        Either::A(fut::ok(()))
                    } else {
                        let f = file
                            .send(JoinFile { addr: my_adder })
                            .into_actor(act)
                            .map(move |id, act, ctx| {
                                act.files.insert((project_id, file_id), (Some(id), file));
                            })
                            .map_err(|e, act, ctx| {
                                panic!("{:?}", e);
                                ()
                            });
                        Either::B(f)
                    }
                }
                _ => unimplemented!(),
            })
    }
    pub fn leave_file_source(
        &mut self,
        project_id: ProjectId,
        file_id: FileId,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = ()> {
        self.get_file(project_id, file_id, ctx)
            .map(move |(con, file), act, ctx| {
                file.do_send(LeaveFile { id: con.unwrap() });
                act.files.remove(&(project_id, file_id));
            })
    }
    pub fn edit_file(
        &mut self,
        project_id: ProjectId,
        file_id: FileId,
        src: String,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = ()> {
        self.get_file(project_id, file_id, ctx)
            .map(|(con, file), act, ctx| {
                file.do_send(EditFile { src });
            })
    }
}

impl Actor for Client {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hub
            .send(hub::Connect {
                adder: ctx.address(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(hub::ConnectRes { id }) => {
                        act.id = id;
                    }
                    _ => {}
                }
                fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // self.hub.do_send(Msg::Disconnect { id: self.id });
        Running::Stop
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Client {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(contents) => {
                let msg: Client2Server = serde_json::from_str(&contents).unwrap();
                println!("Got: {:?}", msg);
                match msg {
                    Client2Server::CreateProject { project_name } => {
                        println!("unhandled create event");
                    }
                    Client2Server::Project {
                        id: project_id,
                        msg,
                    } => match msg {
                        Client2Server_Project::JoinProject => {
                            let f = self.join_project(project_id, ctx);
                            ctx.wait(f);
                        }
                        Client2Server_Project::LeaveProject => {
                            let f = self.leave_project(project_id, ctx);
                            ctx.wait(f);
                        }
                        Client2Server_Project::File { id: file_id, msg } => match msg {
                            Client2Server_Project_File::JoinFileSource => {
                                let f = self.join_file_source(project_id, file_id, ctx);
                                ctx.wait(f);
                            }
                            Client2Server_Project_File::LeaveFileSource => {
                                let f = self.leave_file_source(project_id, file_id, ctx);
                                ctx.wait(f);
                            }
                            Client2Server_Project_File::EditFileSource { contents } => {
                                let f = self.edit_file(project_id, file_id, contents, ctx);
                                ctx.wait(f);
                            }
                            _ => {
                                println!("unhandled file event {:?}", msg);
                            }
                        },
                        _ => {
                            println!("unhandled project event {:?}", msg);
                        }
                    },
                }
                // self.hub.do_send(Msg::Message { contents });
            }
            ws::Message::Binary(bin) => ctx.binary(bin),
            _ => (),
        }
    }
}

// impl Handler<HubMsg> for Client {
//     type Result = ();

//     fn handle(&mut self, msg: HubMsg, ctx: &mut Self::Context) {
//         // match msg {
//         //     HubMsg::Message { contents } => {
//         //         ctx.text(contents);
//         //     }
//         // }
//     }
// }

impl Handler<Server2Client> for Client {
    type Result = ();

    fn handle(&mut self, msg: Server2Client, ctx: &mut Self::Context) {
        Client::send(msg, ctx);
    }
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Lock {
    Unlock,
    LockBy {},
    LockByMe,
}

#[derive(Serialize, Message)]
#[serde(tag = "type")]
pub enum Server2Client {
    Projects {
        list: Vec<ProjectInfo>,
    },
    Project {
        id: ProjectId,
        msg: Server2Client_Project,
    },
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Server2Client_Project {
    Files {
        list: Vec<FileInfo>,
    },
    File {
        id: FileId,
        msg: Server2Client_Project_File,
    },
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Server2Client_Project_File {
    FileLock { lock: Lock },
    FileSource { src: String },
    FileDoc { doc: Doc },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Client2Server {
    CreateProject {
        project_name: String,
    },
    Project {
        id: ProjectId,
        msg: Client2Server_Project,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Client2Server_Project {
    JoinProject,
    LeaveProject,
    CreateFile {
        file_name: String,
    },
    File {
        id: FileId,
        msg: Client2Server_Project_File,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Client2Server_Project_File {
    JoinFileSource,
    LeaveFileSource,
    EditFileSource { contents: String },
    JoinFileDoc,
    LeaveFileDoc,
}
