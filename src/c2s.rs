use serde::Deserialize;

use crate::file::FileId;
use crate::project::ProjectId;

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
    Reorder {
        id: FileId,
        new_index: usize,
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
