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
        msg: Client2ServerProject,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Client2ServerProject {
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
        msg: Client2ServerProjectFile,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Client2ServerProjectFile {
    JoinFileSource,
    LeaveFileSource,
    EditFileSource { contents: String },
    JoinFileDoc,
    LeaveFileDoc,
}
