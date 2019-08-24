#![feature(proc_macro_hygiene, decl_macro)]
#![feature(bind_by_move_pattern_guards)]

#[macro_use]
extern crate rocket;

mod project;
mod routes;
mod state;
mod walk_pandoc;

use rocket::State;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct File {
    path: PathBuf,
    src: String,
    compiled: String,
    src_hash: String,
    compiled_hash: String,
}

fn hash(src: &str) -> String {
    // use sha2::Digest;
    // let mut hasher = sha2::Sha512::new();
    // hasher.input(std::ffi::CString::new(src).unwrap().as_bytes());
    // base64::encode(&hasher.result())
    src.to_owned()
}

fn pandoc_compile(cwd: &Path, src: &str) -> io::Result<String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut cmd = Command::new("pandoc")
        .args(&["-f", "markdown", "-t", "json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    write!(cmd.stdin.as_mut().expect("failed to get stdin"), "{}", src)?;
    let out = cmd.wait_with_output()?;
    let compiled = String::from_utf8_lossy(&out.stdout).to_string();
    Ok(compiled)
}

impl File {
    fn read(project_root: &Path, path: PathBuf) -> io::Result<File> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let src = fs::read_to_string(&path)?;
        let compiled = pandoc_compile(project_root, &src)?;

        let src_hash = hash(&src);
        let compiled_hash = hash(&compiled);

        Ok(File {
            path,
            src,
            compiled,
            src_hash,
            compiled_hash,
        })
    }
    fn update_src(&mut self, project_root: &Path, src: String) {
        println!("UPDATED SRC");

        self.src = src;
        self.src_hash = hash(&self.src);
        let compiled = pandoc_compile(project_root, &self.src).expect("failed to compile");
        self.update_compiled(compiled);
    }
    fn update_compiled(&mut self, compiled: String) {
        self.compiled = compiled;
        self.src_hash = hash(&self.compiled);
    }
}

#[derive(Debug)]
pub struct Project {
    path: PathBuf,
    files: HashMap<PathBuf, File>,
    order: Vec<PathBuf>,
}

impl Project {
    fn open(path: PathBuf) -> io::Result<Project> {
        let src_dir = fs::read_dir(path.join("src"))?;
        let mut files = HashMap::new();
        for s in src_dir {
            let s = s?;
            let file = File::read(&path, s.path())?;
            files.insert(s.path(), file);
        }
        Ok(Project {
            path,
            order: files.keys().cloned().collect(),
            files,
        })
    }
    fn create_index(&self) -> ProjectIndex {
        ProjectIndex {
            order: self.order.clone(),
            files: self
                .files
                .iter()
                .map(|(path, file)| {
                    (
                        path.clone(),
                        FileIndex {
                            path: path.clone(),
                            src_hash: file.src_hash.clone(),
                            compiled_hash: file.compiled_hash.clone(),
                        },
                    )
                })
                .collect(),
        }
    }
}

mod project_state {
    use super::Project;
    use std::collections::HashMap;
    use std::fs;
    use std::io;
    use std::path::PathBuf;
    use std::sync::{Mutex, MutexGuard};

    pub struct ProjectState {
        pub root: PathBuf,
        projects: Mutex<HashMap<String, Project>>,
    }

    impl ProjectState {
        pub fn new(root: PathBuf) -> io::Result<ProjectState> {
            let mut projects = HashMap::new();
            let rd = fs::read_dir("./sample_data/")?;
            for d in rd {
                let d = d?;
                let path = d.path();
                let name = path
                    .file_name()
                    .unwrap()
                    .to_os_string()
                    .into_string()
                    .unwrap();
                let project = Project::open(path.clone())?;
                projects.insert(name, project);
            }
            Ok(ProjectState {
                root,
                projects: Mutex::new(projects),
            })
        }
        pub fn with_project<T>(&self, id: &str, f: impl FnOnce(&Project) -> T) -> T {
            f(self.projects.lock().unwrap().get(id).unwrap())
        }
        pub fn with_project_mut<T>(&self, id: &str, f: impl FnOnce(&mut Project) -> T) -> T {
            f(self.projects.lock().unwrap().get_mut(id).unwrap())
        }
        pub fn list_projects(&self) -> Vec<String> {
            self.projects.lock().unwrap().keys().cloned().collect()
        }
    }
}

use crate::project_state::*;

#[post("/projects")]
fn projects(projects: State<ProjectState>) -> Result<Json<Vec<String>>, std::io::Error> {
    // let rd = fs::read_dir("./sample_data/")?;
    // let rd = rd
    //     .map(|d| d.unwrap().file_name().into_string().unwrap())
    //     .collect();
    // Ok(Json(rd))

    Ok(Json(projects.list_projects()))
}

#[derive(Serialize)]
enum DirEntry {
    File {
        name: String,
    },
    Folder {
        name: String,
        children: Vec<DirEntry>,
    },
}

impl DirEntry {
    fn build_from(path: &Path) -> std::io::Result<DirEntry> {
        let name = path
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        let md = fs::metadata(path)?;
        if md.is_dir() {
            let children: std::io::Result<Vec<_>> = fs::read_dir(path)?
                .map(|d| DirEntry::build_from(&d.unwrap().path()))
                .collect();
            Ok(DirEntry::Folder {
                name,
                children: children?,
            })
        } else {
            Ok(DirEntry::File { name })
        }
    }
}

