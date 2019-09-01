import * as React from "react";
import * as ReactDOM from "react-dom";
import { Landing } from "./Landing";
import { ProjectScreen } from "./ProjectScreen";

import { intersparse } from "./util";
import { socket } from "./com/socket";
import {
  State,
  Routes,
  reducer,
  initialState,
  buildPath,
  PathSegment,
  findProjectInfo
} from "./state";
import { Client2Server } from "./com/c2s";

const App: React.SFC = () => {
  const [state, dispatch] = React.useReducer(reducer, initialState);

  const [wsStatus, send] = socket(msg => dispatch({ type: "Server", msg }));

  const { route } = state;

  const changeRoute = (newRoute: Routes) => {
    if (route.name == "landing" && newRoute.name == "project") {
      send({ type: "Project", id: newRoute.id, msg: { type: "JoinProject" } });
    }
    if (route.name == "project" && newRoute.name == "landing") {
      send({ type: "Project", id: route.id, msg: { type: "LeaveProject" } });
    }

    dispatch({ type: "SetRoute", route: newRoute });
  };

  return (
    <div className="flex flex-1 flex-col items-stretch bg-gray-700">
      <Footer
        path={buildPath(state)}
        changeRoute={changeRoute}
        connectionStatus={wsStatus.type}
      ></Footer>
      <div className="flex flex-1">
        <Router
          state={state}
          route={route}
          changeRoute={changeRoute}
          send={send}
        />
      </div>
    </div>
  );
};

const Footer: React.SFC<{
  path: PathSegment[];
  connectionStatus: string;
  changeRoute: (route: Routes) => any;
}> = ({ path, connectionStatus, changeRoute }) => (
  <div className="flex p-2 bg-gray-900 text-gray-500">
    <div className="flex flex-1">
      {intersparse(
        path.map((p, key) => (
          <a
            key={key}
            href="/"
            className="hover:text-white"
            onClick={e => {
              e.preventDefault();
              changeRoute(p.route);
            }}
          >
            {p.name}
          </a>
        )),
        <span className="px-2">/</span>
      )}
    </div>
    <div className="flex">Connection: {connectionStatus}</div>
  </div>
);

const Router: React.SFC<{
  state: State;
  route: Routes;
  changeRoute: (route: Routes) => any;
  send: (msg: Client2Server) => any;
}> = ({ state, route, changeRoute, send }) => {
  switch (route.name) {
    case "landing": {
      return (
        <Landing
          projects={state.projects}
          selectProject={id => {
            changeRoute({ name: "project", id });
          }}
        />
      );
    }
    case "project": {
      return (
        <ProjectScreen
          info={findProjectInfo(state, route.id)}
          fileInfos={state.projectFileInfos[route.id.project_id] || {}}
          files={state.projectFiles[route.id.project_id]}
          send={send}
          selectFile={id => {
            changeRoute({ name: "project", id: route.id, file: id });
          }}
          selectedFile={
            route.file || state.projects[route.id.project_id].files[0]
          }
        />
      );
    }
  }
};

ReactDOM.render(<App />, document.getElementById("app"));
