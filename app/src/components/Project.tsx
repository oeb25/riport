import * as React from "react";
import { EditorId, ProjectId } from "../server";
import * as server from "../server";
import { Render } from "./Render";
import { Editor } from "./Editor";
import { FilePanel } from "./FilePanel";

function repeat(callback: () => Promise<any>, delay: number = 100) {
  const savedCallback = React.useRef<typeof callback>();

  React.useEffect(() => {
    savedCallback.current = callback;
  });

  React.useEffect(() => {
    let stop = false;

    const tick = async () => {
      if (stop) return;
      const before = Date.now();
      await savedCallback.current!();
      const delta = Date.now() - before;
      const wait = Math.max(delay - delta, 0);
      setTimeout(tick, wait);
    };

    tick();
    return () => {
      stop = true;
    };
  }, [delay]);
}

type CachedFile = {
  source: string;
  compiled: null | string;
  lock: null | server.Lock;
  last_change: null | "me" | server.Change;
};

type State = {
  index: server.ProjectIndex;
  selectedFile: null | server.FileId;
  files: { [fileName: string]: CachedFile };
};
type ActionMap = {
  SELECT_FILE: { fileId: server.FileId };
  UPDATE_FILE_INDEX: { fileId: server.FileId; index: server.FileIndex };
  SET_FILE_SOURCE: {
    fileId: server.FileId;
    source: string;
    change: "me" | server.Change;
  };
  SET_FILE_COMPILED: {
    fileId: server.FileId;
    compiled: string;
  };
  INPUT_SOURCE: { fileId: server.FileId; source: string };
};
type Action = {
  [K in keyof ActionMap]: { type: K } & ActionMap[K];
}[keyof ActionMap];

const initalState: State = {
  index: { files: {}, order: [] },
  files: {},
  selectedFile: null
};

const updateState = (state: State, action: Action): State => {
  switch (action.type) {
    case "SELECT_FILE": {
      return { ...state, selectedFile: action.fileId };
    }
    case "UPDATE_FILE_INDEX": {
      return {
        ...state,
        index: {
          ...state.index,
          files: { ...state.index.files, [action.fileId.path]: action.index }
        }
      };
    }
    case "SET_FILE_SOURCE": {
      const path = action.fileId.path;
      const old: CachedFile =
        path in state.files
          ? state.files[path]
          : {
              source: action.source,
              compiled: null,
              lock: null,
              last_change: action.change
            };

      return {
        ...state,
        files: {
          ...state.files,
          [path]: { ...old, source: action.source }
        }
      };
    }
    case "SET_FILE_COMPILED": {
      const path = action.fileId.path;
      const old: CachedFile =
        path in state.files
          ? state.files[path]
          : {
              source: "",
              compiled: action.compiled,
              lock: null,
              last_change: null
            };

      return {
        ...state,
        files: {
          ...state.files,
          [path]: { ...old, compiled: action.compiled }
        }
      };
    }
    case "INPUT_SOURCE": {
      const path = action.fileId.path;
      if (!(path in state.files)) return state;

      const old = state.files[path];

      return {
        ...state,
        files: {
          ...state.files,
          [path]: {
            ...old,
            source: action.source,
            last_change: "me"
          }
        }
      };
    }
  }
};

const performDelta = (
  projectId: ProjectId,
  dispatch: React.Dispatch<Action>,
  delta: server.ProjectIndexDelta
) => {
  if (
    delta.changed_compiled_files
      .concat(delta.changed_source_files)
      .concat(delta.new_files)
      .concat(delta.removed_files).length > 0
  ) {
    console.log(JSON.stringify(delta));
  }

  delta.new_files.map(n => {
    dispatch({ type: "UPDATE_FILE_INDEX", fileId: n.id, index: n.index });
    server.fileSrc(projectId, n.id).then(src => {
      dispatch({
        type: "SET_FILE_SOURCE",
        fileId: n.id,
        source: src,
        change: n.index.last_source_change!
      });
    });
    server.fileCompiled(projectId, n.id).then(src => {
      dispatch({
        type: "SET_FILE_COMPILED",
        fileId: n.id,
        compiled: src
      });
    });
  });
  delta.changed_source_files.map(n => {
    dispatch({ type: "UPDATE_FILE_INDEX", fileId: n.id, index: n.index });
    server.fileSrc(projectId, n.id).then(source => {
      dispatch({
        type: "SET_FILE_SOURCE",
        fileId: n.id,
        source,
        change: n.index.last_source_change!
      });
    });
  });
  delta.changed_compiled_files.map(n => {
    dispatch({ type: "UPDATE_FILE_INDEX", fileId: n.id, index: n.index });
    server.fileCompiled(projectId, n.id).then(compiled => {
      dispatch({
        type: "SET_FILE_COMPILED",
        fileId: n.id,
        compiled
      });
    });
  });
};

export const Project: React.SFC<{
  editorId: EditorId;
  projectId: ProjectId;
}> = ({ editorId, projectId }) => {
  const [state, dispatch] = React.useReducer(updateState, initalState);

  repeat(async () => {
    if (!state) return;

    const delta = await server.indexDelta(projectId, {
      editor: editorId,
      index: state.index
    });
    performDelta(projectId, dispatch, delta);
  }, 200);

  if (!state) return <div className="flex flex-1">Loading...</div>;

  return (
    <div className="flex flex-1 flex-col">
      <div className="flex">
        <Header projectId={projectId} />
      </div>
      <div className="flex flex-1">
        <div className="border flex">
          <FilePanel
            newFile={async () => {
              const fileId = await server.newFile(projectId, {
                name: `untitled-${Math.floor(Math.random() * 1000)}.md`
              });
              dispatch({ type: "SELECT_FILE", fileId: fileId });
            }}
            selectFile={fileId => dispatch({ type: "SELECT_FILE", fileId })}
            files={state.index.files}
            selectedFile={state.selectedFile}
          ></FilePanel>
        </div>
        <div className="border flex flex-1">
          <Editor
            height={400}
            width={400}
            locked={false}
            onChange={value => {
              if (state.selectedFile) {
                server.editSrc(projectId, {
                  file_id: state.selectedFile,
                  editor: editorId,
                  value
                });
                dispatch({
                  type: "INPUT_SOURCE",
                  fileId: state.selectedFile,
                  source: value
                });
              }
            }}
            value={
              state.selectedFile && state.files[state.selectedFile.path]
                ? state.files[state.selectedFile.path].source
                : ""
            }
          />
        </div>
        <div className="border flex flex-1 markdown p-5">
          <Render
            projectId={projectId}
            src={Object.keys(state.files)
              .map(f => {
                const file = state.files[f];
                if (file.compiled) {
                  return JSON.parse(file.compiled).blocks;
                } else {
                  return [];
                }
              })
              .reduce((acc, f) => [...acc, f], [])}
          ></Render>
        </div>
      </div>
    </div>
  );
};

const Header: React.SFC<{ projectId: ProjectId }> = ({ projectId }) => (
  <div className="flex p-2">
    <a
      className="px-2"
      href="#"
      onClick={e => {
        e.preventDefault();
      }}
    >
      Projects
    </a>
    <span>/</span>
    <a
      className="px-2"
      href="#"
      onClick={e => {
        e.preventDefault();
      }}
    >
      {projectId.project_id}
    </a>
  </div>
);
