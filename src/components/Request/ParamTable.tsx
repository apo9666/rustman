import { useState } from 'react'
import { ArrowBigDown, ArrowBigUp } from 'lucide-react'
import ParamRow from './ParamRow'
import { type State, useHookstate } from '@hookstate/core'

export interface Param {
  id: number
  enable: boolean
  key: string
  value: string
}

const emptyParam: Param = {
  id: 0,
  enable: true,
  key: '',
  value: ''
}

interface ParamsProps {
  url: State<string>
}

const Params: React.FC<ParamsProps> = (props) => {
  const url = useHookstate(props.url)
  const [params, setParams] = useState<Param[]>([emptyParam])
  const [lastParamId, setLastParamId] = useState(0)

  const addParam = (): void => {
    const newParam = {
      ...emptyParam,
      id: lastParamId + 1
    }

    setParams(prevParams => [...prevParams, newParam])
    setLastParamId((id) => id + 1)
  }

  const saveParam = (editedParam: Param): void => {
    setParams(prevParams => prevParams.map(
      param => param.id === editedParam.id ? editedParam : param
    ))
  }

  const removeParam = (removedParam: Param): void => {
    setParams(
      prevParams => prevParams.filter(param => param.id !== removedParam.id)
    )
  }

  const importParams = (): void => {
    if (url.get().trim() === '') {
      return
    }

    try {
      const newUrl = new URL(url.get())
      let count = lastParamId + 1
      const newParams: Param[] = []

      newUrl.searchParams.forEach((value, key) => {
        newParams.push({
          id: count,
          enable: true,
          key,
          value
        })

        count++
      })

      newParams.push({
        ...emptyParam,
        id: count
      })

      setParams(newParams)
      setLastParamId(count)
    } catch (e) { /* empty */ }
  }

  const exportParams = (): void => {
    if (url.get().trim() === '') {
      return
    }

    try {
      const newUrl = new URL(url.get().split('?')[0])

      params
        .filter(param => param.key.trim() !== '' || !param.enable)
        .forEach(param => { newUrl.searchParams.set(param.key, param.value) })

      url.set(newUrl.toString())
    } catch (e) { /* empty */ }
  }

  return (
    <>
      <h2 className="mx-5 my-2 text-zinc-400">
        Query Params
        <ArrowBigDown className="inline cursor-pointer text-zinc-200 rounded-sm bg-blue-500 ml-2 pr-[1px] h-[16px] w-[16px] hover:bg-blue-600" onClick={importParams} size={14} />
        <ArrowBigUp className="inline cursor-pointer text-zinc-200 rounded-sm bg-blue-500 ml-2 pr-[1px] h-[16px] w-[16px] hover:bg-blue-600" onClick={exportParams} size={14} />
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
          {params.map((param) => (
            <ParamRow
              key={param.id}
              param={param}
              lastParamId={lastParamId}
              addParam={addParam}
              saveParam={saveParam}
              removeParam={removeParam} />
          ))}
        </tbody>
      </table>
    </>
  )
}

export default Params
