import * as Tabs from '@radix-ui/react-tabs'
import { useContext } from 'react'
import { TabContext } from '../../context/TabContext'

interface TabsRootProps {
  children?: React.ReactNode
}
const TabsRoot: React.FC<TabsRootProps> = ({ children }) => {
  const { state: { activeTabId }, dispatch } = useContext(TabContext)

  const handleValueChange = (value: string): void => {
    dispatch({
      type: 'SET_ACTIVE_TAB',
      payload: parseInt(value, 10)
    })
  }

  return (
    <Tabs.Root className="flex flex-col h-full text-xs" value={activeTabId.toString()} onValueChange={handleValueChange}>
      {children}
    </Tabs.Root>
  )
}

export default TabsRoot