#[post("/projects/<id>/files")]
fn files(id: String, projects: State<ProjectState>) -> std::io::Result<Json<Vec<DirEntry>>> {
    let files = projects.with_project(&id, |project| {
        project
            .files
            .keys()
            .map(|path| {
                let name = path
                    .file_name()
                    .unwrap()
                    .to_os_string()
                    .into_string()
                    .unwrap();
                DirEntry::File { name }
            })
            .collect()
    });

    Ok(Json(files))

    // let de = DirEntry::build_from(&PathBuf::from("./sample_data/").join(id).join("src"))?;
    // match de {
    //     DirEntry::File { .. } => unimplemented!(),
    //     DirEntry::Folder { children, .. } => Ok(Json(children)),
    // }
}

fn make_project(project_dir: &Path) -> std::io::Result<std::process::Output> {
    std::process::Command::new("gmake")
        .current_dir(project_dir)
        .output()
}

#[derive(Serialize, Debug)]
struct FileIndex {
    path: PathBuf,
    src_hash: String,
    compiled_hash: String,
}

#[derive(Serialize, Debug)]
struct ProjectIndex {
    order: Vec<PathBuf>,
    files: HashMap<PathBuf, FileIndex>,
}

#[post("/projects/<id>/index")]
fn project_index(id: String, projects: State<ProjectState>) -> Json<ProjectIndex> {
    Json(projects.with_project(&id, |p| p.create_index()))
}

#[post("/projects/<id>/output")]
fn output(id: String, projects: State<ProjectState>) -> std::io::Result<String> {
    unimplemented!()
    // let project = projects.with_project(&id, |p|).unwrap();

    // Ok(project
    //     .files
    //     .values()
    //     .take(1)
    //     .map(|x| x.compiled.to_string())
    //     .collect::<Vec<_>>()
    //     .join("\n"))

    // let project_dir = PathBuf::from("./sample_data/").join(id);
    // make_project(&project_dir)?;
    // fs::read_to_string(project_dir.join("out.json"))
}

#[derive(Deserialize, Debug)]
struct FileRequest {
    path: String,
}

#[derive(Serialize)]
struct FileResponse {
    src: String,
    compiled: String,
}

#[post("/projects/<id>/file", format = "json", data = "<req>")]
fn file(
    id: String,
    req: Json<FileRequest>,
    projects: State<ProjectState>,
) -> Option<Json<FileResponse>> {
    let res = projects.with_project(&id, |p| {
        // TODO: Base path
        let file = p.files.get(&PathBuf::from(&req.path))?;
        Ok(FileResponse {
            src: file.src.to_string(),
            compiled: file.compiled.to_string(),
        })
    })?;

    Some(Json(res))

    // let dir = PathBuf::from(req.path);
    // fs::read_to_string(dir)
}

#[derive(Deserialize, Debug)]
struct UpdateFileRequest {
    path: String,
    contents: String,
}

#[post("/projects/<id>/updateFile", format = "json", data = "<req>")]
fn update_file(
    id: String,
    req: Json<UpdateFileRequest>,
    projects: State<ProjectState>,
) -> std::io::Result<()> {
    projects.with_project_mut(&id, |p| {
        // TODO: Base path
        let file = p.files.get_mut(&PathBuf::from(&req.path)).unwrap();
        file.update_src(&p.path, req.contents.to_owned());
    });

    Ok(())

    // Some(Json(FileResponse {
    //     src: file.src.to_string(),
    //     compiled: file.compiled.to_string(),
    // }))

    // let project_dir = PathBuf::from("./sample_data").join(id).join("src");
    // let file_dir = req
    //     .path
    //     .iter()
    //     .fold(project_dir, |path, name| path.join(name));
    // fs::write(file_dir, &req.contents)
}

#[get("/projects/<id>/static/<path..>")]
fn static_files<'r>(
    id: String,
    path: PathBuf,
) -> Result<rocket::response::NamedFile, rocket::http::Status> {
    use rocket::handler::Outcome;
    use rocket::http::Status;

    let project_dir = PathBuf::from("./sample_data/").join(id);
    let path = project_dir.join(path);

    if path.is_dir() {
        return Err(Status::NotFound);
    }

    rocket::response::NamedFile::open(&path).map_err(|_| Status::NotFound)
}

use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};

fn cors_options() -> CorsOptions {
    Default::default()
}

fn main() {
    use rocket::config::{Config, Environment};

    println!("{}", hash("Hello world!"));

    rocket::custom(
        Config::build(Environment::Staging)
            .address("0.0.0.0")
            .unwrap(),
    )
    // .manage(
    //     ProjectState::new(PathBuf::from("./sample_data/1234/"))
    //         .expect("failed to start project state"),
    // )
    // .mount(
    //     "/api",
    //     routes![
    //         projects,
    //         files,
    //         file,
    //         output,
    //         update_file,
    //         project_index,
    //         static_files
    //     ],
    // )
    .manage(state::ProjectsState::new(PathBuf::from("./new_data/")))
    .mount(
        "/api2",
        routes![
            routes::get_editor_id,
            routes::projects,
            routes::new_project,
            routes::project_routes::new_file,
            routes::project_routes::index,
            routes::project_routes::index_delta,
            routes::project_routes::file_src,
            routes::project_routes::file_compiled,
            routes::project_routes::edit_src,
            routes::project_routes::static_files,
        ],
    )
    .mount("/", rocket_contrib::serve::StaticFiles::from("./app/dist/"))
    // .mount("/", rocket_cors::catch_all_options_routes())
    .attach(cors_options().to_cors().expect("To not fail"))
    .launch();
}
