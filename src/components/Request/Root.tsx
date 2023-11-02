interface RequestRootProps {
  children: React.ReactNode
}

const RequestRoot: React.FC<RequestRootProps> = ({ children }) => {
  return (
    <div className="flex-grow flex flex-col max-h-[50%] overflow-y-auto">
      {children}
    </div>
  )
}

export default RequestRoot
