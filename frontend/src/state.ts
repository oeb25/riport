import {
  ProjectId,
  ProjectInfo,
  FileInfo,
  ProjectFiles,
  FileId,
} from './com/types'
import {
  Server2Client,
  Server2Client_Project,
  Server2Client_Project_File,
} from './com/s2c'

export type State = {
  route: Routes
  projects: ProjectInfo[]
  projectFileInfos: { [project_id: number]: FileInfo[] }
  projectFiles: { [project_id: number]: ProjectFiles }
}

export type PathSegment = { name: string; route: Routes }

export type Routes =
  | {
      name: 'landing'
    }
  | {
      name: 'project'
      id: ProjectId
      file?: FileId
    }

export const initialState: State = {
  route: { name: 'landing' },
  projects: [],
  projectFileInfos: {},
  projectFiles: {},
}

export type Action =
  | { type: 'Server'; msg: Server2Client }
  | { type: 'SetRoute'; route: Routes }
  | {
      type: 'UpdateFileValue'
      projectId: ProjectId
      fileId: FileId
      value: string
    }

export const reducer: React.Reducer<State, Action> = (state, action) => {
  switch (action.type) {
    case 'SetRoute': {
      window.location.hash = encodeURIComponent(JSON.stringify(action.route))
      return { ...state, route: action.route }
    }
    case 'Server': {
      return handleServerMsg(state, action.msg)
    }
    case 'UpdateFileValue': {
      const projectFiles = state.projectFiles[action.projectId.project_id] || {}
      const f = projectFiles[action.fileId.file_id] || {
        doc: null,
        id: action.fileId,
        name: 'idk',
        src: action.value,
      }

      return {
        ...state,
        projectFiles: {
          ...state.projectFiles,
          [action.projectId.project_id]: {
            ...projectFiles,
            [action.fileId.file_id]: {
              ...f,
              src: action.value,
            },
          },
        },
      }
    }
  }
}

const handleServerMsg = (state: State, msg: Server2Client): State => {
  switch (msg.type) {
    case 'Projects': {
      return { ...state, projects: msg.list }
    }
    case 'Project': {
      const { id, msg: msg2 } = msg
      return handleServerProjectMsg(state, id, msg2)
    }
    default: {
      console.log('unhandled', msg)
      return state
    }
  }
}

const handleServerProjectMsg = (
  state: State,
  projectId: ProjectId,
  msg: Server2Client_Project,
): State => {
  switch (msg.type) {
    case 'UpdateInfo': {
      return {
        ...state,
        projects: state.projects.map(p => {
          if (p.id.project_id == msg.info.id.project_id) {
            return msg.info
          } else {
            return p
          }
        }),
      }
    }
    case 'Files': {
      return {
        ...state,
        projectFileInfos: {
          ...state.projectFileInfos,
          [projectId.project_id]: msg.list,
        },
        projectFiles: {
          ...state.projectFiles,
          [projectId.project_id]: msg.list.reduce(
            (acc, f) => {
              const pfs = state.projectFiles[projectId.project_id]
              acc[f.id.file_id] = (pfs && pfs[f.id.file_id]) || {
                doc: null,
                id: f.id,
                name: f.name,
                src: '',
              }
              return acc
            },
            {} as ProjectFiles,
          ),
        },
      }
    }
    case 'File': {
      const { id: fileId, msg: msg2 } = msg
      return handleServerProjectFileMsg(state, projectId, fileId, msg2)
    }
    default: {
      console.log('unhandled project', msg)
      return state
    }
  }
}

const handleServerProjectFileMsg = (
  state: State,
  projectId: ProjectId,
  fileId: FileId,
  msg: Server2Client_Project_File,
) => {
  switch (msg.type) {
    case 'FileSource': {
      const projectFiles = state.projectFiles[projectId.project_id] || {}
      const f = projectFiles[fileId.file_id] || {
        doc: null,
        id: fileId,
        name: 'idk',
        src: msg.src,
      }

      return {
        ...state,
        projectFiles: {
          ...state.projectFiles,
          [projectId.project_id]: {
            ...projectFiles,
            [fileId.file_id]: {
              ...f,
              src: msg.src,
            },
          },
        },
      }
    }
    case 'FileDoc': {
      const projectFiles = state.projectFiles[projectId.project_id] || {}
      const f = projectFiles[fileId.file_id] || {
        doc: msg.doc,
        id: fileId,
        name: 'idk',
        src: '',
      }

      return {
        ...state,
        projectFiles: {
          ...state.projectFiles,
          [projectId.project_id]: {
            ...projectFiles,
            [fileId.file_id]: {
              ...f,
              doc: msg.doc || f.doc,
            },
          },
        },
      }
    }
    default: {
      console.log('unhandled file', msg)
      return state
    }
  }
}

export const findProjectInfo = (
  state: State,
  id: ProjectId,
): ProjectInfo | null =>
  state.projects.filter(info => info.id.project_id == id.project_id)[0]

export const getFileName = (
  c: State | ProjectFiles | undefined,
  projectId: ProjectId,
  fileId: FileId,
): string | null => {
  if (!c) return null

  const pfs = 'route' in c ? c.projectFiles[projectId.project_id] : c
  if (!pfs) return null

  if (fileId.file_id in pfs) {
    return pfs[fileId.file_id].name
  }
  return null
}

export const buildPath = (state: State): PathSegment[] => {
  const path: PathSegment[] = [
    {
      name: 'Riport',
      route: { name: 'landing' },
    },
  ]

  const { route } = state

  if (route.name == 'project') {
    const info = findProjectInfo(state, route.id)
    if (!info) return path
    path.push({
      name: info.name,
      route: {
        name: 'project',
        id: info.id,
      },
    })
    if (route.file) {
      path.push({
        name: getFileName(state, route.id, route.file) || '???',
        route: {
          name: 'project',
          id: info.id,
          file: route.file,
        },
      })
    }
  }
  return path
}
