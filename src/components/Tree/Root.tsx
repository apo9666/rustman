interface TreeRootProps {
  children?: React.ReactNode
}

const TreeRoot: React.FC<TreeRootProps> = ({ children }) => {
  return (
    <nav className="overflow-y-auto">
      {children}
    </nav>
  )
}

export default TreeRoot
