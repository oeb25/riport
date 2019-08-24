import * as React from "react";
import * as server from "../server";
import ReactMonaco from "react-monaco-editor";
import { DarkMode } from "./DarkMode";

export const Editor: React.SFC<{
  value: string;
  locked: boolean;
  width: number;
  height: number;
  onChange: (value: string) => any;
}> = ({ value, locked, width, height, onChange }) => {
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
          width={width}
          height={height}
          language="markdown"
          value={value}
          theme={darkmode.on ? "vs-dark" : "vs-bright"}
          onChange={onChange}
        />
      )}
    </DarkMode.Consumer>
  );
};
