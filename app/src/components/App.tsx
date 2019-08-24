import * as React from "react";
import { Project } from "./Project";
import * as server from "../server";

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

export const App: React.SFC = () => {
  const editorId = getPromise(() => server.getEditorId({}));
  const projects = getPromise(() => server.getProjects({}));
  let [
    selectedProjectId,
    setSelectedProject
  ] = React.useState<null | server.ProjectId>(null);

  if (!editorId) {
    return <div>Geting id...</div>;
  }
  if (!projects) {
    return <div>Geting projects...</div>;
  }

  selectedProjectId = selectedProjectId || projects[0];

  if (!selectedProjectId) {
    return (
      <div>
        <h1>Select project</h1>
        <ul>
          {projects.map(id => (
            <li key={id.project_id}>
              <a
                href="#"
                onClick={e => {
                  e.preventDefault();
                  setSelectedProject(id);
                }}
              >
                {id.project_id}
              </a>
            </li>
          ))}
        </ul>
        <button
          onClick={() => {
            server.newProject({});
          }}
        >
          Create Project
        </button>
      </div>
    );
  }

  return (
    <div className="h-screen flex">
      <Project editorId={editorId} projectId={selectedProjectId}></Project>
    </div>
  );
};
