import { type MouseEvent } from 'react'
import * as Tabs from '@radix-ui/react-tabs'
import { Plus } from 'lucide-react'
import { useHookstate } from '@hookstate/core'
import { MethodEnum, tabState } from '../../state'

interface TabsListProps {
  children?: React.ReactNode
}

const TabsList: React.FC<TabsListProps> = ({ children }) => {
  const state = useHookstate(tabState)

  const addTab = (e: MouseEvent<SVGSVGElement>): void => {
    e.preventDefault()

    const lastTabId = state.lastTabId.get() + 1
    state.lastTabId.set(lastTabId)
    state.activeTabId.set(lastTabId)
    state.tabs.set((tabs) => [
      ...tabs,
      {
        id: lastTabId,
        label: 'New Tab',
        content: {
          url: '',
          method: MethodEnum.GET,
          body: '',
          response: {
            data: '',
            headers: {},
            ok: true,
            rawHeaders: {},
            status: 200,
            url: ''
          }
        }
      }
    ])
  }

  return (
    <Tabs.List className="flex-wrap">
      {children}
      <Plus onPointerDown={addTab} className="flex ml-1 cursor-pointer" size={20} />
    </Tabs.List>
  )
}

export default TabsList
