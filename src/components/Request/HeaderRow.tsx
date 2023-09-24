import { type ChangeEvent } from 'react'
import { Trash2 } from 'lucide-react'
import { type Header } from '../../state'
import { type State, useHookstate } from '@hookstate/core'

interface HeaderRowProps {
  index: number
  header: State<Header>
  addHeader: () => void
  removeHeader: (removeIndex: number) => void
  last: boolean
}

const HeaderRow: React.FC<HeaderRowProps> = (props) => {
  const state = useHookstate(props.header)

  const handleEnableChange = (e: ChangeEvent<HTMLInputElement>): void => {
    state.set(header => ({
      ...header,
      enable: e.target.checked
    }))
  }

  const handleKeyChange = (e: ChangeEvent<HTMLInputElement>): void => {
    state.set(header => ({
      ...header,
      key: e.target.value
    }))

    if (props.last) {
      props.addHeader()
    }
  }

  const handleValueChange = (e: ChangeEvent<HTMLInputElement>): void => {
    state.set(header => ({
      ...header,
      value: e.target.value
    }))

    if (props.last) {
      props.addHeader()
    }
  }

  return (
    <tr className="border-collapse">
      <td className="border-zinc-700 border p-2 text-center w-10" >
        <input type="checkbox" onChange={handleEnableChange} checked={state.enable.get()} />
      </td>
      <td className="border-zinc-700 border">
        <input type="text" className="bg-transparent w-full p-2" onChange={handleKeyChange} value={state.key.get()} />
      </td>
      <td className="border-zinc-700 border">
        <input type="text" className="bg-transparent w-full p-2" onChange={handleValueChange} value={state.nested('value').get()} />
      </td>
      <td className="border-zinc-700 border p-2 text-center w-10">
        {!props.last &&
          <button onClick={() => { props.removeHeader(props.index) }}>
            <Trash2 size={14} className="cursor-pointer hover:text-zinc-500" />
          </button>
        }
      </td>
    </tr>
  )
}

export default HeaderRow
