import { useContext } from 'react'
import * as Collapsible from '@radix-ui/react-collapsible'
import { ChevronRight, ChevronDown, Folder, ArrowRight } from 'lucide-react'
import { TreeContext, type TreeNode } from '../../context/TreeContext'
import { TabContext } from '../../context/TabContext'

interface TreeDirectoryProps {
  node: TreeNode
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
  const { dispatch } = useContext(TreeContext)
  const { dispatch: tabDispatch } = useContext(TabContext)

  const handleOpenChange = (open: boolean): void => {
    dispatch({
      type: 'EDIT_NODE',
      payload: {
        node: {
          ...node,
          expanded: open
        }
      }
    })
  }

  const handleOnClick = (): void => {
    if (node.content !== undefined) {
      tabDispatch({
        type: 'ADD_TAB',
        payload: {
          label: node.label,
          content: node.content
        }
      })
    }
  }

  if (node.children !== undefined) {
    return (
      <Collapsible.Root open={node.expanded} onOpenChange={handleOpenChange}>
        <a className="flex items-center gap-1 px-5 py-1">
          <span className="flex items-center w-10">
            <Collapsible.Trigger asChild>
              {node.expanded ? (<ChevronDown />) : (<ChevronRight />)}
            </Collapsible.Trigger>
            <Folder className="ml-3" />
          </span>
          <span className="truncate hover:cursor-default">{node.label}</span>
        </a>
        <Collapsible.Content className="pl-3">
          {node.children.map(child => <TreeDirectory key={child.id} node={child} />)}
        </Collapsible.Content>
      </Collapsible.Root>
    )
  }
  return (
    <a className="flex items-center gap-1 px-5 py-1" onClick={handleOnClick}>
      {/* <span className="flex items-center">
        <span className={`text-xs ${colors[node.label] !== undefined ? 'text-' + colors[node.label] + '-600' : ''} w-10 text-right`}>{short[node.label] ?? node.label}</span>
      </span> */}
      <span className="w-96 truncate hover:cursor-default" title={node.label}>{node.label}</span>
      <ArrowRight />
    </a>
  )
}

export default TreeDirectory
