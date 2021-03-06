use actix::*;
use futures::future::join_all;
use futures::prelude::*;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::client::{Client, ClientId};
use crate::project::file::FileId;
use crate::project::{Project, ProjectId, ProjectInfo};
use crate::project_actor::{GetInfo, ProjectActor};

use crate::s2c::*;

pub struct Hub {
    projects_path: PathBuf,
    connections: HashMap<ClientId, Addr<Client>>,
    projects: HashMap<ProjectId, Addr<ProjectActor>>,
    tmpdir: tempdir::TempDir,
}

impl Hub {
    pub fn new(projects_path: PathBuf) -> io::Result<Hub> {
        let mut hub = Hub {
            projects_path,
            connections: HashMap::new(),
            projects: HashMap::new(),
            tmpdir: tempdir::TempDir::new("hub").unwrap(),
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

        join_all(projects).map(|project_infos| {
            project_infos
                .into_iter()
                .map(|project_info| project_info.0)
                .collect()
        })
    }
    fn create_project(&mut self, name: String) -> (ProjectId, Addr<ProjectActor>) {
        let id = ProjectId {
            project_id: self.projects.len() as _,
        };
        let path = self.projects_path.join(&name);
        let project_tmpdir = self.tmpdir.path().join(&format!("{}", id.project_id));
        fs::create_dir_all(&project_tmpdir).unwrap();
        let project = ProjectActor::new(id, name, path, project_tmpdir);
        self.projects.insert(id, project.clone());

        (id, project)
    }
    fn load_project(&mut self, name: String, path: PathBuf) -> (ProjectId, Addr<ProjectActor>) {
        println!("loading project at {:?}", path);
        let id = ProjectId {
            project_id: self.projects.len() as _,
        };
        let project_tmpdir = self.tmpdir.path().join(&format!("{}", id.project_id));
        fs::create_dir_all(&project_tmpdir).unwrap();
        let project = ProjectActor::read_from_disk(path.to_owned(), project_tmpdir);
        self.projects.insert(id, project.clone());

        (id, project)
    }
}

impl Actor for Hub {
    type Context = Context<Self>;
}

pub struct Connect {
    pub client_id: ClientId,
    pub adder: Addr<Client>,
}

impl Message for Connect {
    type Result = ();
}

impl Handler<Connect> for Hub {
    type Result = ();
    fn handle(&mut self, msg: Connect, ctx: &mut Context<Self>) {
        self.connections.insert(msg.client_id, msg.adder.clone());

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
    type Result = Option<Addr<ProjectActor>>;
}

impl Handler<GetProject> for Hub {
    type Result = Option<Addr<ProjectActor>>;
    fn handle(&mut self, msg: GetProject, _: &mut Context<Self>) -> Option<Addr<ProjectActor>> {
        self.projects.get(&msg.id).cloned()
    }
}

#[derive(Debug)]
pub struct GetCompileArtifactDir {
    pub project_id: ProjectId,
    pub file_id: FileId,
}

impl Message for GetCompileArtifactDir {
    type Result = CompileArtifactDir;
}

#[derive(MessageResponse)]
pub struct CompileArtifactDir {
    pub path: PathBuf,
}

impl Handler<GetCompileArtifactDir> for Hub {
    type Result = CompileArtifactDir;
    fn handle(&mut self, msg: GetCompileArtifactDir, _: &mut Context<Self>) -> CompileArtifactDir {
        let path = self.tmpdir.path().join(format!(
            "{}/{}",
            msg.project_id.project_id, msg.file_id.file_id
        ));
        CompileArtifactDir { path }
    }
}
