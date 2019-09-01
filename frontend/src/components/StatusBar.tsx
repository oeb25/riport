import * as React from "react";
import { PathSegment, Routes } from "../state";
import { intersparse } from "../util";

export const StatusBar: React.SFC<{
  path: PathSegment[];
  connectionStatus: string;
  changeRoute: (route: Routes) => any;
}> = ({ path, connectionStatus, changeRoute }) => (
  <div className="flex p-2 bg-gray-900 text-gray-500">
    <div className="flex flex-1">
      {intersparse(
        path.map((p, key) => (
          <a
            key={"link" + key}
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
        i => (
          <span key={i} className="px-2">
            /
          </span>
        )
      )}
    </div>
    <div className="flex">Connection: {connectionStatus}</div>
  </div>
);
