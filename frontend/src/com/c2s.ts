import { Mapper } from "../util";
import { ProjectId, FileId } from "./types";

export type Client2Server = Mapper<{
  CreateProject: {
    project_name: string;
  };
  Project: {
    id: ProjectId;
    msg: Client2Server_Project;
  };
}>;

export type Client2Server_Project = Mapper<{
  JoinProject: {};
  LeaveProject: {};
  CreateFile: {
    file_name: string;
  };
  File: {
    id: FileId;
    msg: Client2Server_Project_File;
  };
}>;

export type Client2Server_Project_File = Mapper<{
  JoinFileSource: {};
  LeaveFileSource: {};
  EditFileSource: { contents: string };
  JoinFileDoc: {};
  LeaveFileDoc: {};
}>;
