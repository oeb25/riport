use std::path::PathBuf;
use std::time;

use rocket::State;
use rocket_contrib::json::Json;
use serde::Deserialize;

use crate::project::{EditorId, FileId, ProjectId, ProjectIndex, ProjectIndexDelta};
use crate::state::ProjectsState;

type S<'a> = State<'a, ProjectsState>;
// TODO: Proper error
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[post("/get-editor-id")]
pub fn get_editor_id(state: S) -> Json<EditorId> {
    Json(state.new_editor())
}

#[post("/new-project")]
pub fn new_project(state: S) -> Result<Json<ProjectId>> {
    Ok(Json(state.init_project()?))
}

#[post("/projects")]
pub fn projects(state: S) -> Result<Json<Vec<ProjectId>>> {
    Ok(Json(state.list_projects()))
}

pub mod project_routes {
    use super::*;

    #[derive(Deserialize)]
    pub struct NewFile {
        name: String,
    }

    #[post("/<id>/new-file", data = "<data>")]
    pub fn new_file(id: usize, state: S, data: Json<NewFile>) -> Result<Json<FileId>> {
        let projcet = state.get_project(ProjectId { project_id: id }).unwrap();
        let mut project = projcet.lock().expect("failed to lock project");
        Ok(Json(project.new_file(&data.name)))
    }

    #[post("/<id>/index")]
    pub fn index(id: usize, state: S) -> Result<Json<ProjectIndex>> {
        let projcet = state.get_project(ProjectId { project_id: id }).unwrap();
        let project = projcet.lock().expect("failed to lock project");
        Ok(Json(project.index()))
    }

    #[derive(Deserialize)]
    pub struct IndexDeltaRequest {
        index: ProjectIndex,
        editor: EditorId,
    }

    #[post("/<id>/index-delta", data = "<data>")]
    pub fn index_delta(
        id: usize,
        state: S,
        data: Json<IndexDeltaRequest>,
    ) -> Result<Json<ProjectIndexDelta>> {
        let projcet = state.get_project(ProjectId { project_id: id }).unwrap();
        let project = projcet.lock().expect("failed to lock project");
        Ok(Json(project.index_delta(data.0.editor, data.0.index)))
    }

    #[post("/<id>/file-src", data = "<data>")]
    pub fn file_src(id: usize, state: S, data: Json<FileId>) -> Result<Json<String>> {
        let projcet = state.get_project(ProjectId { project_id: id }).unwrap();
        let project = projcet.lock().expect("failed to lock project");
        Ok(Json(project.get_source(&data.0)?.to_string()))
    }

    #[post("/<id>/file-compiled", data = "<data>")]
    pub fn file_compiled(id: usize, state: S, data: Json<FileId>) -> Result<Json<String>> {
        let projcet = state.get_project(ProjectId { project_id: id }).unwrap();
        let project = projcet.lock().expect("failed to lock project");
        Ok(Json(project.get_compiled(&data.0)?.to_string()))
    }

    #[derive(Deserialize)]
    pub struct EditSrc {
        file_id: FileId,
        editor: EditorId,
        value: String,
    }

    #[post("/<id>/edit-src", data = "<data>")]
    pub fn edit_src(id: usize, state: S, data: Json<EditSrc>) -> Result<Json<time::SystemTime>> {
        let projcet = state.get_project(ProjectId { project_id: id }).unwrap();
        let mut project = projcet.lock().expect("failed to lock project");
        let edit = data.0;
        let time = project.edit_file(edit.editor, &edit.file_id, &edit.value)?;
        Ok(Json(time))
    }

    #[get("/projects/<id>/static/<path..>")]
    pub fn static_files<'r>(
        id: usize,
        path: PathBuf,
        state: S,
    ) -> std::result::Result<rocket::response::NamedFile, rocket::http::Status> {
        let projcet = state.get_project(ProjectId { project_id: id }).unwrap();
        let project = projcet.lock().expect("failed to lock project");

        let path = project.static_file_path(&path);

        if path.is_dir() {
            return Err(rocket::http::Status::NotFound);
        }

        rocket::response::NamedFile::open(&path).map_err(|_| rocket::http::Status::NotFound)
    }
}
