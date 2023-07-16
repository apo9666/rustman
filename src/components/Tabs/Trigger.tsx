import { useContext, type MouseEvent } from 'react'
import { TabContext, type Tab } from '../../context/TabContext'
import * as Tabs from '@radix-ui/react-tabs'
import { Dot, X } from 'lucide-react'

interface TabsTriggerProps {
  tab: Tab
}

const TabsTrigger: React.FC<TabsTriggerProps> = ({ tab }) => {
  const { dispatch } = useContext(TabContext)

  const closeTab = (e: MouseEvent<SVGSVGElement>): void => {
    e.preventDefault()
    dispatch({ type: 'REMOVE_TAB', payload: tab })
  }

  return (
    <Tabs.Trigger key={tab.id} className="group [&[data-state='active']]:border-b [&[data-state='active']]:border-yellow-500 [&[data-state='inactive']]:text-gray-400 p-2" value={tab.id.toString()}>
      <a title={tab.label}>{tab.label} <Dot className="inline text-yellow-300" />
        <X onPointerDown={closeTab} className="inline invisible mb-[1px] group-hover:visible group-[&[data-state='active']]:visible" size={14} />
      </a>
    </Tabs.Trigger>
  )
}

export default TabsTrigger
