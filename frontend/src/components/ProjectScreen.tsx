import * as React from 'react'
import { editor } from 'monaco-editor'
import MonacoEditor from 'react-monaco-editor'
import '../editorSetup'

import { Render } from './Render'
import {
  ProjectInfo,
  FileInfo,
  ProjectFiles,
  FileId,
  ProjectId,
  Doc,
} from '../com/types'
import { Client2Server } from '../com/c2s'
import { getFileName } from '../state'
import { List } from './List'

editor.defineTheme('darko', {
  base: 'vs-dark',
  inherit: true,
  rules: [],
  colors: {
    'editor.foreground': '#ffffff',
    'editor.background': '#2d3748',
  },
})

export const ProjectScreen: React.SFC<{
  info: ProjectInfo | null
  fileInfos: { [fileId: number]: FileInfo }
  files: ProjectFiles
  send: (msg: Client2Server) => any
  selectedFile: FileId | null
  editFile: (id: FileId, value: string) => any
  selectFile: (id: FileId) => any
}> = ({ info, fileInfos, files, send, selectedFile, selectFile, editFile }) => {
  const [
    editor,
    setEditor,
  ] = React.useState<null | editor.IStandaloneCodeEditor>(null)

  React.useEffect(() => {
    if (info) {
      send({ type: 'Project', id: info.id, msg: { type: 'JoinProject' } })
      return () =>
        send({ type: 'Project', id: info.id, msg: { type: 'LeaveProject' } })
    }
  }, [info && info.id.project_id])

  React.useEffect(() => {
    if (selectedFile && info) {
      send({
        type: 'Project',
        id: info.id,
        msg: {
          type: 'File',
          id: selectedFile,
          msg: {
            type: 'JoinFileSource',
          },
        },
      })
    }
    return () => {
      if (selectedFile && info)
        send({
          type: 'Project',
          id: info.id,
          msg: {
            type: 'File',
            id: selectedFile,
            msg: {
              type: 'LeaveFileSource',
            },
          },
        })
    }
  }, [selectedFile && selectedFile.file_id])

  const f =
    files && selectedFile && selectedFile.file_id in files
      ? files[selectedFile.file_id]
      : null

  React.useEffect(() => {
    let stop = false

    const loop = () => {
      if (stop) return
      if (editor) editor.layout()
      requestAnimationFrame(loop)
    }

    const t = setTimeout(() => {
      // resize
      stop = true
    }, 500)

    loop()

    return () => {
      clearTimeout(t)
      stop = true
    }
  }, [selectedFile && selectedFile.file_id, f && f.id.file_id])

  React.useEffect(() => {
    const resize = () => {
      if (editor) editor.layout()
    }
    window.addEventListener('resize', resize)
    return () => window.removeEventListener('resize', resize)
  }, [editor])

  const animate = true

  return (
    <div className="flex flex-1 h-full">
      <div className="flex flex-col my-2 ml-2 mr-1 shadow-xl bg-gray-800 w-40 rounded overflow-hidden">
        <List
          items={info ? info.files : []}
          title="Files"
          keyer={file => file.file_id}
          render={file => getFileName(files, info!.id, file) || '???'}
          select={selectFile}
          isSelected={file =>
            (selectedFile && selectedFile.file_id == file.file_id) || false
          }
          reorder={(file, from, to) => {
            if (info)
              send({
                type: 'Project',
                id: info.id,
                msg: {
                  type: 'Reorder',
                  id: file,
                  new_index: to,
                },
              })
          }}
          footer="+ New File"
        />
      </div>
      <div className="flex flex-1 justify-evenly">
        <div
          style={{ transition: 'all 200ms ease' }}
          className={`flex rounded overflow-hidden ${
            f || !animate ? 'w-1/2 mx-2' : 'w-0 mx-0 opacity-0'
          } flex-col max-w-3xl shadow-xl bg-gray-800 my-2`}
        >
          <div className="flex flex-1 relative">
            <div className="flex flex-1 absolute inset-0">
              <MonacoEditor
                editorDidMount={e => {
                  setEditor(e)
                }}
                value={f ? f.src : ''}
                options={{
                  lineNumbers: 'off',
                  language: 'markdown',
                  minimap: {
                    enabled: false,
                  },
                  wordWrap: 'on',
                  glyphMargin: false,
                  folding: false,
                  // Undocumented see https://github.com/Microsoft/vscode/issues/30795#issuecomment-410998882
                  lineDecorationsWidth: 0,
                  lineNumbersMinChars: 0,
                }}
                onChange={value => {
                  if (selectedFile && info) {
                    editFile(selectedFile, value)
                  }
                }}
                theme="solarized-dark"
              />
            </div>
          </div>
        </div>
        <div className="flex flex-1 max-w-3xl mx-2 shadow-xl bg-gray-800 overflow-y-auto p-4 my-2 rounded">
          <div className="flex flex-1 relative">
            <div className="flex flex-1 markdown absolute inset-0">
              {info &&
                info.files.map(fileId => {
                  const projectId = info.id
                  const fileInfo = fileInfos[fileId.file_id]
                  const file = files && files[fileId.file_id]
                  if (!fileInfo) return null
                  return (
                    <DocListener
                      key={fileId.file_id}
                      projectId={projectId}
                      fileId={fileInfo.id}
                      send={send}
                      doc={(file && file.doc) || []}
                    />
                  )
                })}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

const DocListener: React.SFC<{
  projectId: ProjectId
  fileId: FileId
  doc: Doc
  send: (msg: Client2Server) => any
}> = ({ projectId, fileId, doc, send }) => {
  React.useEffect(() => {
    send({
      type: 'Project',
      id: projectId,
      msg: {
        type: 'File',
        id: fileId,
        msg: {
          type: 'JoinFileDoc',
        },
      },
    })

    return () => {
      send({
        type: 'Project',
        id: projectId,
        msg: {
          type: 'File',
          id: fileId,
          msg: {
            type: 'LeaveFileDoc',
          },
        },
      })
    }
  }, [fileId.file_id])

  return <Render src={doc} staticUrl={s => s}></Render>
}
