import { useContext } from 'react'
import * as Collapsible from '@radix-ui/react-collapsible'
import { ChevronRight, ChevronDown, Folder, ArrowRight } from 'lucide-react'
import { type State, useHookstate } from '@hookstate/core'
import { type TreeNode, tabState } from '../../state'

interface TreeDirectoryProps {
  node: State<TreeNode>
}

const colors: Record<string, string> = {
  GET: 'green',
  POST: 'yellow',
  PUT: 'blue',
  DELETE: 'red'
}

const short: Record<string, string> = {
  DELETE: 'DEL'
}

const TreeDirectory: React.FC<TreeDirectoryProps> = ({ node }) => {
  const treeState = useHookstate(node)
  const tab = useHookstate(tabState)

  const handleOpenChange = (open: boolean): void => {
    treeState.expanded.set(open)
  }

  const handleOnClick = (): void => {
    const content = node.content.get()
    if (content == null) {
      return
    }

    tab.tabs.set(tabs => ([
      ...tabs,
      {
        id: tab.lastTabId.get() + 1,
        content: {
          ...content
        },
        label: node.label.get()
      }
    ]))
  }

  if (node.children.ornull !== undefined) {
    return (
      <Collapsible.Root open={node.expanded.get()} onOpenChange={handleOpenChange}>
        <a className="flex items-center gap-1 px-5 py-1">
          <span className="flex items-center w-10">
            <Collapsible.Trigger asChild>
              {node.expanded.get() ? (<ChevronDown />) : (<ChevronRight />)}
            </Collapsible.Trigger>
            <Folder className="ml-3" />
          </span>
          <span className="truncate hover:cursor-default">{node.label.get()}</span>
        </a>
        <Collapsible.Content className="pl-3">
          {node.children.ornull?.map(child => <TreeDirectory key={child.id.get()} node={child} />)}
        </Collapsible.Content>
      </Collapsible.Root>
    )
  }
  return (
    <a className="flex items-center gap-1 px-5 py-1" onClick={handleOnClick}>
      {/* <span className="flex items-center">
        <span className={`text-xs ${colors[node.label] !== undefined ? 'text-' + colors[node.label] + '-600' : ''} w-10 text-right`}>{short[node.label] ?? node.label}</span>
      </span> */}
      <span className="w-96 truncate hover:cursor-default" title={node.label.get()}>{node.label.get()}</span>
      <ArrowRight />
    </a>
  )
}

export default TreeDirectory
