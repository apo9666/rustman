import DialogDemo from './DialogDemo'
import Section from './Section'
import Side from './Side'

const App: React.FC = () => {
  return (
    <div className="h-screen flex">
      <aside className="flex-shrink-0 w-72 text-sm bg-local bg-zinc-800 overflow-y-auto">
        <Side />
        <DialogDemo />
      </aside>
      <main className="flex-grow bg-zinc-900 overflow-y-auto">
        <Section />
      </main>
    </div>
  )
}

export default App
