import AceEditor from 'react-ace'
import { Braces } from 'lucide-react'
import { type State, useHookstate } from '@hookstate/core'

import 'ace-builds/src-noconflict/mode-json'
import 'ace-builds/src-noconflict/theme-twilight'
import 'ace-builds/src-noconflict/ext-language_tools'

interface ResponseContentProps {
  data: State<string>
}

const ResponseContent: React.FC<ResponseContentProps> = (props) => {
  const state = useHookstate(props.data)

  const format = (): void => {
    if (state.get() === undefined) {
      return
    }
    try {
      state.set(data => JSON.stringify(
        JSON.parse(data),
        null,
        2
      ))
    } catch (e) { /* empty */ }
  }

  return (
    <div className="flex-grow flex flex-col">
      <hr className="border-zinc-600" />
      <h2 className="mx-5 mt-2 text-zinc-400">Response <Braces className="inline cursor-pointer text-zinc-200 rounded-sm bg-blue-500 ml-2 mb-[2px] p-[1px] hover:bg-blue-600" onClick={format} size={14} /></h2>
      <div className="flex-grow m-2 border border-zinc-600">
        <AceEditor
          mode="json"
          theme="twilight"
          name="UNIQUE_ID_OF_DIV"
          value={state.get()}
          editorProps={{ $blockScrolling: true }}
          width="100%"
          height="100%"
          setOptions={{
            useWorker: false
          }}
        />
      </div>
    </div>
  )
}

export default ResponseContent
