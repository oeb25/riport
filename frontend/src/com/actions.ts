import { ProjectId, FileId } from './types'

import {
  Client2Server,
  Client2ServerProject,
  Client2ServerProjectFile,
} from './c2s'

export const projectMsg = (
  id: ProjectId,
  msg: Client2ServerProject,
): Client2Server => ({
  type: 'Project',
  id,
  msg,
})
export const fileMsg = (
  projectId: ProjectId,
  id: FileId,
  msg: Client2ServerProjectFile,
): Client2Server =>
  projectMsg(projectId, {
    type: 'File',
    id,
    msg,
  })

export const joinProject = (projectId: ProjectId): Client2Server =>
  projectMsg(projectId, {
    type: 'JoinProject',
  })
export const leaveProject = (projectId: ProjectId): Client2Server =>
  projectMsg(projectId, {
    type: 'LeaveProject',
  })

export const reorderFiles = (
  projectId: ProjectId,
  id: FileId,
  new_index: number,
): Client2Server =>
  projectMsg(projectId, {
    type: 'Reorder',
    id,
    new_index,
  })

export const joinFileSource = (
  projectId: ProjectId,
  fileId: FileId,
): Client2Server =>
  fileMsg(projectId, fileId, {
    type: 'JoinFileSource',
  })
export const leaveFileSource = (
  projectId: ProjectId,
  fileId: FileId,
): Client2Server =>
  fileMsg(projectId, fileId, {
    type: 'LeaveFileSource',
  })

export const joinFileDoc = (
  projectId: ProjectId,
  fileId: FileId,
): Client2Server =>
  fileMsg(projectId, fileId, {
    type: 'JoinFileDoc',
  })
export const leaveFileDoc = (
  projectId: ProjectId,
  fileId: FileId,
): Client2Server =>
  fileMsg(projectId, fileId, {
    type: 'LeaveFileDoc',
  })
