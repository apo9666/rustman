import * as Tabs from '@radix-ui/react-tabs'
import { useHookstate } from '@hookstate/core'
import { tabState } from '../../state'

interface TabsRootProps {
  children?: React.ReactNode
}
const TabsRoot: React.FC<TabsRootProps> = ({ children }) => {
  const state = useHookstate(tabState)

  const handleValueChange = (value: string): void => {
    state.activeTabId.set(parseInt(value, 10))
  }

  return (
    <Tabs.Root className="flex flex-col h-full text-xs" value={state.activeTabId.get().toString()} onValueChange={handleValueChange}>
      {children}
    </Tabs.Root>
  )
}

export default TabsRoot
