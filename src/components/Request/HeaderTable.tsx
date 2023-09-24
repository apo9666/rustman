import { type State, useHookstate } from '@hookstate/core'
import HeaderRow from './HeaderRow'
import { type Header, headerInitialState } from '../../state'

interface HeaderTableProps {
  headers: State<Header[]>
}

const HeaderTable: React.FC<HeaderTableProps> = (props) => {
  const state = useHookstate(props.headers)

  const addHeader = (): void => {
    state.merge(
      [headerInitialState]
    )
  }

  const removeHeader = (removeIndex: number): void => {
    state.set(headers => headers.filter((_, index) => index !== removeIndex))
  }

  return (
    <>
      <h2 className="mx-5 my-2 text-zinc-400">
        Headers
      </h2>
      <table className="table-auto w-full">
        <thead>
          <tr>
            <th className="border-zinc-700 border p-2"></th>
            <th className="border-zinc-700 border p-2">KEY</th>
            <th className="border-zinc-700 border p-2">VALUE</th>
            <th className="border-zinc-700 border p-2">REMOVE</th>
          </tr>
        </thead>
        <tbody>
          {state.map((header, index) => (
            <HeaderRow
              key={index}
              index={index}
              header={header}
              addHeader={addHeader}
              removeHeader={removeHeader}
              last={state.length === index + 1} />
          ))}
        </tbody>
      </table>
    </>
  )
}

export default HeaderTable
