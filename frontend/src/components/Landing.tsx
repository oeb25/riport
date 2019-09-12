import * as React from 'react'

import { ProjectInfo, ProjectId, SystemTime } from '../com/types'
import { Client2Server } from '../com/c2s'
import { List } from './List'

export const Landing: React.SFC<{
  projects: ProjectInfo[]
  selectProject: (projectId: ProjectId) => any
  send: (msg: Client2Server) => any
}> = ({ projects, selectProject, send }) => (
  <div className="flex flex-1 justify-center items-center">
    <div className="flex w-full flex-col mb-10 justify-center items-center">
      <h1 className="text-5xl border-b mb-5 px-5 italic">Riport</h1>

      <List
        items={projects}
        title={`Projects (${projects.length})`}
        keyer={info => info.id.project_id}
        render={info => <ProjectListItem info={info} send={send} />}
        select={info => selectProject(info.id)}
        isSelected={info => false}
        reorder={() => {}}
        footer="+ New Project"
      />
    </div>
  </div>
)

const ProjectListItem: React.SFC<{
  info: ProjectInfo
  send: (msg: Client2Server) => any
}> = ({ info, send }) => {
  React.useEffect(() => {
    send({ type: 'Project', id: info.id, msg: { type: 'JoinProject' } })
    return () =>
      send({ type: 'Project', id: info.id, msg: { type: 'LeaveProject' } })
  }, [info])

  return (
    <>
      <div className="flex flex-1 items-center">{info.name}</div>
      <div className="text-right">
        <div className="text-gray-600 text-xs">Last edit:</div>
        <div className="text-gray-500 text-sm">
          {/* <LiveSince time={systemTime2Date(info.last_changed).valueOf()} /> */}
        </div>
      </div>
    </>
  )
}

const LiveSince: React.SFC<{ time: number }> = ({ time }) => {
  const [delta, setDelta] = React.useState(Date.now() - time)
  React.useEffect(() => {
    const i = setInterval(() => {
      setDelta(Date.now() - time)
    }, 1000)
    return () => clearInterval(i)
  }, [time, setDelta])

  return <span>{formatDelta(delta)}</span>
}

export const systemTime2Date = (st: SystemTime): Date => {
  const ms = st.secs_since_epoch * 1000 + st.nanos_since_epoch / 1000000
  return new Date(ms)
}

const SECOND_IN_MS = 1000
const MINUTE_IN_MS = SECOND_IN_MS * 60
const HOUR_IN_MS = MINUTE_IN_MS * 60

export const formatDelta = (delta: number) => {
  if (delta > HOUR_IN_MS) {
    return `${Math.floor(delta / HOUR_IN_MS)} hours ago`
  }
  if (delta > MINUTE_IN_MS) {
    return `${Math.floor(delta / MINUTE_IN_MS)} min ago`
  }
  if (delta > SECOND_IN_MS * 10) {
    return `${Math.floor(delta / SECOND_IN_MS)} sec ago`
  }
  return `Just now`
}

export const formatDate = (date: Date) => {
  const delta = Date.now() - date.valueOf()

  return formatDelta(delta)
}
