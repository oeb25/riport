import * as React from "react";
import { socket, SocketProvider } from "../com/socket";
import { reducer, initialState, Routes, buildPath } from "../state";
import { StatusBar } from "./StatusBar";
import { Router } from "./Router";

export const App: React.SFC = () => {
  const [state, dispatch] = React.useReducer(reducer, initialState);

  const [wsStatus, send] = socket(msg => dispatch({ type: "Server", msg }));

  const { route } = state;

  const changeRoute = (newRoute: Routes) => {
    // if (route.name == "landing" && newRoute.name == "project") {
    //   send({ type: "Project", id: newRoute.id, msg: { type: "JoinProject" } });
    // }
    // if (route.name == "project" && newRoute.name == "landing") {
    //   send({ type: "Project", id: route.id, msg: { type: "LeaveProject" } });
    // }

    dispatch({ type: "SetRoute", route: newRoute });
  };

  React.useEffect(() => {
    let t = setTimeout(() => {
      const hash = JSON.parse(
        decodeURIComponent(window.location.hash.slice(1)) ||
          '{ "name": "landing" }'
      );
      changeRoute(hash);
    }, 100);
    return () => clearTimeout(t);
  }, [wsStatus.type]);

  return (
    <SocketProvider wsStatus={wsStatus}>
      <div className="flex flex-1 h-full flex-col items-stretch bg-gray-700">
        <StatusBar
          path={buildPath(state)}
          changeRoute={changeRoute}
          connectionStatus={wsStatus.type}
        ></StatusBar>
        <div className="flex flex-1 h-full">
          <Router
            state={state}
            route={route}
            changeRoute={changeRoute}
            send={send}
            editFile={(projectId, fileId, value) => {
              dispatch({
                type: "UpdateFileValue",
                projectId,
                fileId,
                value
              });

              send({
                type: "Project",
                id: projectId,
                msg: {
                  type: "File",
                  id: fileId,
                  msg: {
                    type: "EditFileSource",
                    contents: value
                  }
                }
              });
            }}
          />
        </div>
      </div>
    </SocketProvider>
  );
};
