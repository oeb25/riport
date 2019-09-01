use actix::Message;

use serde::Serialize;

use crate::file::{Doc, FileId, FileInfo};
use crate::project::{ProjectId, ProjectInfo};

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Lock {
    Unlock,
    LockBy {},
    LockByMe,
}

#[derive(Serialize, Message)]
#[serde(tag = "type")]
pub enum Server2Client {
    Projects {
        list: Vec<ProjectInfo>,
    },
    Project {
        id: ProjectId,
        msg: Server2Client_Project,
    },
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Server2Client_Project {
    Files {
        list: Vec<FileInfo>,
    },
    UpdateInfo {
        info: ProjectInfo,
    },
    File {
        id: FileId,
        msg: Server2Client_Project_File,
    },
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Server2Client_Project_File {
    FileLock { lock: Lock },
    FileSource { src: String },
    FileDoc { doc: Doc },
}
