import {
  ProjectId,
  ProjectInfo,
  FileInfo,
  ProjectFiles,
  FileId
} from "./com/types";
import { Server2Client } from "./com/s2c";

export type State = {
  route: Routes;
  projects: ProjectInfo[];
  projectFileInfos: { [project_id: number]: FileInfo[] };
  projectFiles: { [project_id: number]: ProjectFiles };
};

export type PathSegment = { name: string; route: Routes };

export type Routes =
  | {
      name: "landing";
    }
  | {
      name: "project";
      id: ProjectId;
      file?: FileId;
    };

export const initialState: State = {
  route: { name: "landing" },
  projects: [],
  projectFileInfos: {},
  projectFiles: {}
};

export type Action =
  | { type: "Server"; msg: Server2Client }
  | { type: "SetRoute"; route: Routes };

export const reducer: React.Reducer<State, Action> = (state, action) => {
  switch (action.type) {
    case "SetRoute": {
      state = { ...state, route: action.route };
      break;
    }
    case "Server": {
      return handleServerMsg(state, action.msg);
    }
  }

  return state;
};

const handleServerMsg = (state: State, msg: Server2Client): State => {
  switch (msg.type) {
    case "Projects": {
      return { ...state, projects: msg.list };
    }
    case "Project": {
      const { id, msg: msg2 } = msg;
      switch (msg2.type) {
        case "Files": {
          return {
            ...state,
            projectFileInfos: {
              ...state.projectFileInfos,
              [id.project_id]: msg2.list
            },
            projectFiles: {
              ...state.projectFiles,
              [id.project_id]: msg2.list.reduce(
                (acc, f) => {
                  acc[f.id.file_id] = {
                    doc: null,
                    id: f.id,
                    name: f.name,
                    src: ""
                  };
                  return acc;
                },
                {} as ProjectFiles
              )
            }
          };
        }
        case "File": {
          const { id: fileId, msg: msg3 } = msg2;
          switch (msg3.type) {
            case "FileSource": {
              const projectFiles = state.projectFiles[id.project_id] || {};
              const f = projectFiles[fileId.file_id] || {
                doc: null,
                id: fileId,
                name: "idk",
                src: msg3.src
              };

              return {
                ...state,
                projectFiles: {
                  ...state.projectFiles,
                  [id.project_id]: {
                    ...projectFiles,
                    [fileId.file_id]: {
                      ...f,
                      src: msg3.src
                    }
                  }
                }
              };
            }
            case "FileDoc": {
              const projectFiles = state.projectFiles[id.project_id] || {};
              const f = projectFiles[fileId.file_id] || {
                doc: msg3.doc,
                id: fileId,
                name: "idk",
                src: ""
              };

              return {
                ...state,
                projectFiles: {
                  ...state.projectFiles,
                  [id.project_id]: {
                    ...projectFiles,
                    [fileId.file_id]: {
                      ...f,
                      doc: msg3.doc
                    }
                  }
                }
              };
            }
            default: {
              console.log("unhandled file", msg);
              return state;
            }
          }
        }
        default: {
          console.log("unhandled project", msg);
          return state;
        }
      }
    }
    default: {
      console.log("unhandled", msg);
      return state;
    }
  }
};

export const findProjectInfo = (state: State, id: ProjectId): ProjectInfo =>
  state.projects.filter(info => info.id.project_id == id.project_id)[0];

export const getFileName = (
  c: State | ProjectFiles | undefined,
  projectId: ProjectId,
  fileId: FileId
): string | null => {
  if (!c) return null;

  const pfs = "route" in c ? c.projectFiles[projectId.project_id] : c;

  // if (projectId.project_id in projectFiles) {
  // const pfs = projectFiles[projectId.project_id];
  if (fileId.file_id in pfs) {
    return pfs[fileId.file_id].name;
  }
  // }
  return null;
};

export const buildPath = (state: State): PathSegment[] => {
  const path: PathSegment[] = [
    {
      name: "Riport",
      route: { name: "landing" }
    }
  ];

  const { route } = state;

  if (route.name == "project") {
    const info = findProjectInfo(state, route.id);
    path.push({
      name: info.name,
      route: {
        name: "project",
        id: info.id
      }
    });
    if (route.file) {
      path.push({
        name: getFileName(state, route.id, route.file) || "???",
        route: {
          name: "project",
          id: info.id,
          file: route.file
        }
      });
    }
  }
  return path;
};
