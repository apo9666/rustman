import { type ChangeEvent } from 'react'
import { Trash2 } from 'lucide-react'

export interface Param {
  id: number
  enable: boolean
  key: string
  value: string
}

interface ParamRowProps {
  param: Param
  lastParamId: number
  addParam: () => void
  removeParam: (param: Param) => void
  saveParam: (param: Param) => void
}

const ParamRow: React.FC<ParamRowProps> = ({
  param,
  lastParamId,
  addParam,
  removeParam,
  saveParam
}) => {
  const handleEnableChange = (e: ChangeEvent<HTMLInputElement>): void => {
    saveParam({
      ...param,
      enable: e.target.checked
    })
  }

  const handleKeyChange = (e: ChangeEvent<HTMLInputElement>): void => {
    saveParam({
      ...param,
      key: e.target.value
    })

    if (param.id === lastParamId) {
      addParam()
    }
  }

  const handleValueChange = (e: ChangeEvent<HTMLInputElement>): void => {
    saveParam({
      ...param,
      value: e.target.value
    })

    if (param.id === lastParamId) {
      addParam()
    }
  }

  return (
    <tr className="border-collapse">
      <td className="border-zinc-700 border p-2 text-center w-10" >
        <input type="checkbox" onChange={handleEnableChange} checked={param.enable} />
      </td>
      <td className="border-zinc-700 border">
        <input type="text" className="bg-transparent w-full p-2" onChange={handleKeyChange} value={param.key} />
      </td>
      <td className="border-zinc-700 border">
        <input type="text" className="bg-transparent w-full p-2" onChange={handleValueChange} value={param.value} />
      </td>
      <td className="border-zinc-700 border p-2 text-center w-10">
        <button onClick={() => { removeParam(param) }}>
          <Trash2 size={14} className="cursor-pointer hover:text-zinc-500" />
        </button>
      </td>
    </tr>
  )
}

export default ParamRow
