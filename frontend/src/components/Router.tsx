import * as React from 'react'
import { State, Routes, findProjectInfo } from '../state'
import { Client2Server } from '../com/c2s'
import { ProjectId, FileId } from '../com/types'
import { Landing } from './Landing'
import { ProjectScreen, ProjectInfoContext } from './ProjectScreen'

export const Router: React.SFC<{
  state: State
  route: Routes
  changeRoute: (route: Routes) => any
  send: (msg: Client2Server) => any
  editFile: (projectId: ProjectId, fileId: FileId, value: string) => any
}> = ({ state, route, changeRoute, send, editFile }) => {
  switch (route.name) {
    case 'landing': {
      return (
        <Landing
          projects={state.projects}
          selectProject={id => {
            changeRoute({ name: 'project', id })
          }}
          send={send}
        />
      )
    }
    case 'project': {
      return (
        <ProjectInfoContext.Provider value={findProjectInfo(state, route.id)}>
          <ProjectScreen
            fileInfos={state.projectFileInfos[route.id.project_id] || {}}
            files={state.projectFiles[route.id.project_id]}
            send={send}
            selectConfig={() => {
              changeRoute({ name: 'project', id: route.id, config: true })
            }}
            selectFile={id => {
              changeRoute({ name: 'project', id: route.id, file: id })
            }}
            selectedFile={'file' in route ? route.file : null}
            editFile={(id, value) => {
              editFile(route.id, id, value)
            }}
          />
        </ProjectInfoContext.Provider>
      )
    }
  }
}
