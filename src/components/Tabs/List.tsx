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

    state.activeTabId.set(state.tabs.length)
    state.tabs.merge([
      {
        content: {
          url: '',
          headers: [
            {
              enable: true,
              key: '',
              value: ''
            }
          ],
          params: [
            {
              enable: true,
              key: '',
              value: ''
            }
          ],
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
        },
        label: 'New'
      }
    ])
  }

  return (
    <Tabs.List className="flex-wrap">
      {children}
      <Plus onPointerDown={addTab} className="ml-1 cursor-pointer inline-block" size={20} />
    </Tabs.List>
  )
}

export default TabsList
