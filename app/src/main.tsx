import * as React from "react";
import * as ReactDOM from "react-dom";
import ReactMonaco from "react-monaco-editor";

import * as state from "./state";
import "./editorSetup";
import * as api from "./api";
import { Fragment, Render } from "./pandoc";
import renderMathInElement from "katex/dist/contrib/auto-render";
import "./server";
import { App as NewApp } from "./components/App";

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

const DarkMode = React.createContext({ on: true, toggle: () => {} });

const App: React.SFC = () => {
  const [darkmode, setDarkmode] = React.useState(true);
  const [projectId, setProjectId] = React.useState<null | state.ProjectId>(
    null
  );
  const s = getPromise(async () => {
    const s = await state.init();
    setProjectId(s.projects[0]);
    return s;
  });
  const [project, setProject] = React.useState<null | state.Project>(null);
  useInterval(async () => {
    if (s && projectId && (!project || project.id != projectId)) {
      setProject(state.selectProject(s, projectId));
      return;
    }
    if (project) {
      state.updateProject(project);
    }
  }, 200);

  return (
    <DarkMode.Provider
      value={{ on: darkmode, toggle: () => setDarkmode(!darkmode) }}
    >
      <div className={`flex flex-1 ${darkmode ? "" : "bright"}`}>
        {project && (
          <Project
            project={project}
            updateSrc={async (name, value) => {
              setProject(await state.updateSrc(project, name, value));
            }}
          />
        )}
      </div>
    </DarkMode.Provider>
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

const Project: React.SFC<{
  project: state.Project;
  updateSrc: (name: state.FilePath, value: string) => any;
}> = ({ project, updateSrc }) => {
  // const files = getPromise(() => api.getFiles({ projectId }));
  const [selectedFile, setSelectedFile] = React.useState<null | state.FilePath>(
    null
  );
  // const selectedFile =
  //   selectedFile_ ||
  //   (files && "File" in files[0] ? [(files[0] as api.File).File.name] : null);
  return (
    <div className="flex flex-1 flex-row items-stretch">
      <FileBrowser
        project={project}
        selectedItem={selectedFile || null}
        selectItem={path => setSelectedFile(path)}
      />
      <EditorSpace
        project={project}
        selectedFile={selectedFile}
        updateSrc={updateSrc}
      />
    </div>
  );
};

const EditorSpace: React.SFC<{
  project: state.Project;
  selectedFile: null | state.FilePath;
  updateSrc: (name: state.FilePath, value: string) => any;
}> = ({ project, selectedFile, updateSrc }) => {
  // const [contents, setContents] = React.useState("");
  // const [updateInterval, setUpdateInterval] = React.useState(500);
  // useInterval(() => {
  //   if (selectedFile && updateInterval < 10000) {
  //     api.getFile({ projectId, path: selectedFile }).then(v => setContents(v));
  //   }
  // }, updateInterval);
  // const [output, setOutput] = React.useState<Document>({ blocks: [] });
  // useInterval(() => {
  //   api.getOutput({ projectId }).then(output => setOutput(output));
  // }, 500);
  // const [updateTimeout, setUpdateTimeout] = React.useState(0);
  // const [updateTimeout2, setUpdateTimeout2] = React.useState(0);
  const [containerWidth, setContainerWidth] = React.useState(0);
  const [containerHeight, setContainerHeight] = React.useState(0);

  return (
    <div
      className="flex flex-1"
      ref={container => {
        if (container) {
          setContainerWidth(container.clientWidth);
          setContainerHeight(container.clientHeight);
        }
      }}
    >
      <div className="flex flex-1">
        <Editor
          selectedFile={selectedFile}
          project={project}
          width={containerWidth / 2}
          height={containerHeight}
          onChange={value => {
            if (selectedFile) {
              updateSrc(selectedFile, value);
            }
            // if (selectedFile) {
            //   setContents(value);
            //   setUpdateInterval(10000000);
            //   clearTimeout(updateTimeout);
            //   setUpdateTimeout(
            //     setTimeout(() => {
            //       api.updateFile({
            //         projectId,
            //         path: selectedFile,
            //         contents: value
            //       });
            //     }, 200)
            //   );
            //   clearTimeout(updateTimeout2);
            //   setUpdateTimeout2(
            //     setTimeout(() => {
            //       setUpdateInterval(500);
            //     }, 1000)
            //   );
            // }
          }}
        />
      </div>
      <Output
        fragments={project.index.order.reduce<Fragment[]>(
          (fragments, name) => [
            ...fragments,
            ...JSON.parse(state.getForPath(project.files, name).compiled).blocks
          ],
          [] as Fragment[]
        )}
        projectId={project.id}
      />
    </div>
  );
};

const Output: React.SFC<{
  fragments: Fragment[];
  projectId: state.ProjectId;
}> = ({ fragments, projectId }) => (
  <div className="flex flex-1 max-h-screen p-5 overflow-y-auto">
    <div className="flex flex-1 markdown">
      <Render src={fragments} projectId={projectId} />
      <div className="pb-20" />
    </div>
  </div>
);

const Editor: React.SFC<{
  selectedFile: state.FilePath | null;
  project: state.Project;
  width: number;
  height: number;
  onChange: (value: string) => any;
}> = ({ selectedFile, project, width, height, onChange }) => {
  // const [
  //   editor,
  //   setEditor
  // ] = React.useState<null | Monaco.editor.IStandaloneCodeEditor>(null);
  // const [windowHeight, setWindowHeight] = React.useState(window.innerHeight);
  // React.useEffect(() => {
  //   const listener = () => {
  //     if (editor) {
  //       editor.layout({
  //         width: window.innerWidth / 2,
  //         height: windowHeight
  //       });
  //     }
  //     setWindowHeight(window.innerHeight);
  //   };
  //   window.addEventListener("resize", listener);
  //   return () => {
  //     console.log("remove listener");
  //     window.removeEventListener("resize", listener);
  //   };
  // }, [editor]);

  return (
    <DarkMode.Consumer>
      {darkmode => (
        <ReactMonaco
          editorDidMount={e => {
            e.onKeyDown(e => {
              if ((e.metaKey || e.ctrlKey) && e.code == "KeyS") {
                e.preventDefault();
              }
            });
            // setEditor(e);
          }}
          options={{
            wordWrap: "on",
            minimap: {
              enabled: false
            }
          }}
          key={selectedFile + ""}
          width={width}
          height={height}
          language="markdown"
          value={
            selectedFile
              ? state.getForPath(project.files, selectedFile).src
              : ""
          }
          theme={darkmode.on ? "vs-dark" : "vs-bright"}
          onChange={onChange}
        />
      )}
    </DarkMode.Consumer>
  );
};

const FileBrowser: React.SFC<{
  project: state.Project;
  selectedItem: state.FilePath | null;
  selectItem: (path: state.FilePath) => void;
}> = ({ project, selectItem, selectedItem }) => {
  const darkmode = React.useContext(DarkMode);

  return (
    <div className="flex flex-col p-5">
      <div className="flex flex-1 flex-col">
        {project.index.order.map(name => (
          <a
            className={`${
              JSON.stringify(name) == JSON.stringify(selectedItem)
                ? "selected"
                : ""
            }`}
            href="#"
            onClick={e => {
              e.preventDefault();
              selectItem(name);
            }}
          >
            {name}
          </a>

          // <FileBrowserItem
          //   selectedItem={selectedItem}
          //   parentPath={[]}
          //   key={name}
          //   file={{ File: { name } }}
          //   selectItem={selectItem}
          // />
        ))}
      </div>
      <div>
        <a
          href="#"
          onClick={e => {
            e.preventDefault();
            darkmode.toggle();
          }}
        >
          <svg
            focusable="false"
            role="img"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 512 512"
            className="w-6 h-6"
          >
            <path
              fill="currentColor"
              d="M279.135 512c78.756 0 150.982-35.804 198.844-94.775 28.27-34.831-2.558-85.722-46.249-77.401-82.348 15.683-158.272-47.268-158.272-130.792 0-48.424 26.06-92.292 67.434-115.836 38.745-22.05 28.999-80.788-15.022-88.919A257.936 257.936 0 0 0 279.135 0c-141.36 0-256 114.575-256 256 0 141.36 114.576 256 256 256zm0-464c12.985 0 25.689 1.201 38.016 3.478-54.76 31.163-91.693 90.042-91.693 157.554 0 113.848 103.641 199.2 215.252 177.944C402.574 433.964 344.366 464 279.135 464c-114.875 0-208-93.125-208-208s93.125-208 208-208z"
            />
          </svg>
        </a>
      </div>
    </div>
  );
};

// const FileBrowserItem: React.SFC<{
//   parentPath: string[];
//   file: api.DirEntry;
//   selectedItem: string[];
//   selectItem: (path: string[]) => void;
// }> = ({ parentPath, file, selectedItem, selectItem }) => {
//   const name = getFileName(file);
//   const path = [...parentPath, name];
//   const link = (
//     <a
//       className={`${
//         JSON.stringify(path) == JSON.stringify(selectedItem) ? "selected" : ""
//       }`}
//       href="#"
//       onClick={e => {
//         e.preventDefault();
//         selectItem(path);
//       }}
//     >
//       {name}
//     </a>
//   );
//   return "File" in file ? (
//     <div>{link}</div>
//   ) : (
//     <div>
//       <span>{name}</span>
//       <div className="pl-2">
//         {file.Folder.children.map(de => (
//           <FileBrowserItem
//             key={getFileName(de)}
//             parentPath={path}
//             file={de}
//             selectedItem={selectedItem}
//             selectItem={selectItem}
//           />
//         ))}
//       </div>
//     </div>
//   );
// };
ReactDOM.render(<NewApp />, document.getElementById("app"));
