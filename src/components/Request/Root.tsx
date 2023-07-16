
interface RequestRootProps {
  children: React.ReactNode
}

const RequestRoot: React.FC<RequestRootProps> = ({ children }) => {
  return (
    <div className="flex-grow flex flex-col">
      {children}
    </div>
  )
}

export default RequestRoot
