export const base = "http://localhost:8000/api2";

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
const text = async (path: string, body: object = {}) =>
  (await post(path, body)).text();
const jsonReq = async (path: string, body: object = {}) =>
  JSON.parse(await text(path, body));

export type ProjectId = { project_id: string };
export type FileId = { path: string };
export type EditorId = { editor_id: string };

export type SystemTime = {
  nanos_since_epoch: number;
  secs_since_epoch: number;
};
export type Duration = number;

export const systemTimeToDate = (st: SystemTime): Date => {
  const ms = st.secs_since_epoch * 1000 + st.nanos_since_epoch / 1000000;
  return new Date(ms);
};

export type Lock = {
  by: EditorId;
  locked_at: SystemTime;
  duration: Duration;
};
export type Change = {
  by: EditorId;
  time: SystemTime;
};

export type FileIndex = {
  lock: null | Lock;
  last_source_change: null | Change;
  last_compiled_time: null | SystemTime;
};
export type ProjectIndex = {
  order: FileId[];
  files: { [F in FileId["path"]]: FileIndex };
};

export type ProjectIndexDeltaItem = {
  id: FileId;
  index: FileIndex;
};
export type ProjectIndexDelta = {
  new_files: ProjectIndexDeltaItem[];
  removed_files: ProjectIndexDeltaItem[];
  changed_source_files: ProjectIndexDeltaItem[];
  changed_compiled_files: ProjectIndexDeltaItem[];
};

const json = <Body extends object, Res>(path: string) => (body: Body) =>
  jsonReq(`${path}`, body) as Promise<Res>;

const projectJson = <Body extends object, Res>(path: string) => (
  projectId: ProjectId,
  body: Body
) => jsonReq(`${projectId.project_id}${path}`, body) as Promise<Res>;

export const getEditorId = json<{}, EditorId>("/get-editor-id");
export const getProjects = json<{}, ProjectId[]>("/projects");
export const newProject = json<{}, ProjectId>("/new-project");

export const newFile = projectJson<{ name: string }, FileId>("/new-file");
export const index = projectJson<{}, ProjectIndex>("/index");
export const indexDelta = projectJson<
  { index: ProjectIndex; editor: EditorId },
  ProjectIndexDelta
>("/index-delta");
export const fileSrc = projectJson<FileId, string>("/file-src");
export const fileCompiled = projectJson<FileId, string>("/file-compiled");
export const editSrc = projectJson<
  { file_id: FileId; editor: EditorId; value: string },
  SystemTime
>("/edit-src");

export const staticUrl = (projectId: ProjectId, src: string) =>
  `${base}/projects/${projectId.project_id}/static/${src}`;

const run = async () => {
  const editorId = await getEditorId({});
  console.log({ editorId });
  const projectId = await newProject({});
  console.log({ projectId });
  const file = await newFile(projectId, { name: "test" });
  console.log({ file });
  const res = await editSrc(projectId, {
    file_id: file,
    editor: editorId,
    value: "# Hello, world!"
  });
  console.log({ res });
  const compiled = await fileCompiled(projectId, file);
  console.log({ compiled });
  const i = await index(projectId, {});
  console.log({ index: i });
};

// run();
