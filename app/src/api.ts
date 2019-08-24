import { Index, File, Project, ProjectId, FilePath } from "./state";

export const base = "http://localhost:8000/api";

export const post = (path: string, data: object = {}) =>
  fetch(`${base}/${path}`, {
    method: "POST",
    // mode: "no-cors",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/json",
      "Access-Control-Request-Headers": "*",
      "Access-Control-Request-Method": "*"
    },
    body: JSON.stringify(data)
  });

export const json = async (path: string, data: object = {}) => {
  const req = await post(path, data);
  const text = await req.text();
  try {
    return JSON.parse(text);
  } catch (e) {
    console.error("failed to parse", `${base}/${path}`);
    console.error(text);
    throw e;
  }
};

export const text = async (path: string, data: object = {}) =>
  (await post(path, data)).text();

export const getProjects = () => json("projects") as Promise<string[]>;
export const getIndex = (p: { projectId: ProjectId }) =>
  json(`projects/${p.projectId}/index`) as Promise<Index>;

export type File = { File: { name: FilePath } };
export type Folder = { Folder: { name: FilePath; children: DirEntry[] } };
export type DirEntry = File | Folder;

export const getFile = (p: { projectId: ProjectId; path: FilePath }) =>
  json(`projects/${p.projectId}/file`, { path: p.path }) as Promise<File>;

export const updateFile = (p: {
  projectId: ProjectId;
  path: FilePath;
  contents: string;
}) => post(`projects/${p.projectId}/updateFile`, p);

export const getOutput = (p: { projectId: ProjectId }) =>
  json(`projects/${p.projectId}/output`);
