use actix::Message;

use serde::Serialize;

use crate::project::file::{Doc, FileId, FileInfo};
use crate::project::{ProjectId, ProjectInfo};

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum Lock {
    Unlock,
    LockBy {},
    LockByMe,
}

#[derive(Serialize, Clone, Message)]
#[serde(tag = "type")]
pub enum Server2Client {
    Projects {
        list: Vec<ProjectInfo>,
    },
    Project {
        id: ProjectId,
        msg: Server2ClientProject,
    },
}

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum Server2ClientProject {
    Files {
        list: Vec<FileInfo>,
    },
    UpdateInfo {
        info: ProjectInfo,
    },
    File {
        id: FileId,
        msg: Server2ClientProjectFile,
    },
}

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum Server2ClientProjectFile {
    FileLock { lock: Lock },
    FileSource { src: String },
    FileDoc { doc: Doc },
}
