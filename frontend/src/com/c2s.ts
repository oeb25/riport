import { Mapper } from '../util'
import { ProjectId, FileId } from './types'

export type Client2Server = Mapper<{
  CreateProject: {
    project_name: string
  }
  Project: {
    id: ProjectId
    msg: Client2ServerProject
  }
}>

export type Client2ServerProject = Mapper<{
  JoinProject: {}
  LeaveProject: {}
  CreateFile: {
    file_name: string
  }
  Reorder: {
    id: FileId
    new_index: number
  }
  File: {
    id: FileId
    msg: Client2ServerProjectFile
  }
}>

export type Client2ServerProjectFile = Mapper<{
  JoinFileSource: {}
  LeaveFileSource: {}
  EditFileSource: { contents: string }
  JoinFileDoc: {}
  LeaveFileDoc: {}
}>
