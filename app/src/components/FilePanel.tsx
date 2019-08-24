import * as React from "react";
import { FileId, ProjectIndex } from "../server";

export const FilePanel: React.SFC<{
  newFile: () => any;
  selectFile: (fileId: FileId) => any;
  files: ProjectIndex["files"];
  selectedFile: null | FileId;
}> = ({ newFile, selectFile, files, selectedFile }) => (
  <div className="flex flex-col">
    <div className="flex p-2 w-40">
      <button onClick={newFile}>New File</button>
    </div>
    <div className="flex flex-1 p-2 flex-col">
      {Object.keys(files)
        .sort()
        .map(f => (
          <a
            key={f}
            href="#"
            className={selectedFile && selectedFile.path == f ? "selected" : ""}
            onClick={e => {
              e.preventDefault();
              selectFile({ path: f });
            }}
          >
            {f}
          </a>
        ))}
    </div>
  </div>
);
