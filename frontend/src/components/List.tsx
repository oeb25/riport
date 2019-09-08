import * as React from 'react'

export function List<T>(
  props: React.Props<{}> & {
    title: React.ReactNode
    items: T[]
    keyer: (t: T, i: number) => string | number
    render: (t: T, i: number) => React.ReactNode
    select: (t: T, i: number) => any
    isSelected: (t: T, i: number) => boolean
    reorder: (t: T, from: number, to: number) => any
    footer?: React.ReactNode
  },
) {
  const [drag, setDrag] = React.useState<null | number>(null)
  const [dragOver, setDragOver] = React.useState<null | number>(null)

  return (
    <div className="bg-gray-900 flex flex-col flex-1 shadow flex-shrink w-full max-w-md rounded overflow-hidden">
      <div className="flex p-2 border-b text-gray-500">
        <p className="flex flex-col flex-1 text-gray-500">{props.title}</p>
        <a
          className="pl-3 pr-1 hover:text-white"
          href="/"
          onClick={e => {
            e.preventDefault()
          }}
        >
          +
        </a>
      </div>
      <div className="flex flex-1 flex-col bg-gray-800">
        <div className="flex flex-1 flex-col">
          {props.items.map((item, i) => (
            <a
              onDrag={() => {
                // console.log("onDrag", e);
                setDrag(i)
              }}
              onDragEnd={() => {
                if (drag != null && dragOver != null && drag != dragOver) {
                  props.reorder(props.items[drag], drag, dragOver)
                }
              }}
              key={props.keyer(item, i)}
              href="/"
              className={`flex relative py-1 border-b px-2 last:border-b-0 border-gray-600 items-center hover:bg-gray-700 ${
                props.isSelected(item, i) ? 'bg-gray-700' : ''
              }`}
              onClick={e => {
                e.preventDefault()
                props.select(item, i)
              }}
            >
              <div className="flex flex-1">{props.render(item, i)}</div>
              <div className="flex flex-col absolute top-0 left-0 bottom-0 right-0">
                <div
                  className="flex flex-1"
                  onDragOver={e => {
                    // console.log("onDragOver TOP", i);
                    setDragOver(i)
                  }}
                ></div>
                <div
                  className="flex flex-1"
                  onDragOver={e => {
                    // console.log("onDragOver BOTTOM", i);
                    setDragOver(i + 1)
                  }}
                ></div>
              </div>
            </a>
          ))}
        </div>
        {props.footer && (
          <a
            href="/"
            className="flex p-2 bg-gray-900 text-gray-500 hover:bg-black hover:text-white"
            onClick={e => {
              e.preventDefault()
            }}
          >
            {props.footer}
          </a>
        )}
      </div>
    </div>
  )
}
