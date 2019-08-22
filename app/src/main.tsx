import * as React from "react";
import * as ReactDOM from "react-dom";
import ReactMonaco from "react-monaco-editor";
import Monaco from "monaco-editor";

import "./editorSetup";
import * as api from "./api";
import { Document, Render } from "./pandoc";

const getPromise = function<T>(
  data: null | (() => Promise<T>),
  deps: any[] = []
): null | T {
  const [d, set] = React.useState<null | T>(null);
  React.useEffect(() => {
    data && data().then(set);
    return () => {};
  }, deps);
  return d;
};

const App: React.SFC = () => {
  const projects = getPromise(() => api.getProjects());

  return (
    <div className="text-gray-100">
      {projects && <Project projectId={projects[0]} />}
    </div>
  );
};

function useInterval(callback: () => any, delay: number) {
  const savedCallback = React.useRef<typeof callback>();

  React.useEffect(() => {
    savedCallback.current = callback;
  });

  React.useEffect(() => {
    function tick() {
      savedCallback.current!();
    }

    let id = setInterval(tick, delay);
    return () => clearInterval(id);
  }, [delay]);
}

const Project: React.SFC<{ projectId: string }> = ({ projectId }) => {
  const files = getPromise(() => api.getFiles({ projectId }));
  const [selectedFile, setSelectedFile] = React.useState<null | string[]>(null);
  const [contents, setContents] = React.useState("");
  const [updateInterval, setUpdateInterval] = React.useState(500);
  useInterval(() => {
    if (selectedFile && updateInterval < 10000) {
      api.getFile({ projectId, path: selectedFile }).then(v => setContents(v));
    }
  }, updateInterval);
  const [output, setOutput] = React.useState<Document>({ blocks: [] });
  useInterval(() => {
    api.getOutput({ projectId }).then(output => setOutput(output));
  }, 500);
  const [updateTimeout, setUpdateTimeout] = React.useState(0);
  const [updateTimeout2, setUpdateTimeout2] = React.useState(0);

  return (
    <div className="flex flex-row">
      {files && (
        <FileBrowser
          files={files}
          selectedItem={selectedFile || []}
          selectItem={path => setSelectedFile(path)}
        />
      )}
      <div className="flex flex-1">
        <Editor
          selectedFile={selectedFile || []}
          contents={contents}
          onChange={value => {
            if (selectedFile) {
              setContents(value);

              setUpdateInterval(10000000);
              clearTimeout(updateTimeout);
              setUpdateTimeout(
                setTimeout(() => {
                  api.updateFile({
                    projectId,
                    path: selectedFile,
                    contents: value
                  });
                }, 200)
              );
              clearTimeout(updateTimeout2);
              setUpdateTimeout2(
                setTimeout(() => {
                  setUpdateInterval(500);
                }, 1000)
              );
            }
          }}
        />
      </div>
      <div className="flex flex-1 p-5 markdown">
        <Render src={output.blocks} />
      </div>
    </div>
  );
};

const Editor: React.SFC<{
  selectedFile: string[];
  contents: string;
  onChange: (value: string) => any;
}> = ({ selectedFile, contents, onChange }) => {
  const [
    editor,
    setEditor
  ] = React.useState<null | Monaco.editor.IStandaloneCodeEditor>(null);
  const [windowHeight, setWindowHeight] = React.useState(window.innerHeight);
  React.useEffect(() => {
    const listener = () => {
      if (editor) {
        editor.layout({
          width: window.innerWidth / 2,
          height: windowHeight
        });
      }
      setWindowHeight(window.innerHeight);
    };
    window.addEventListener("resize", listener);
    return () => {
      console.log("remove listener");
      window.removeEventListener("resize", listener);
    };
  }, [editor]);

  return (
    <ReactMonaco
      editorDidMount={e => {
        e.onKeyDown(e => {
          if ((e.metaKey || e.ctrlKey) && e.code == "KeyS") {
            e.preventDefault();
          }
        });
        setEditor(e);
      }}
      options={{
        wordWrap: "on",
        minimap: {
          enabled: false
        }
      }}
      key={selectedFile + ""}
      height={windowHeight}
      language="markdown"
      value={contents}
      theme="vs-dark"
      onChange={onChange}
    />
  );
};

const getFileName = (de: api.DirEntry) =>
  "File" in de ? de.File.name : de.Folder.name;

const FileBrowser: React.SFC<{
  files: api.DirEntry[];
  selectedItem: string[];
  selectItem: (path: string[]) => void;
}> = ({ files, selectItem, selectedItem }) => {
  return (
    <div className="p-5">
      {files.map(de => (
        <FileBrowserItem
          selectedItem={selectedItem}
          parentPath={[]}
          key={getFileName(de)}
          file={de}
          selectItem={selectItem}
        />
      ))}
    </div>
  );
};

const FileBrowserItem: React.SFC<{
  parentPath: string[];
  file: api.DirEntry;
  selectedItem: string[];
  selectItem: (path: string[]) => void;
}> = ({ parentPath, file, selectedItem, selectItem }) => {
  const name = getFileName(file);
  const path = [...parentPath, name];
  const link = (
    <a
      className={`text-blue-200 ${
        JSON.stringify(path) == JSON.stringify(selectedItem) ? "font-bold" : ""
      }`}
      href="#"
      onClick={e => {
        e.preventDefault();
        selectItem(path);
      }}
    >
      {name}
    </a>
  );
  return "File" in file ? (
    <div>{link}</div>
  ) : (
    <div>
      <span>{name}</span>
      <div className="pl-2">
        {file.Folder.children.map(de => (
          <FileBrowserItem
            key={getFileName(de)}
            parentPath={path}
            file={de}
            selectedItem={selectedItem}
            selectItem={selectItem}
          />
        ))}
      </div>
    </div>
  );
};
ReactDOM.render(<App />, document.getElementById("app"));
