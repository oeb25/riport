import * as React from "react";

import { ProjectInfo, ProjectId, SystemTime } from "../com/types";
import { Client2Server } from "../com/c2s";

export const Landing: React.SFC<{
  projects: ProjectInfo[];
  selectProject: (projectId: ProjectId) => any;
  send: (msg: Client2Server) => any;
}> = ({ projects, selectProject, send }) => (
  <div className="flex flex-1 justify-center items-center">
    <div className="flex w-full flex-col mb-10 justify-center items-center">
      <h1 className="text-5xl border-b mb-5 px-5 italic">Riport</h1>
      <div className="bg-gray-900 shadow flex-shrink w-full max-w-md rounded">
        <div className="flex p-2 border-b text-gray-500">
          <p className="flex-1 text-gray-500">Projects ({projects.length})</p>
          <a
            className="pl-3 pr-1 hover:text-white"
            href="/"
            onClick={e => {
              e.preventDefault();
            }}
          >
            +
          </a>
        </div>
        <div className="flex flex-col bg-gray-800">
          <div className="flex flex-col">
            {projects.map(info => (
              <ProjectItem
                key={info.id.project_id}
                info={info}
                select={() => selectProject(info.id)}
                send={send}
              />
            ))}
          </div>
          <a
            href="/"
            className="flex p-2 bg-gray-900 text-gray-500 hover:bg-black hover:text-white"
            onClick={e => {
              e.preventDefault();
            }}
          >
            + New Project
          </a>
        </div>
      </div>
    </div>
  </div>
);

const ProjectItem: React.SFC<{
  info: ProjectInfo;
  select: () => any;
  send: (msg: Client2Server) => any;
}> = ({ info, select, send }) => {
  React.useEffect(() => {
    send({ type: "Project", id: info.id, msg: { type: "JoinProject" } });
    return () =>
      send({ type: "Project", id: info.id, msg: { type: "LeaveProject" } });
  }, [info]);

  return (
    <a
      href="/"
      className="flex relative py-1 border-b px-2 last:border-b-0 border-gray-600 items-center hover:bg-gray-700"
      onClick={e => {
        e.preventDefault();
        select();
      }}
    >
      <div className="flex flex-1">{info.name}</div>
      <div className="text-right">
        <div className="text-gray-600 text-xs">Last edit:</div>
        <div className="text-gray-500 text-sm">
          <LiveSince time={systemTime2Date(info.last_changed).valueOf()} />
        </div>
      </div>
      <div className="flex flex-col absolute top-0 left-0 bottom-0 right-0">
        <div
          className="flex flex-1"
          onDragOver={e => {
            // console.log("onDragOver TOP", i);
          }}
        ></div>
        <div
          className="flex flex-1"
          onDragOver={e => {
            // console.log("onDragOver BOTTOM", i);
          }}
        ></div>
      </div>
    </a>
  );
};

const LiveSince: React.SFC<{ time: number }> = ({ time }) => {
  const [delta, setDelta] = React.useState(Date.now() - time);
  React.useEffect(() => {
    const i = setInterval(() => {
      setDelta(Date.now() - time);
    }, 1000);
    return () => clearInterval(i);
  }, [time, setDelta]);

  return <span>{formatDelta(delta)}</span>;
};

export const systemTime2Date = (st: SystemTime): Date => {
  const ms = st.secs_since_epoch * 1000 + st.nanos_since_epoch / 1000000;
  return new Date(ms);
};

const SECOND_IN_MS = 1000;
const MINUTE_IN_MS = SECOND_IN_MS * 60;
const HOUR_IN_MS = MINUTE_IN_MS * 60;

export const formatDelta = (delta: number) => {
  if (delta > HOUR_IN_MS) {
    return `${Math.floor(delta / HOUR_IN_MS)} hours ago`;
  }
  if (delta > MINUTE_IN_MS) {
    return `${Math.floor(delta / MINUTE_IN_MS)} min ago`;
  }
  if (delta > SECOND_IN_MS * 10) {
    return `${Math.floor(delta / SECOND_IN_MS)} sec ago`;
  }
  return `Just now`;
};

export const formatDate = (date: Date) => {
  const delta = Date.now() - date.valueOf();

  return formatDelta(delta);
};
