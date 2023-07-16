import AceEditor from 'react-ace'
import { Braces } from 'lucide-react'
import { useContext } from 'react'
import { TabContext } from '../../context/TabContext'

import 'ace-builds/src-noconflict/mode-json'
import 'ace-builds/src-noconflict/theme-twilight'
import 'ace-builds/src-noconflict/ext-language_tools'

interface BodyProps {
  id: number
  body: string
}

const Body: React.FC<BodyProps> = ({ id, body }) => {
  const { dispatch } = useContext(TabContext)
  const format = (): void => {
    try {
      dispatch({
        type: 'UPDATE_CONTENT',
        payload: {
          id,
          content: {
            body: JSON.stringify(
              JSON.parse(body),
              null,
              2
            )
          }
        }
      })
    } catch (e) { /* empty */ }
  }

  const handleChange = (newBody: string): void => {
    dispatch({
      type: 'UPDATE_CONTENT',
      payload: {
        id,
        content: {
          body: newBody
        }
      }
    })
  }

  return (
    <div className="h-full flex flex-col">
      <h2 className="mx-5 mt-2 text-zinc-400">Body <Braces className="inline cursor-pointer text-zinc-200 rounded-sm bg-blue-500 ml-2 mb-[2px] p-[1px] hover:bg-blue-600" onClick={format} size={14} /></h2>
      <div className="flex-grow m-2 border border-zinc-600">
        <AceEditor
          mode="json"
          theme="twilight"
          onChange={handleChange}
          name="UNIQUE_ID_OF_DIV"
          value={body}
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

export default Body
