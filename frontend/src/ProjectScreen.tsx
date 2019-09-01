import * as React from "react";
import { editor } from "monaco-editor";
import MonacoEditor from "react-monaco-editor";
import "./editorSetup";

import { Render } from "./Render";
import { ProjectInfo, FileInfo, ProjectFiles, FileId } from "./com/types";
import { Client2Server } from "./com/c2s";
import { getFileName } from "./state";

editor.defineTheme("darko", {
  base: "vs-dark",
  inherit: true,
  rules: [],
  colors: {
    "editor.foreground": "#ffffff",
    "editor.background": "#2d3748"
  }
});

export const ProjectScreen: React.SFC<{
  info: ProjectInfo;
  fileInfos: FileInfo[];
  files: ProjectFiles;
  send: (msg: Client2Server) => any;
  selectedFile: FileId;
  selectFile: (id: FileId) => any;
}> = ({ info, fileInfos, files, send, selectedFile, selectFile }) => {
  React.useEffect(() => {
    send({
      type: "Project",
      id: info.id,
      msg: {
        type: "File",
        id: selectedFile,
        msg: {
          type: "JoinFileSource"
        }
      }
    });

    return () => {
      send({
        type: "Project",
        id: info.id,
        msg: {
          type: "File",
          id: selectedFile,
          msg: {
            type: "LeaveFileSource"
          }
        }
      });
    };
  }, [selectedFile.file_id]);

  const f =
    files && selectedFile.file_id in files ? files[selectedFile.file_id] : null;

  return (
    <div className="flex flex-1">
      <div className="flex flex-col my-2 ml-2 mr-1 shadow-xl bg-gray-800 w-40">
        <div className="flex flex-col px-2 py-1 bg-gray-900">Files</div>
        <div className="flex flex-col px-2 py-1">
          {info.files.map(file => (
            <a
              href="/"
              onClick={e => {
                e.preventDefault();
                selectFile(file);
              }}
            >
              {getFileName(files, info.id, file) || "???"}
            </a>
          ))}
        </div>
      </div>
      <div className="flex flex-1 flex-col my-2 ml-2 mr-1 shadow-xl bg-gray-800">
        <EditorView
          value={f ? f.src : ""}
          setValue={value => {
            send({
              type: "Project",
              id: info.id,
              msg: {
                type: "File",
                id: selectedFile,
                msg: {
                  type: "EditFileSource",
                  contents: value
                }
              }
            });
          }}
        ></EditorView>
      </div>
      <div className="flex flex-1 my-2 ml-1 mr-2 shadow-xl bg-gray-800">
        <div className="flex flex-1 p-2 markdown">
          <Render src={f && f.doc ? f.doc : []} staticUrl={s => s}></Render>
        </div>
      </div>
    </div>
  );
};

const EditorView: React.SFC<{
  value: string;
  setValue: (value: string) => any;
}> = ({ value, setValue }) => {
  return (
    <div className="flex flex-1">
      <MonacoEditor
        value={value}
        options={{
          lineNumbers: "off",
          language: "markdown",
          minimap: {
            enabled: false
          }
        }}
        onChange={value => {
          setValue(value);
        }}
        theme="solarized-dark"
      />
    </div>
  );
};
