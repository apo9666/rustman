import { Save } from 'lucide-react'

interface RequestTitleProps {
  title: string
}

const RequestTitle: React.FC<RequestTitleProps> = ({ title }) => {
  return (
    <div className="flex">
      <h1 className="flex-grow p-3 font-bold text-base">{title}</h1>
      <div className="flex items-center">
        <button className="bg-zinc-600 mr-2 p-2 rounded-md font-bold hover:bg-zinc-700"><Save className="inline mb-[2px]" size={16} /> Save</button>
      </div>
    </div>
  )
}

export default RequestTitle
