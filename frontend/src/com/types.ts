import { Fragment } from '../components/Render'

export type SystemTime = {
  secs_since_epoch: number
  nanos_since_epoch: number
}

export type File = {
  name: string
  id: FileId
  src: string
  doc: null | Doc
}

export type ProjectFiles = {
  [file_id: number]: File
}

export type ProjectId = { project_id: number }
export type FileId = { file_id: number }

export type ProjectInfo = {
  name: string
  id: ProjectId
  files: FileId[]
}
export type FileInfo = {
  name: string
  id: FileId
}
export type Doc = Fragment[]
