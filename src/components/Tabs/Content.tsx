import * as Tabs from '@radix-ui/react-tabs'
import { useHookstate, type State } from '@hookstate/core'
import { type Tab } from '../../state'

interface TabsContentProps {
  tab: State<Tab>
  children?: React.ReactNode
}

const TabsContent: React.FC<TabsContentProps> = ({ tab, children }) => {
  const state = useHookstate(tab)
  return (
    <Tabs.Content key={state.id.get()} className="flex-grow flex-shrink-0 overflow-y-auto" value={state.id.get().toString()}>
      {children}
    </Tabs.Content>
  )
}

export default TabsContent
