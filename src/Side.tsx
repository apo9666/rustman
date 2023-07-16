import Tree from './components/Tree'
import { useContext } from 'react'
import { TreeContext } from './context/TreeContext'

const Side: React.FC = () => {
  const { state } = useContext(TreeContext)

  return (
    <Tree.Root>
      <Tree.Directory node={state.tree} />
    </Tree.Root>
  )
}

export default Side
