import { Mapper } from "../util";
import { ProjectInfo, ProjectId, FileInfo, FileId, Doc } from "./types";

export type Lock = Mapper<{
  Unlock: {};
  LockBy: {};
  LockByMe: {};
}>;

export type Server2Client = Mapper<{
  Projects: {
    list: ProjectInfo[];
  };
  Project: {
    id: ProjectId;
    msg: Server2Client_Project;
  };
}>;

export type Server2Client_Project = Mapper<{
  Files: {
    list: FileInfo[];
  };
  UpdateInfo: {
    info: ProjectInfo;
  };
  File: {
    id: FileId;
    msg: Server2Client_Project_File;
  };
}>;

export type Server2Client_Project_File = Mapper<{
  FileLock: { lock: Lock };
  FileSource: { src: string };
  FileDoc: { doc: Doc };
}>;
