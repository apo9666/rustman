import { type ChangeEvent, useContext, type FormEvent } from 'react'
import { ChevronDown } from 'lucide-react'
import * as Select from '@radix-ui/react-select'
import { Body, ResponseType, getClient } from '@tauri-apps/api/http'
import { TabContext } from '../../context/TabContext'

interface RequestUrlProps {
  id: number
  method: string
  url: string
  body: string
}

const RequestUrl: React.FC<RequestUrlProps> = ({
  id,
  method,
  url,
  body
}) => {
  const { dispatch } = useContext(TabContext)
  console.log(url)

  const request = async (): Promise<void> => {
    const client = await getClient()
    switch (method) {
      case 'get': {
        const response = await client.get<string>(url, {
          timeout: 30,
          responseType: ResponseType.Text
        })
        dispatch({
          type: 'UPDATE_CONTENT',
          payload: {
            id,
            content: {
              response
            }
          }
        })
      } break
      case 'post': {
        const response = await client.post<string>(
          url,
          Body.json(JSON.parse(body)),
          {
            responseType: ResponseType.Text
          }
        )
        dispatch({
          type: 'UPDATE_CONTENT',
          payload: {
            id,
            content: {
              response
            }
          }
        })
      }
    }
  }

  const handleMethodChange = (value: string): void => {
    dispatch({
      type: 'UPDATE_CONTENT',
      payload: {
        id,
        content: {
          method: value
        }
      }
    })
  }

  const handleUrlChange = (e: ChangeEvent<HTMLInputElement>): void => {
    dispatch({
      type: 'UPDATE_CONTENT',
      payload: {
        id,
        content: {
          url: e.target.value
        }
      }
    })
  }

  const handleSend = (e: FormEvent<HTMLFormElement>): void => {
    console.log('chamou send')
    e.preventDefault()
    request().catch(error => { console.error(error) })
  }

  return (
    <form className="p-2 flex" onSubmit={handleSend}>
      <Select.Root defaultValue={method} onValueChange={handleMethodChange}>
        <Select.Trigger className="inline-flex items-center justify-center rounded-l-md p-2 gap-2 bg-zinc-800 hover:bg-zinc-700">
          <Select.Value />
          <Select.Icon>
            <ChevronDown size={18} />
          </Select.Icon>
        </Select.Trigger>
        <Select.Portal>
          <Select.Content className="overflow-hidden bg-zinc-800 rounded-l-md ">
            <Select.Viewport>
              <Select.Group>
                <Select.Item className="hover:bg-zinc-700 px-2 py-1 text-sm cursor-pointer" value="get">
                  <Select.ItemText>GET</Select.ItemText>
                </Select.Item>
                <Select.Item className="hover:bg-zinc-700 px-2 py-1 text-sm cursor-pointer" value="put">
                  <Select.ItemText>PUT</Select.ItemText>
                </Select.Item>
                <Select.Item className="hover:bg-zinc-700 px-2 py-1 text-sm cursor-pointer" value="post">
                  <Select.ItemText>POST</Select.ItemText>
                </Select.Item>
                <Select.Item className="hover:bg-zinc-700 px-2 py-1 text-sm cursor-pointer" value="delete">
                  <Select.ItemText>DELETE</Select.ItemText>
                </Select.Item>
              </Select.Group>
            </Select.Viewport>
          </Select.Content>
        </Select.Portal>
      </Select.Root>
      <input className="flex-grow bg-zinc-700 rounded-e-md pl-2" placeholder="Enter request URL" value={url} onChange={handleUrlChange} />
      <button type="submit" className="bg-blue-500 ml-2 p-2 rounded-md font-bold hover:bg-blue-600">Send</button>
    </form>
  )
}

export default RequestUrl
