import { type ChangeEvent, type FormEvent } from 'react'
import { ChevronDown } from 'lucide-react'
import * as Select from '@radix-ui/react-select'
import { useHookstate, type State, type ImmutableArray } from '@hookstate/core'
import { Body, ResponseType, getClient, type Response } from '@tauri-apps/api/http'
import { type Header, MethodEnum, type TabContent, type Param } from '../../state'

interface RequestUrlProps {
  content: State<TabContent>
  index: number
}

const convertHeaders = (headers: ImmutableArray<Header>): Record<string, any> | undefined => {
  const enabledHeaders = headers.filter(({ enable, key }) => enable && key !== '')

  if (enabledHeaders.length === 0) {
    return undefined
  }

  return enabledHeaders.reduce((prev, curr) => ({
    ...prev,
    [curr.key]: curr.value
  }), {})
}

const RequestUrl: React.FC<RequestUrlProps> = (props) => {
  const content = useHookstate(props.content)

  const request = async (): Promise<void> => {
    const client = await getClient()
    let response: Response<string> = {
      data: '',
      headers: {},
      ok: true,
      rawHeaders: {},
      status: 200,
      url: ''
    }
    const url = content.url.get()
    const body = content.body.get()
    switch (content.method.get()) {
      case MethodEnum.GET:
        response = await client.get<string>(url, {
          timeout: 30,
          responseType: ResponseType.Text,
          headers: convertHeaders(content.headers.get())
        })
        break
      case MethodEnum.POST:
        response = await client.post<string>(
          url,
          Body.json(JSON.parse(body)),
          {
            responseType: ResponseType.Text,
            headers: convertHeaders(content.headers.get())
          }
        )
        break
      case MethodEnum.PUT:
        response = await client.put<string>(
          url,
          Body.json(JSON.parse(body)),
          {
            responseType: ResponseType.Text,
            headers: convertHeaders(content.headers.get())
          }
        )
        break
      case MethodEnum.PATCH:
        response = await client.post<string>(
          url,
          Body.json(JSON.parse(body)),
          {
            responseType: ResponseType.Text,
            headers: convertHeaders(content.headers.get())
          }
        )
        break
      case MethodEnum.DELETE:
        response = await client.delete<string>(url, {
          timeout: 30,
          responseType: ResponseType.Text,
          headers: convertHeaders(content.headers.get())
        })
        break
      case MethodEnum.OPTIONS:
        response = await client.request<string>({
          timeout: 30,
          responseType: ResponseType.Text,
          method: 'OPTIONS',
          url,
          headers: convertHeaders(content.headers.get())
        })
        break
    }
    console.log(response)

    content.response.set({
      url: response.url,
      status: response.status,
      ok: response.ok,
      headers: response.headers,
      rawHeaders: response.rawHeaders,
      data: response.data
    })
  }

  const handleMethodChange = (value: MethodEnum): void => {
    content.method.set(value)
  }

  const handleUrlChange = (e: ChangeEvent<HTMLInputElement>): void => {
    content.url.set(e.target.value)

    try {
      const newUrl = new URL(content.url.get())
      content.params.set(params => params.filter(param => !param.enable))

      const params: Param[] = []
      newUrl.searchParams.forEach((value, key) => {
        params.push({
          enable: true,
          key,
          value
        })
      })

      content.params.merge([
        ...params,
        {
          enable: true,
          key: '',
          value: ''
        }
      ])
    } catch (e) { /* empty */ }
  }

  const handleSend = (e: FormEvent<HTMLFormElement>): void => {
    e.preventDefault()
    request().catch(error => { console.error(error) })
  }

  return (
    <form className="p-2 flex" onSubmit={handleSend}>
      <Select.Root defaultValue={content.method.get()} onValueChange={handleMethodChange}>
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
                <Select.Item className="hover:bg-zinc-700 px-2 py-1 text-sm cursor-pointer" value={MethodEnum.GET}>
                  <Select.ItemText>GET</Select.ItemText>
                </Select.Item>
                <Select.Item className="hover:bg-zinc-700 px-2 py-1 text-sm cursor-pointer" value={MethodEnum.POST}>
                  <Select.ItemText>POST</Select.ItemText>
                </Select.Item>
                <Select.Item className="hover:bg-zinc-700 px-2 py-1 text-sm cursor-pointer" value={MethodEnum.PUT}>
                  <Select.ItemText>PUT</Select.ItemText>
                </Select.Item>
                <Select.Item className="hover:bg-zinc-700 px-2 py-1 text-sm cursor-pointer" value={MethodEnum.PATCH}>
                  <Select.ItemText>PATCH</Select.ItemText>
                </Select.Item>
                <Select.Item className="hover:bg-zinc-700 px-2 py-1 text-sm cursor-pointer" value={MethodEnum.DELETE}>
                  <Select.ItemText>DELETE</Select.ItemText>
                </Select.Item>
                <Select.Item className="hover:bg-zinc-700 px-2 py-1 text-sm cursor-pointer" value={MethodEnum.OPTIONS}>
                  <Select.ItemText>OPTIONS</Select.ItemText>
                </Select.Item>
              </Select.Group>
            </Select.Viewport>
          </Select.Content>
        </Select.Portal>
      </Select.Root>
      <input className="flex-grow bg-zinc-700 rounded-e-md pl-2" placeholder="Enter request URL" value={content.url.get()} onChange={handleUrlChange} />
      <button type="submit" className="bg-blue-500 ml-2 p-2 rounded-md font-bold hover:bg-blue-600">Send</button>
    </form >
  )
}

export default RequestUrl
