import { useEffect, useState } from 'react'
import { readTextFile, writeFile } from '@tauri-apps/api/fs'
import { save, open } from '@tauri-apps/api/dialog'
import { listen } from '@tauri-apps/api/event'
import YAML from 'yaml'
import { OpenApiBuilder, type PathItemObject, type RequestBodyObject, type ReferenceObject } from 'openapi3-ts/oas31'
import Section from './Section'
import Side from './Side'
import { MethodEnum, type TabContent, treeState } from './state'
import { useHookstate } from '@hookstate/core'

const containBody = (body?: RequestBodyObject | ReferenceObject): body is RequestBodyObject => {
  if (body === undefined) {
    return false
  }

  if ((body as RequestBodyObject).content === undefined) {
    return false
  }

  return true
}

const getBody = (body?: RequestBodyObject | ReferenceObject): string => {
  if (!containBody(body)) {
    return ''
  }

  try {
    return JSON.stringify(
      body.content['application/json']?.example,
      null,
      2
    ) ?? ''
  } catch (e) {
    console.log(e)
    return ''
  }
}

const convertContent = (
  server: string,
  key: string,
  path: PathItemObject
): TabContent => {
  let method = MethodEnum.GET
  let body = ''
  if (path.get !== undefined) {
    method = MethodEnum.GET
  } else if (path.post !== undefined) {
    const { requestBody } = path.post
    body = getBody(requestBody)
    method = MethodEnum.POST
  } else if (path.put !== undefined) {
    const { requestBody } = path.put
    body = getBody(requestBody)
    method = MethodEnum.PUT
  } else if (path.patch !== undefined) {
    const { requestBody } = path.patch
    body = getBody(requestBody)
    method = MethodEnum.PATCH
  } else if (path.delete !== undefined) {
    method = MethodEnum.DELETE
  } else if (path.options !== undefined) {
    method = MethodEnum.OPTIONS
  } else if (path.head !== undefined) {
    method = MethodEnum.HEAD
  } else if (path.trace !== undefined) {
    method = MethodEnum.TRACE
  }

  return {
    url: server.replace(/\/$/, '') + key,
    method,
    body,
    headers: [
      {
        enable: true,
        key: '',
        value: ''
      }
    ],
    response: {
      data: '',
      headers: {},
      ok: true,
      rawHeaders: {},
      status: 200,
      url: ''
    }
  }
}

const App: React.FC = () => {
  const state = useHookstate(treeState)
  const [menuPayload, setMenuPayload] = useState('')
  const [menuOpen, setMenuOpen] = useState(false)

  useEffect(() => {
    listen('menu-event', (e) => {
      console.log(e.payload)
      setMenuPayload(e.payload)
      setMenuOpen(true)
    }).catch(console.log)
  }, [])

  const OpenFile = async () => {
    try {
      const filepath = await open()
      const content = await readTextFile(filepath)
      const a = new OpenApiBuilder(YAML.parse(content))
      const o = a.getSpec()
      const keys = Object.entries(o.paths)
        .sort(([a], [b]) => a.localeCompare(b))
      console.log(o)

      if (o.servers === undefined) {
        console.log('no servers')
        return
      }

      state.tree.children.merge([
        {
          expanded: true,
          label: o.info.title,
          children: o.servers?.map(server => ({
            expanded: false,
            label: server.url,
            children: keys.map(([key, value]) => ({
              expanded: false,
              label: key,
              content: convertContent(server.url, key, value)
            }))
          }))
        }
      ])
    } catch (e) {
      console.log(e)
    }
  }

  const SaveFile = async (text) => {
    try {
      const filepath = await save()
      await writeFile({ contents: text, path: filepath })
    } catch (e) {
      console.log(e)
    }
  }

  useEffect(() => {
    if (menuOpen) {
      switch (menuPayload) {
        case 'open-event':
          OpenFile()
          break
        case 'save-event':
          SaveFile()
          break

        default:
          break
      }
      setMenuOpen(false)
    }
  }, [menuOpen])

  return (
    <div className="h-screen flex">
      <aside className="flex-shrink-0 w-72 text-sm bg-local bg-zinc-800 overflow-y-auto">
        <Side />
      </aside>
      <main className="flex-grow bg-zinc-900 overflow-y-auto">
        <Section />
      </main>
    </div>
  )
}

export default App
