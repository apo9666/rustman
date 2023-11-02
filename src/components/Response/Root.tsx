interface ResponseRootProps {
  children: React.ReactNode
}

const ResponseRoot: React.FC<ResponseRootProps> = ({ children }) => {
  return (
    <div className="flex-grow flex flex-col min-h-[50%]">
      {children}
    </div>
  )
}

export default ResponseRoot
