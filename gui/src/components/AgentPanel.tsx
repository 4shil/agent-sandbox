import { useState, useEffect } from 'react'

interface Agent {
  name: string
  displayName: string
  installed: boolean
  path?: string
}

export default function AgentPanel() {
  const [agents, setAgents] = useState<Agent[]>([
    { name: 'claude', displayName: 'Claude Code', installed: true, path: '/usr/local/bin/claude' },
    { name: 'opencode', displayName: 'OpenCode', installed: true, path: '/usr/local/bin/opencode' },
    { name: 'codex', displayName: 'OpenAI Codex', installed: false },
    { name: 'gemini', displayName: 'Google Gemini', installed: false },
  ])

  const [detecting, setDetecting] = useState(false)

  const handleDetect = () => {
    setDetecting(true)
    // TODO: Tauri command to scan PATH for agents
    setTimeout(() => setDetecting(false), 1500)
  }

  const handleInstall = (name: string) => {
    console.log('Install agent:', name)
    // TODO: Tauri command to install agent via npm
  }

  return (
    <div className="p-4">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-2xl font-bold ascii-art">AGENT DETECTION</h2>
        <button 
          onClick={handleDetect}
          disabled={detecting}
          className="px-4 py-2 bg-primary text-background rounded hover:opacity-80 text-sm"
        >
          {detecting ? 'Detecting...' : 'Re-Detect'}
        </button>
      </div>
      <div className="space-y-2">
        {agents.map(agent => (
          <div key={agent.name} className="p-3 bg-surface rounded border border-border flex justify-between items-center">
            <div className="flex items-center gap-3">
              <span className={`w-3 h-3 rounded-full ${agent.installed ? 'bg-success' : 'bg-error'}`} />
              <div>
                <div className="font-bold text-primary">{agent.displayName}</div>
                <div className="text-xs text-text-muted">{agent.name}</div>
                {agent.installed && agent.path && (
                  <div className="text-xs text-text-muted">{agent.path}</div>
                )}
              </div>
            </div>
            <div>
              {agent.installed ? (
                <span className="text-success text-sm">Installed</span>
              ) : (
                <button 
                  onClick={() => handleInstall(agent.name)}
                  className="px-3 py-1 bg-accent text-background rounded hover:opacity-80 text-sm"
                >
                  Install
                </button>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
