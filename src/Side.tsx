import Tree from './components/Tree'
import { useHookstate } from '@hookstate/core'
import { treeState } from './state'

const Side: React.FC = () => {
  const state = useHookstate(treeState)

  return (
    <Tree.Root>
      <Tree.Directory node={state.tree} />
    </Tree.Root>
  )
}

export default Side
