import { useState } from 'react'
import { type TreeNode } from '../../context/TreeContext'

interface TreeBranchProps {
  branch: TreeNode
}

const TreeBranch: React.FC<TreeBranchProps> = ({ branch: { children, label } }) => {
  const [showChildren, setShowChildren] = useState(false)

  const handleClick = (): void => {
    setShowChildren(!showChildren)
  }
  return (
    <>
      <div onClick={handleClick} style={{ marginBottom: '10px' }}>
        <span>{label}</span>
      </div>
      <ul style={{ paddingLeft: '10px', borderLeft: '1px solid black' }}>
        {showChildren && children?.map(child => (<TreeBranch branch={child} />))}
      </ul>
    </>
  )
}

export default TreeBranch
