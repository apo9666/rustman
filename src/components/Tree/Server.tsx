import { useState } from 'react'
import * as Collapsible from '@radix-ui/react-collapsible'
import { ChevronRight, ChevronDown, ArrowRight, Server } from 'lucide-react'
import { useHookstate, type State } from '@hookstate/core'
import { type ServerObject, type PathsObject } from 'openapi3-ts/oas31'

const colors: Record<string, string> = {
  GET: 'green',
  POST: 'yellow',
  PUT: 'blue',
  DELETE: 'red'
}

const short: Record<string, string> = {
  DELETE: 'DEL'
}

export interface TreeServerProps {
  serverState: State<ServerObject>
  pathsState: State<PathsObject | undefined>
}

const TreeServer: React.FC<TreeServerProps> = ({ serverState, pathsState }) => {
  const [expanded, setExpanded] = useState(false)
  const server = useHookstate(serverState)
  const paths = useHookstate(pathsState)

  const handleOpenChange = (open: boolean): void => {
    setExpanded(open)
  }

  const handleOnClick = (): void => {

  }

  return (
    <Collapsible.Root open={expanded} onOpenChange={handleOpenChange}>
      <a className="flex items-center gap-1 px-5 py-1">
        <span className="flex items-center w-10">
          <Collapsible.Trigger asChild>
            {expanded ? (<ChevronDown />) : (<ChevronRight />)}
          </Collapsible.Trigger>
          <Server className="ml-3" />
        </span>
        <span className="truncate hover:cursor-default">{server.get().url}</span>
      </a>
      <Collapsible.Content className="pl-3">
        {Object.entries(paths.ornull ?? {}).map(([key, value]) =>
          <a className="flex items-center gap-1 px-5 py-1" onClick={handleOnClick}>
            {/* <span className="flex items-center">
      <span className={`text-xs ${colors[node.label] !== undefined ? 'text-' + colors[node.label] + '-600' : ''} w-10 text-right`}>{short[node.label] ?? node.label}</span>
    </span> */}
            <span className="w-96 truncate hover:cursor-default" title={key}>{key}</span>
            <ArrowRight />
          </a>
        )}
      </Collapsible.Content>
    </Collapsible.Root>
  )
}

export default TreeServer
