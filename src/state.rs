use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use crate::project::{EditorId, Project, ProjectId};

pub struct ProjectsState {
    root: PathBuf,
    editor_count: AtomicUsize,
    projects: Mutex<HashMap<ProjectId, Arc<Mutex<Project>>>>,
}

impl ProjectsState {
    pub fn new(root: PathBuf) -> ProjectsState {
        let state = ProjectsState {
            root,
            editor_count: AtomicUsize::new(0),
            projects: Mutex::new(HashMap::new()),
        };
        let pid = state.init_project().unwrap();
        let project = state.get_project(pid).unwrap();
        let mut project = project.lock().unwrap();
        project.new_file("index.md");
        state
    }
    pub fn new_editor(&self) -> EditorId {
        let editor_id = self.editor_count.fetch_add(1, Ordering::SeqCst);
        EditorId { editor_id }
    }
    pub fn get_project(&self, id: ProjectId) -> Option<Arc<Mutex<Project>>> {
        let projects = self.projects.lock().expect("failed to lock projects");
        Some(Arc::clone(projects.get(&id)?))
    }
    pub fn init_project(&self) -> io::Result<ProjectId> {
        let mut projects = self.projects.lock().unwrap();
        let id = ProjectId {
            project_id: projects.len(),
        };
        let project_root = self.root.join(format!("{}", id.project_id));
        let project = Project::init(id, project_root)?;

        projects.insert(id, Arc::new(Mutex::new(project)));

        Ok(id)
    }
    pub fn list_projects(&self) -> Vec<ProjectId> {
        self.projects.lock().unwrap().keys().cloned().collect()
    }
}

// #[test]
// fn test_get_project_mut() -> Result<(), Box<dyn std::error::Error>> {
//     let state = ProjectsState::new(PathBuf::from("./sample_data/test"));
//     let editor_a = state.new_editor();
//     let editor_b = state.new_editor();
//     assert!(editor_a != editor_b);
//     let id = state.init_project()?;
//     let file_id = {
//         let project = state.get_project(id).unwrap();
//         let mut project = project.lock().unwrap();
//         project.new_file("test")
//     };
//     {
//         let project = state.get_project(id).unwrap();
//         let mut project = project.lock().unwrap();
//         let list = project.list_files();
//         assert_eq!(list, vec![file_id.clone()]);
//         project.edit_file(editor_a, &file_id, "# Hello, world!")?;
//     }
//     {
//         let project = state.get_project(id).unwrap();
//         let mut project = project.lock().unwrap();
//         let source = project.get_source(&file_id);
//         assert_eq!(source, Ok("# Hello, world!"));
//     }
//     {
//         let project = state.get_project(id).unwrap();
//         let mut project = project.lock().unwrap();
//         let compiled = project.get_compiled(&file_id);
//         assert_eq!(compiled, Err(EditError::FileNotCompiled));
//         project.compile()?;
//         let compiled = project.get_compiled(&file_id);
//         assert_eq!(compiled.map(|x| x.trim()), Ok(r#"{"blocks":[{"t":"Header","c":[1,["hello-world",[],[]],[{"t":"Str","c":"Hello,"},{"t":"Space"},{"t":"Str","c":"world!"}]]}],"pandoc-api-version":[1,17,5,4],"meta":{}}"#));
//     }
//     {
//         let project = state.get_project(id).unwrap();
//         let mut project = project.lock().unwrap();
//         let list = project.list_files();
//         assert_eq!(list, vec![file_id.clone()]);
//         let edit_res = project.edit_file(editor_b, &file_id, "Interupt");
//         let err = edit_res.unwrap_err();
//         let lock = err.locked().unwrap();
//     }
//     {
//         let project = state.get_project(id).unwrap();
//         let mut project = project.lock().unwrap();
//         let source = project.get_source(&file_id);
//         assert_eq!(source, Ok("# Hello, world!"));
//     }
//     Ok(())
// }
