import Tabs from './components/Tabs'
import Request from './components/Request'
import Response from './components/Response'
import { useHookstate } from '@hookstate/core'
import { tabState } from './state'

const Section: React.FC = () => {
  const state = useHookstate(tabState)

  return (
    <Tabs.Root>
      <Tabs.List>
        {state.tabs.map(tab => (
          <Tabs.Trigger key={tab.id.get()} tab={tab} />
        ))}
      </Tabs.List>
      {state.tabs.map(tab => (
        <Tabs.Content tab={tab} key={tab.id.get()}>
          <div className="h-full flex flex-col">
            <Request.Root>
              <Request.Title title={tab.label.get()} />
              <hr className="border-zinc-600" />
              <Request.Url content={tab.content} />
              <Request.Content content={tab.content} />
            </Request.Root>
            <Response.Root>
              <Response.Content data={tab.content.response.data} />
            </Response.Root>
          </div>
        </Tabs.Content>
      ))}
    </Tabs.Root>
  )
}

export default Section
