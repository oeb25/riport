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
import { Client2Server, Client2ServerProjectFile } from '../com/c2s'
import { getFileName } from '../state'
import { List } from './List'
import {
  joinFileSource,
  leaveFileSource,
  joinProject,
  leaveProject,
  reorderFiles,
  leaveFileDoc,
  joinFileDoc,
} from '../com/actions'
import { Send } from '../com/socket'

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
  fileInfos: { [fileId: number]: FileInfo }
  files: ProjectFiles
  send: Send
  selectedFile: FileId | null
  editFile: (id: FileId, value: string) => any
  selectConfig: () => any
  selectFile: (id: FileId) => any
}> = ({ fileInfos, files, send, selectedFile, selectFile, editFile }) => {
  const info = React.useContext(ProjectInfoContext)

  if (!info) return <div>Loading...</div>

  const f =
    files && selectedFile && selectedFile.file_id in files
      ? files[selectedFile.file_id]
      : null

  const animate = true

  return (
    <div className="flex flex-1 h-full">
      <div className="flex flex-col my-2 ml-2 mr-1 shadow-xl bg-gray-800 w-40 rounded overflow-hidden">
        <List
          items={info.files}
          title="Files"
          keyer={file => file.file_id}
          render={file => getFileName(files, info.id, file) || '???'}
          select={selectFile}
          isSelected={file =>
            (selectedFile && selectedFile.file_id == file.file_id) || false
          }
          reorder={(file, _, to) => {
            if (info) send(reorderFiles(info.id, file, to))
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
            {selectedFile && (
              <Editor
                fileId={selectedFile}
                src={f ? f.src : ''}
                onChange={value => {
                  if (selectedFile) {
                    editFile(selectedFile, value)
                  }
                }}
                send={send}
              />
            )}
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
                      fileId={fileId}
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
  send: Send
}> = ({ projectId, fileId, doc, send }) => {
  React.useEffect(() => {
    send(joinFileDoc(projectId, fileId))

    return () => {
      send(leaveFileDoc(projectId, fileId))
    }
  }, [projectId.project_id, fileId.file_id])

  return (
    <Render
      src={doc}
      fileId={fileId}
      staticUrl={s =>
        `/artifacts/${[projectId.project_id]}/${fileId.file_id}/${s}`
      }
    ></Render>
  )
}

export const ProjectInfoContext = React.createContext<ProjectInfo | null>(null)

const Editor: React.FC<{
  fileId: FileId
  src: string
  onChange: (value: string) => any
  send: Send
}> = ({ fileId, src, onChange, send }) => {
  const info = React.useContext(ProjectInfoContext)!

  React.useEffect(() => {
    send(joinFileSource(info.id, fileId))
    return () => send(leaveFileSource(info.id, fileId))
  }, [info.id.project_id, fileId.file_id])

  const [
    editor,
    setEditor,
  ] = React.useState<null | editor.IStandaloneCodeEditor>(null)

  React.useEffect(() => {
    if (info) {
      send(joinProject(info.id))
      return () => send(leaveProject(info.id))
    }
  }, [info && info.id.project_id])

  React.useEffect(() => {
    let stop = false

    const loop = () => {
      if (stop) return
      if (editor) {
        editor.layout()
      }
      requestAnimationFrame(loop)
    }

    const t = setTimeout(() => {
      stop = true
      // TODO: find appropriate number for this delay
    }, 2000)

    loop()

    return () => {
      clearTimeout(t)
      stop = true
    }
  }, [editor, fileId.file_id])

  React.useEffect(() => {
    const resize = () => {
      if (editor) editor.layout()
    }
    window.addEventListener('resize', resize)
    return () => window.removeEventListener('resize', resize)
  }, [editor])

  return (
    <div className="flex flex-1 absolute inset-0">
      <MonacoEditor
        editorDidMount={setEditor}
        value={src}
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
        onChange={onChange}
        theme="solarized-dark"
      />
    </div>
  )
}

const ConfigWindow: React.FC<{}> = ({}) => <div></div>
