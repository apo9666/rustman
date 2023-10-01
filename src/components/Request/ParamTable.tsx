import { type State, useHookstate } from '@hookstate/core'
import ParamRow from './ParamRow'
import { type Param } from '../../state'

interface ParamsProps {
  url: State<string>
  params: State<Param[]>
}

const Params: React.FC<ParamsProps> = (props) => {
  const urlState = useHookstate(props.url)
  const paramsState = useHookstate(props.params)

  const addParam = (): void => {
    paramsState.merge([
      {
        enable: true,
        key: '',
        value: ''
      }
    ])
  }

  const removeParam = (removeIndex: number): void => {
    paramsState.set(
      params => params.filter((_, index) => index !== removeIndex)
    )
    changeParam()
  }

  const changeParam = (): void => {
    try {
      const searchParams = new URLSearchParams(
        paramsState.get().filter(
          (param) => (param.key.trim() !== '' && param.enable)
        ).map(param => ([param.key, param.value]))
      )

      urlState.set(`${urlState.get().split('?')[0]}?${searchParams.toString()}`)
    } catch (e) { /* empty */ }
  }

  return (
    <>
      <h2 className="mx-5 my-2 text-zinc-400">
        Query Params
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
          {paramsState.map((param, index) => (
            <ParamRow
              key={index}
              index={index}
              param={param}
              addParam={addParam}
              changeParam={changeParam}
              removeParam={removeParam}
              last={paramsState.length === index + 1} />
          ))}
        </tbody>
      </table>
    </>
  )
}

export default Params
