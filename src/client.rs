use actix::fut::Either;
use actix::*;
use actix_web_actors::ws;

use std::collections::HashMap;

use crate::file::{EditFile, File, FileId, JoinFile, LeaveFile, ListenKind};
use crate::hub::{self, GetProject, Hub};
use crate::project::{GetFile, JoinProject, LeaveProject, Project, ProjectId, ReorderFile};

use crate::c2s::*;
use crate::s2c::*;

#[derive(Clone)]
pub struct FileCache {
    src_id: Option<usize>,
    doc_id: Option<usize>,
    file: Addr<File>,
}

impl FileCache {
    fn id_for(&self, kind: ListenKind) -> Option<usize> {
        match kind {
            ListenKind::Src => self.src_id,
            ListenKind::Doc => self.doc_id,
        }
    }
    fn set_id_for(&mut self, kind: ListenKind, v: Option<usize>) {
        match kind {
            ListenKind::Src => self.src_id = v,
            ListenKind::Doc => self.doc_id = v,
        }
    }
}

pub struct Client {
    id: usize,
    hub: Addr<Hub>,
    projects: HashMap<ProjectId, (Option<usize>, Addr<Project>)>,
    files: HashMap<(ProjectId, FileId), FileCache>,
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
        _: &mut ws::WebsocketContext<Self>,
    ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = (Option<usize>, Addr<Project>)>
    {
        if let Some(p) = self.projects.get(&id) {
            Either::A(fut::ok(p.clone()))
        } else {
            let f = self
                .hub
                .send(GetProject { id })
                .into_actor(self)
                .map(move |res, act, _| match res {
                    Some(project) => {
                        act.projects.insert(id, (None, project.clone()));
                        (None, project)
                    }
                    _ => unimplemented!(),
                })
                .map_err(|e, _, _| {
                    panic!("{:?}", e);
                });
            Either::B(f)
        }
    }
    pub fn get_file(
        &mut self,
        project_id: ProjectId,
        file_id: FileId,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> impl fut::ActorFuture<Actor = Self, Error = (), Item = FileCache> {
        if let Some(p) = self.files.get(&(project_id, file_id)) {
            Either::A(fut::ok(p.clone()))
        } else {
            let f = self
                .get_project(project_id, ctx)
                .then(move |res, act, _| match res {
                    Ok((_, project)) => project.send(GetFile { id: file_id }).into_actor(act),
                    _ => unimplemented!(),
                })
                .map(move |file, act, _| {
                    let file = file.expect("file not found");
                    let cache = FileCache {
                        src_id: None,
                        doc_id: None,
                        file: file.clone(),
                    };
                    act.files.insert((project_id, file_id), cache.clone());
                    cache
                })
                .map_err(|e, _, _| {
                    panic!("{:?}", e);
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
            .then(move |res, act, _| match res {
                Ok((id, project)) => {
                    if id.is_some() {
                        Either::A(fut::ok(()))
                    } else {
                        let f = project
                            .send(JoinProject { addr: my_adder })
                            .into_actor(act)
                            .map(move |id, act, _| {
                                act.projects.insert(project_id, (Some(id), project));
                            })
                            .map_err(|e, _, _| {
                                panic!("{:?}", e);
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
            .map(move |(con, project), act, _| {
                project.do_send(LeaveProject { id: con.unwrap() });
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
        self.get_file(project_id, file_id, ctx)
            .then(move |res, act, _| match res {
                Ok(cache) => {
                    if cache.id_for(kind).is_some() {
                        Either::A(fut::ok(()))
                    } else {
                        let f = cache
                            .file
                            .send(JoinFile {
                                addr: my_adder,
                                kind,
                            })
                            .into_actor(act)
                            .map(move |id, act, _| {
                                act.files.insert(
                                    (project_id, file_id),
                                    FileCache {
                                        src_id: if kind == ListenKind::Src {
                                            Some(id)
                                        } else {
                                            cache.id_for(ListenKind::Src)
                                        },
                                        doc_id: if kind == ListenKind::Doc {
                                            Some(id)
                                        } else {
                                            cache.id_for(ListenKind::Doc)
                                        },
                                        file: cache.file,
                                    },
                                );
                            })
                            .map_err(|e, _, _| {
                                panic!("{:?}", e);
                            });
                        Either::B(f)
                    }
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
        self.get_file(project_id, file_id, ctx)
            .map(move |cache, act, _| {
                cache.file.do_send(LeaveFile {
                    id: cache.id_for(kind).unwrap(),
                });
                let remove = {
                    let f = act.files.get_mut(&(project_id, file_id)).unwrap();
                    f.set_id_for(kind, None);
                    f.src_id.or(f.doc_id).is_none()
                };
                if remove {
                    act.files.remove(&(project_id, file_id));
                }
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
            .map(move |cache, act, _| {
                cache.file.do_send(EditFile {
                    src,
                    ignore: act
                        .files
                        .get(&(project_id, file_id))
                        .and_then(|cache| cache.src_id),
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
        self.get_project(project_id, ctx)
            .map(move |(_, project), _, _| {
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
        self.hub
            .send(hub::Connect {
                adder: ctx.address(),
            })
            .into_actor(self)
            .then(|res, act, _| {
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
                    Client2Server::CreateProject { .. } => {
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
                        Client2Server_Project::Reorder {
                            id: file_id,
                            new_index,
                        } => {
                            let f = self.reorder_file(project_id, file_id, new_index, ctx);
                            ctx.wait(f);
                        }
                        Client2Server_Project::File { id: file_id, msg } => match msg {
                            Client2Server_Project_File::JoinFileSource => {
                                let f = self.join_file(project_id, file_id, ListenKind::Src, ctx);
                                ctx.wait(f);
                            }
                            Client2Server_Project_File::LeaveFileSource => {
                                let f = self.leave_file(project_id, file_id, ListenKind::Src, ctx);
                                ctx.wait(f);
                            }
                            Client2Server_Project_File::JoinFileDoc => {
                                let f = self.join_file(project_id, file_id, ListenKind::Doc, ctx);
                                ctx.wait(f);
                            }
                            Client2Server_Project_File::LeaveFileDoc => {
                                let f = self.leave_file(project_id, file_id, ListenKind::Doc, ctx);
                                ctx.wait(f);
                            }
                            Client2Server_Project_File::EditFileSource { contents } => {
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
