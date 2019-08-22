import * as React from "react";
import * as ReactDOM from "react-dom";
import Monaco from "react-monaco-editor";

import * as api from "./api";

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

  return <div>{projects && <Project projectId={projects[0]} />}</div>;
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
  const [output, setOutput] = React.useState("");
  useInterval(() => {
    api.getOutput({ projectId }).then(output => setOutput(output));
  }, 1000);
  const [updateTimeout, setUpdateTimeout] = React.useState(0);
  const [updateTimeout2, setUpdateTimeout2] = React.useState(0);

  return (
    <div className="flex flex-row">
      {files && (
        <FileBrowser files={files} selectItem={path => setSelectedFile(path)} />
      )}
      <div className="flex flex-1">
        <Monaco
          key={selectedFile + ""}
          height={window.innerHeight}
          language="json"
          value={contents}
          onChange={e => {
            if (selectedFile) {
              setContents(e);

              setUpdateInterval(10000000);
              clearTimeout(updateTimeout);
              setUpdateTimeout(
                setTimeout(() => {
                  api.updateFile({
                    projectId,
                    path: selectedFile,
                    contents: e
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
      <div
        className="flex flex-1 p-5 markdown"
        dangerouslySetInnerHTML={{ __html: output || "" }}
      />
    </div>
  );
};

const getFileName = (de: api.DirEntry) =>
  "File" in de ? de.File.name : de.Folder.name;

const FileBrowser: React.SFC<{
  files: api.DirEntry[];
  selectItem: (path: string[]) => void;
}> = ({ files, selectItem }) => {
  return (
    <div className="p-5">
      {files.map(de => (
        <FileBrowserItem
          key={getFileName(de)}
          file={de}
          selectItem={selectItem}
        />
      ))}
    </div>
  );
};

const FileBrowserItem: React.SFC<{
  file: api.DirEntry;
  selectItem: (path: string[]) => void;
}> = ({ file, selectItem }) => {
  const name = getFileName(file);
  const link = (
    <a
      href="#"
      onClick={e => {
        e.preventDefault();
        selectItem([name]);
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
            file={de}
            selectItem={path => selectItem([getFileName(file), ...path])}
          />
        ))}
      </div>
    </div>
  );
};
ReactDOM.render(<App />, document.getElementById("app"));
