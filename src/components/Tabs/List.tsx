import { useContext, type MouseEvent } from 'react'
import * as Tabs from '@radix-ui/react-tabs'
import { Plus } from 'lucide-react'
import { TabContext } from '../../context/TabContext'

interface TabsListProps {
  children?: React.ReactNode
}

const TabsList: React.FC<TabsListProps> = ({ children }) => {
  const { dispatch } = useContext(TabContext)

  const addTab = (e: MouseEvent<SVGSVGElement>): void => {
    e.preventDefault()

    dispatch({
      type: 'ADD_TAB'
    })
  }

  return (
    <Tabs.List className="flex-wrap">
      {children}
      <Plus onPointerDown={addTab} className="flex ml-1 cursor-pointer" size={20} />
    </Tabs.List>
  )
}

export default TabsList
