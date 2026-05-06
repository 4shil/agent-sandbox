import { useState } from 'react'

interface Tab {
  id: string
  title: string
  agent: string
  active: boolean
}

export default function SandboxTabs() {
  const [tabs, setTabs] = useState<Tab[]>([
    { id: '1', title: 'Claude Session', agent: 'claude', active: true },
    { id: '2', title: 'OpenCode Session', agent: 'opencode', active: false },
  ])
  const [activeTab, setActiveTab] = useState('1')

  const handleNewTab = () => {
    const newTab: Tab = {
      id: Date.now().toString(),
      title: 'New Session',
      agent: 'claude',
      active: false,
    }
    setTabs([...tabs, newTab])
  }

  const handleCloseTab = (id: string) => {
    setTabs(tabs.filter(t => t.id !== id))
    if (activeTab === id && tabs.length > 1) {
      setActiveTab(tabs[0].id)
    }
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex border-b border-border bg-surface">
        {tabs.map(tab => (
          <div
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`px-4 py-2 border-b-2 cursor-pointer text-sm ${
              activeTab === tab.id 
                ? 'border-primary text-primary' 
                : 'border-transparent text-text-muted hover:text-text-primary'
            }`}
          >
            <span>{tab.title}</span>
            <button 
              onClick={(e) => { e.stopPropagation(); handleCloseTab(tab.id) }}
              className="ml-2 text-text-muted hover:text-error"
            >
              ×
            </button>
          </div>
        ))}
        <button 
          onClick={handleNewTab}
          className="px-4 py-2 text-text-muted hover:text-primary"
          title="New Sandbox Tab"
        >
          +
        </button>
      </div>
      <div className="flex-1 p-4">
        <div className="text-text-muted">Sandbox content for: {tabs.find(t => t.id === activeTab)?.title}</div>
      </div>
    </div>
  )
}
