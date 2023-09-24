import * as Tabs from '@radix-ui/react-tabs'
import { type State } from '@hookstate/core'
import ParamTable from './ParamTable'
import Body from './Body'
import { type TabContent } from '../../state'

interface RequestContentProps {
  content: State<TabContent>
}

const RequestContent: React.FC<RequestContentProps> = ({ content }) => {
  return (
    <Tabs.Root className="flex flex-col flex-grow text-xs overflow-y-auto" defaultValue="params">
      <Tabs.List className="flex-wrap px-2">
        <Tabs.Trigger className="[&[data-state='active']]:border-b [&[data-state='active']]:border-yellow-500 [&[data-state='inactive']]:text-gray-400 p-2" value="params">
          Params
        </Tabs.Trigger>
        <Tabs.Trigger className="[&[data-state='active']]:border-b [&[data-state='active']]:border-yellow-500 [&[data-state='inactive']]:text-gray-400 p-2" value="headers">
          Headers
        </Tabs.Trigger>
        <Tabs.Trigger className="[&[data-state='active']]:border-b [&[data-state='active']]:border-yellow-500 [&[data-state='inactive']]:text-gray-400 p-2" value="body">
          Body
        </Tabs.Trigger>
      </Tabs.List>
      <Tabs.Content className="flex-grow flex-shrink-0 overflow-y-auto" value="params">
        <ParamTable url={content.url} />
      </Tabs.Content>
      <Tabs.Content className="flex-grow flex-shrink-0 overflow-y-auto" value="headers">
        <ParamTable url={content.url} />
      </Tabs.Content>
      <Tabs.Content className="flex-grow flex-shrink-0 overflow-y-auto" value="body">
        <Body body={content.body} />
      </Tabs.Content>
    </Tabs.Root>
  )
}

export default RequestContent
