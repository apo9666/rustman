import * as Tabs from '@radix-ui/react-tabs'
import { type Tab } from '../../context/TabContext'

interface TabsContentProps {
  tab: Tab
  children?: React.ReactNode
}

const TabsContent: React.FC<TabsContentProps> = ({ tab, children }) => {
  return (
    <Tabs.Content key={tab.id} className="flex-grow flex-shrink-0 overflow-y-auto" value={tab.id.toString()}>
      {children}
    </Tabs.Content>
  )
}

export default TabsContent
