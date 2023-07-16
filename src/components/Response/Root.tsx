interface ResponseRootProps {
  children: React.ReactNode
}

const ResponseRoot: React.FC<ResponseRootProps> = ({ children }) => {
  return (
    <div className="flex-grow flex flex-col">
      {children}
    </div>
  )
}

export default ResponseRoot
