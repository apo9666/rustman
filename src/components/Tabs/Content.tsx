import * as Tabs from '@radix-ui/react-tabs'

interface TabsContentProps {
  children?: React.ReactNode
  index: number
}

const TabsContent: React.FC<TabsContentProps> = ({ children, index }) => {
  return (
    <Tabs.Content className="flex-grow flex-shrink-0 overflow-y-auto" value={index.toString()}>
      {children}
    </Tabs.Content>
  )
}

export default TabsContent
