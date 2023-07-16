import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import './index.css'
import { TreeProvider } from './context/TreeContext.tsx'
import { TabProvider } from './context/TabContext.tsx'

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <TreeProvider>
      <TabProvider>
        <App />
      </TabProvider>
    </TreeProvider>
  </React.StrictMode>
)
