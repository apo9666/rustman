import { ArrowRight } from 'lucide-react'

interface RequestProps {
  name: string
  method: string
  open: boolean
}

const colors: Record<string, string> = {
  GET: 'green',
  POST: 'yellow',
  PUT: 'blue',
  DELETE: 'red'
}

const short: Record<string, string> = {
  DELETE: 'DEL'
}

const TreeRequest: React.FC<RequestProps> = ({ name, method, open }) => {
  return (
    <a className="flex items-center gap-1 px-5 py-1">
      <span className="flex items-center">
        <span className={`text-xs ${colors[method] !== undefined ? 'text-' + colors[method] + '-600' : ''} w-10 text-right`}>{short[method] ?? method}</span>
      </span>
      <span className="w-96 truncate hover:cursor-default" title={name}>{name}</span>
      {open ? <ArrowRight /> : ''}
    </a>
  )
}

export default TreeRequest
