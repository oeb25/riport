use actix::fut::Either;
use actix::*;
use actix_web_actors::ws;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::hub::{self, GetProject, Hub};
use crate::project::file::{File, FileId};
use crate::project::{Project, ProjectId};
use crate::project_actor::{
    EditFile, JoinFile, JoinProject, LeaveFile, LeaveProject, ListenKind, ProjectActor, ReorderFile,
};

use crate::c2s::*;
use crate::s2c::*;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClientId {
    pub client_id: u64,
}

pub struct Client {
    id: ClientId,
    hub: Addr<Hub>,
    projects: HashMap<ProjectId, Addr<ProjectActor>>,
}

impl Client {
    pub fn new(id: ClientId, hub: Addr<Hub>) -> Client {
        Client {
            id,
            hub,
            projects: HashMap::new(),
        }
    }
    pub fn send(msg: Server2Client, ctx: &mut ws::WebsocketContext<Self>) {
        let text = serde_json::to_string(&msg).unwrap();
        ctx.text(text);
    }
    pub fn get_project(
        &mut self,
        id: ProjectId,
        _: &mut ws::WebsocketContext<Self>,
    ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = Addr<ProjectActor>> {
        if let Some(p) = self.projects.get(&id) {
            Either::A(fut::ok(p.clone()))
        } else {
            let f = self
                .hub
                .send(GetProject { id })
                .into_actor(self)
                .map(move |res, act, _| match res {
                    Some(project) => {
                        act.projects.insert(id, project.clone());
                        project
                    }
                    _ => unimplemented!(),
                })
                .map_err(|e, _, _| {
                    panic!("{:?}", e);
                });
            Either::B(f)
        }
    }
    // pub fn get_file(
    //     &mut self,
    //     project_id: ProjectId,
    //     file_id: FileId,
    //     ctx: &mut ws::WebsocketContext<Self>,
    // ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = FileCache> {
    //     if let Some(p) = self.files.get(&(project_id, file_id)) {
    //         Either::A(fut::ok(p.clone()))
    //     } else {
    //         let f = self
    //             .get_project(project_id, ctx)
    //             .then(move |res, act, _| match res {
    //                 Ok((_, project)) => project.send(GetFile { id: file_id }).into_actor(act),
    //                 _ => unimplemented!(),
    //             })
    //             .map(move |file, act, _| {
    //                 let file = file.expect("file not found");
    //                 let cache = FileCache {
    //                     src_id: None,
    //                     doc_id: None,
    //                     file: file.clone(),
    //                 };
    //                 act.files.insert((project_id, file_id), cache.clone());
    //                 cache
    //             })
    //             .map_err(|e, _, _| {
    //                 panic!("{:?}", e);
    //             });
    //         Either::B(f)
    //     }
    // }
    pub fn join_project(
        &mut self,
        project_id: ProjectId,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = ()> {
        let client_id = self.id;
        let my_adder = ctx.address();
        self.get_project(project_id, ctx)
            .then(move |res, act, _| match res {
                Ok(project) => {
                    project.do_send(JoinProject {
                        addr: my_adder,
                        client_id,
                    });
                    fut::ok(())
                }
                _ => unimplemented!(),
            })
    }
    pub fn leave_project(
        &mut self,
        project_id: ProjectId,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = ()> {
        let client_id = self.id;
        self.get_project(project_id, ctx)
            .map(move |project, act, _| {
                project.do_send(LeaveProject { client_id });
                act.projects.remove(&project_id);
            })
    }
    pub fn join_file(
        &mut self,
        project_id: ProjectId,
        file_id: FileId,
        kind: ListenKind,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = ()> {
        let my_adder = ctx.address();
        let client_id = self.id;
        self.get_project(project_id, ctx)
            .then(move |res, act, _| match res {
                Ok(project) => {
                    project.do_send(JoinFile {
                        file_id,
                        client_id,
                        addr: my_adder,
                        kind,
                    });
                    fut::ok(())
                }
                _ => unimplemented!(),
            })
    }
    pub fn leave_file(
        &mut self,
        project_id: ProjectId,
        file_id: FileId,
        kind: ListenKind,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = ()> {
        let client_id = self.id;
        self.get_project(project_id, ctx)
            .map(move |project, act, _| {
                project.do_send(LeaveFile {
                    file_id,
                    client_id,
                    kind,
                });
            })
    }
    pub fn edit_file(
        &mut self,
        project_id: ProjectId,
        file_id: FileId,
        src: String,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = ()> {
        let ignore_listener = self.id;
        self.get_project(project_id, ctx)
            .map(move |project, act, _| {
                project.do_send(EditFile {
                    file_id,
                    src,
                    ignore_listener,
                });
            })
    }
    pub fn reorder_file(
        &mut self,
        project_id: ProjectId,
        file_id: FileId,
        new_index: usize,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = ()> {
        self.get_project(project_id, ctx).map(move |project, _, _| {
            project.do_send(ReorderFile {
                id: file_id,
                new_index,
            });
        })
    }
}

impl Actor for Client {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hub.do_send(hub::Connect {
            client_id: self.id,
            adder: ctx.address(),
        });
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
                    Client2Server::CreateProject { .. } => {
                        println!("unhandled create event");
                    }
                    Client2Server::Project {
                        id: project_id,
                        msg,
                    } => match msg {
                        Client2ServerProject::JoinProject => {
                            let f = self.join_project(project_id, ctx);
                            ctx.wait(f);
                        }
                        Client2ServerProject::LeaveProject => {
                            let f = self.leave_project(project_id, ctx);
                            ctx.wait(f);
                        }
                        Client2ServerProject::Reorder {
                            id: file_id,
                            new_index,
                        } => {
                            let f = self.reorder_file(project_id, file_id, new_index, ctx);
                            ctx.wait(f);
                        }
                        Client2ServerProject::File { id: file_id, msg } => match msg {
                            Client2ServerProjectFile::JoinFileSource => {
                                let f = self.join_file(project_id, file_id, ListenKind::Src, ctx);
                                ctx.wait(f);
                            }
                            Client2ServerProjectFile::LeaveFileSource => {
                                let f = self.leave_file(project_id, file_id, ListenKind::Src, ctx);
                                ctx.wait(f);
                            }
                            Client2ServerProjectFile::JoinFileDoc => {
                                let f = self.join_file(project_id, file_id, ListenKind::Doc, ctx);
                                ctx.wait(f);
                            }
                            Client2ServerProjectFile::LeaveFileDoc => {
                                let f = self.leave_file(project_id, file_id, ListenKind::Doc, ctx);
                                ctx.wait(f);
                            }
                            Client2ServerProjectFile::EditFileSource { contents } => {
                                let f = self.edit_file(project_id, file_id, contents, ctx);
                                ctx.wait(f);
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

impl Handler<Server2Client> for Client {
    type Result = ();

    fn handle(&mut self, msg: Server2Client, ctx: &mut Self::Context) {
        Client::send(msg, ctx);
    }
}
