import { type ChangeEvent } from 'react'
import { Trash2 } from 'lucide-react'
import { useHookstate, type State } from '@hookstate/core'
import { type Param } from '../../state'

interface ParamRowProps {
  index: number
  param: State<Param>
  addParam: () => void
  removeParam: (removeIndex: number) => void
  changeParam: () => void
  last: boolean
}

const ParamRow: React.FC<ParamRowProps> = ({
  index,
  param,
  addParam,
  removeParam,
  changeParam,
  last
}) => {
  const state = useHookstate(param)

  const handleEnableChange = (e: ChangeEvent<HTMLInputElement>): void => {
    state.enable.set(e.target.checked)
    changeParam()

    if (last) {
      addParam()
    }
  }

  const handleKeyChange = (e: ChangeEvent<HTMLInputElement>): void => {
    state.key.set(e.target.value)
    changeParam()

    if (last) {
      addParam()
    }
  }

  const handleValueChange = (e: ChangeEvent<HTMLInputElement>): void => {
    state.nested('value').set(e.target.value)
    changeParam()

    if (last) {
      addParam()
    }
  }

  return (
    <tr className="border-collapse">
      <td className="border-zinc-700 border p-2 text-center w-10" >
        <input type="checkbox" onChange={handleEnableChange} checked={param.enable.get()} />
      </td>
      <td className="border-zinc-700 border">
        <input type="text" className="bg-transparent w-full p-2" onChange={handleKeyChange} value={param.key.get()} />
      </td>
      <td className="border-zinc-700 border">
        <input type="text" className="bg-transparent w-full p-2" onChange={handleValueChange} value={param.nested('value').get()} />
      </td>
      <td className="border-zinc-700 border p-2 text-center w-10">
        {!last &&
          <button onClick={() => { removeParam(index) }}>
            <Trash2 size={14} className="cursor-pointer hover:text-zinc-500" />
          </button>
        }
      </td>
    </tr>
  )
}

export default ParamRow
