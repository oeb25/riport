import * as api from "./api";
import * as sha512 from "js-sha512";

export interface FilePath {}
export interface ProjectId {}

const createFilePath = (path: string): FilePath => path as FilePath;
const filePathToString = (path: FilePath): string => path as string;

export interface FilePathMap<T> {
  __files: T[];
}

export const containsPath = <T>(map: FilePathMap<T>, path: FilePath): boolean =>
  (path as string) in (map as any);
export const getForPath = <T>(map: FilePathMap<T>, path: FilePath): T =>
  (map as any)[path as string];
export const setForPath = <T>(map: FilePathMap<T>, path: FilePath, value: T) =>
  ((map as any)[path as string] = value);
export const removeForPath = <T>(map: FilePathMap<T>, path: FilePath) =>
  delete (map as any)[path as string];
export const createMap = <T>(): FilePathMap<T> => ({} as FilePathMap<T>);
export const cloneMap = <T>(map: FilePathMap<T>): FilePathMap<T> => ({
  ...map
});

export type Index = {
  order: FilePath[];
  files: FilePathMap<FileIndex>;
};

export type File = {
  src: string;
  compiled: string;
};

export type FileIndex = {
  name: FilePath;
  src_hash: string;
  compiled_hash: string;
};

export type Project = {
  id: ProjectId;
  index: Index;
  files: FilePathMap<File>;
};

export type State = {
  projects: ProjectId[];
};

export const hash = (input: string) => input;

export const init = async (): Promise<State> => {
  return {
    projects: await api.getProjects()
  };
};

export const selectProject = (state: State, projectId: ProjectId): Project => {
  return {
    id: projectId,
    index: {
      order: [],
      files: createMap()
    },
    files: createMap()
  };
};

export const updateProject = async (project: Project): Promise<Project> => {
  const newIndex = await api.getIndex({ projectId: project.id });
  const indexDelta = createIndexDelta(project.index, newIndex);

  if (indexDelta.removedFiles.length > 0) {
    const files = { ...project.files };
    for (const file of indexDelta.removedFiles) {
      removeForPath(files, file);
    }

    project = { ...project, index: newIndex, files };
  }
  if (indexDelta.newFiles.length > 0) {
    const files = { ...project.files };
    for (const file of indexDelta.newFiles) {
      const newFile = await api.getFile({ projectId: project.id, path: file });
      setForPath(files, file, newFile);
    }

    project = { ...project, index: newIndex, files };
  }
  if (indexDelta.newCompiledFiles.length > 0) {
    const files = { ...project.files };
    for (const file of indexDelta.newFiles) {
      const newFile = await api.getFile({ projectId: project.id, path: file });
      setForPath(files, file, newFile);
    }

    project = { ...project, index: newIndex, files };
  }

  return project;
};

const keys = <T>(x: T): (keyof T)[] => Object.keys(x) as (keyof T)[];

const createIndexDelta = (a: Index, b: Index) => {
  const aFiles = keys(a.files) as FilePath[];
  const bFiles = keys(b.files) as FilePath[];

  const newFiles = bFiles.filter(f => !containsPath(a.files, f));
  const removedFiles = aFiles.filter(f => !containsPath(b.files, f));
  const newSrcFiles = bFiles.filter(
    f =>
      containsPath(a.files, f) &&
      getForPath(a.files, f).src_hash !== getForPath(b.files, f).src_hash
  );
  const newCompiledFiles = bFiles.filter(
    f =>
      containsPath(a.files, f) &&
      getForPath(a.files, f).compiled_hash !==
        getForPath(b.files, f).compiled_hash
  );

  return { newFiles, removedFiles, newSrcFiles, newCompiledFiles };
};

export const updateSrc = async (
  project: Project,
  path: FilePath,
  value: string
): Promise<Project> => {
  api.updateFile({ projectId: project.id, path, contents: value });
  const newFiles = cloneMap(project.files);
  setForPath<File>(newFiles, path, {
    ...getForPath(newFiles, path),
    src: value
  });
  const newIndexFiles = cloneMap(project.index.files);
  setForPath<FileIndex>(newIndexFiles, path, {
    ...getForPath(newIndexFiles, path),
    src_hash: hash(value)
  });
  return {
    ...project,
    files: newFiles,
    index: { ...project.index, files: newIndexFiles }
  };
};
