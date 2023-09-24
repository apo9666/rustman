import { type MouseEvent } from 'react'
import * as Tabs from '@radix-ui/react-tabs'
import { Dot, X } from 'lucide-react'
import { type State, useHookstate } from '@hookstate/core'
import { type Tab, tabState } from '../../state'

interface TabsTriggerProps {
  tab: State<Tab>
  index: number
}

const TabsTrigger: React.FC<TabsTriggerProps> = (props) => {
  const tab = useHookstate(props.tab)
  const state = useHookstate(tabState)

  const closeTab = (e: MouseEvent<SVGSVGElement>): void => {
    e.preventDefault()

    state.tabs.set(tabs => tabs.filter((_, i) => i !== props.index))

    const activeTabId = state.activeTabId.get()
    if (activeTabId >= props.index) {
      state.activeTabId.set(props.index - 1)
    }
  }

  return (
    <Tabs.Trigger className="group [&[data-state='active']]:border-b [&[data-state='active']]:border-yellow-500 [&[data-state='inactive']]:text-gray-400 p-2" value={props.index.toString()}>
      <a title={tab.label.get()}>{tab.label.get()} <Dot className="inline text-yellow-300" />
        <X onPointerDown={closeTab} className="inline invisible mb-[1px] group-hover:visible group-[&[data-state='active']]:visible" size={14} />
      </a>
    </Tabs.Trigger>
  )
}

export default TabsTrigger
