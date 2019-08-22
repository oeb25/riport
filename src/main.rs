#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct ProjectId(usize);

#[derive(Serialize)]
struct Project {
    path: PathBuf,
}

#[post("/projects")]
fn projects() -> Result<Json<Vec<String>>, std::io::Error> {
    let rd = fs::read_dir("./sample_data/")?;
    let rd = rd
        .map(|d| d.unwrap().file_name().into_string().unwrap())
        .collect();
    Ok(Json(rd))
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
fn files(id: String) -> std::io::Result<Json<Vec<DirEntry>>> {
    let de = DirEntry::build_from(&PathBuf::from("./sample_data/").join(id))?;
    match de {
        DirEntry::File { .. } => unimplemented!(),
        DirEntry::Folder { children, .. } => Ok(Json(children)),
    }
}

fn make_project(project_dir: &Path) -> std::io::Result<std::process::Output> {
    std::process::Command::new("gmake")
        .current_dir(project_dir)
        .output()
}

#[post("/projects/<id>/output")]
fn output(id: String) -> std::io::Result<String> {
    let project_dir = PathBuf::from("./sample_data/").join(id);
    make_project(&project_dir)?;
    fs::read_to_string(project_dir.join("out.html"))
}

#[derive(Deserialize, Debug)]
struct FileRequest {
    path: Vec<String>,
}

#[post("/projects/<id>/file", format = "json", data = "<req>")]
fn file(id: String, req: Json<FileRequest>) -> std::io::Result<String> {
    let project_dir = PathBuf::from("./sample_data/").join(id);
    let file_dir = req
        .path
        .iter()
        .fold(project_dir, |path, name| path.join(name));
    fs::read_to_string(file_dir)
}

#[derive(Deserialize, Debug)]
struct UpdateFileRequest {
    path: Vec<String>,
    contents: String,
}

#[post("/projects/<id>/updateFile", format = "json", data = "<req>")]
fn update_file(id: String, req: Json<UpdateFileRequest>) -> std::io::Result<()> {
    let project_dir = PathBuf::from("./sample_data/").join(id);
    let file_dir = req
        .path
        .iter()
        .fold(project_dir, |path, name| path.join(name));
    fs::write(file_dir, &req.contents)
}

fn main() {
    use rocket::config::{Config, Environment};

    rocket::custom(
        Config::build(Environment::Staging)
            .address("0.0.0.0")
            .unwrap(),
    )
    .mount("/api", routes![projects, files, file, output, update_file])
    .mount("/", rocket_contrib::serve::StaticFiles::from("./app/dist/"))
    .launch();
}
