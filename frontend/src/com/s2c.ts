import { Mapper } from '../util'
import { ProjectInfo, ProjectId, FileInfo, FileId, Doc } from './types'

export type Lock = Mapper<{
  Unlock: {}
  LockBy: {}
  LockByMe: {}
}>

export type Server2Client = Mapper<{
  Projects: {
    list: ProjectInfo[]
  }
  Project: {
    id: ProjectId
    msg: Server2ClientProject
  }
}>

export type Server2ClientProject = Mapper<{
  Files: {
    list: FileInfo[]
  }
  UpdateInfo: {
    info: ProjectInfo
  }
  File: {
    id: FileId
    msg: Server2ClientProjectFile
  }
}>

export type Server2ClientProjectFile = Mapper<{
  FileLock: { lock: Lock }
  FileSource: { src: string }
  FileDoc: { doc: Doc }
}>
