export const base = "/api";

export const post = async (path: string, data: object = {}) =>
  await fetch(`${base}/${path}`, {
    method: "POST",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/json"
    },
    body: JSON.stringify(data)
  });

export const json = async (path: string, data: object = {}) =>
  (await post(path, data)).json();

export const text = async (path: string, data: object = {}) =>
  (await post(path, data)).text();

export const getProjects = () => json("projects") as Promise<string[]>;

export type File = { File: { name: string } };
export type Folder = { Folder: { name: string; children: DirEntry[] } };
export type DirEntry = File | Folder;

export const getFiles = (p: { projectId: string }) =>
  json(`projects/${p.projectId}/files`) as Promise<DirEntry[]>;

export const getFile = (p: { projectId: string; path: string[] }) =>
  text(`projects/${p.projectId}/file`, { path: p.path });

export const updateFile = (p: {
  projectId: string;
  path: string[];
  contents: string;
}) => post(`projects/${p.projectId}/updateFile`, p);

export const getOutput = (p: { projectId: string }) =>
  json(`projects/${p.projectId}/output`);
