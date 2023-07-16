import { useContext } from 'react'
import { TabContext } from './context/TabContext'
import Tabs from './components/Tabs'
import Request from './components/Request'
import Response from './components/Response'

const Section: React.FC = () => {
  const { state: { tabs } } = useContext(TabContext)

  return (
    <Tabs.Root>
      <Tabs.List>
        {tabs.map(tab => (
          <Tabs.Trigger key={tab.id} tab={tab} />
        ))}
      </Tabs.List>
      {tabs.map(tab => (
        <Tabs.Content tab={tab} key={tab.id}>
          <div className="h-full flex flex-col">
            <Request.Root>
              <Request.Title title={tab.label} />
              <hr className="border-zinc-600" />
              <Request.Url id={tab.id} method={tab.content.method} url={tab.content.url} body={tab.content.body} />
              <Request.Content id={tab.id} content={tab.content} />
            </Request.Root>
            <Response.Root>
              <Response.Content response={tab.content.response} />
            </Response.Root>
          </div>
        </Tabs.Content>
      ))}
    </Tabs.Root>
  )
}

export default Section
