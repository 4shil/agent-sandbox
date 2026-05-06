import { useState } from 'react'

interface Permission {
  id: string
  name: string
  description: string
  enabled: boolean
  agent: string
}

export default function AgentPermissions() {
  const [permissions, setPermissions] = useState<Permission[]>([
    { id: '1', name: 'file_read', description: 'Read files in workspace', enabled: true, agent: 'claude' },
    { id: '2', name: 'file_write', description: 'Write files in workspace', enabled: true, agent: 'claude' },
    { id: '3', name: 'network', description: 'Make network requests', enabled: false, agent: 'claude' },
    { id: '4', name: 'execute', description: 'Execute commands', enabled: true, agent: 'claude' },
    { id: '5', name: 'file_read', description: 'Read files in workspace', enabled: true, agent: 'opencode' },
    { id: '6', name: 'file_write', description: 'Write files in workspace', enabled: true, agent: 'opencode' },
  ])

  const togglePermission = (id: string) => {
    setPermissions(perms => 
      perms.map(p => p.id === id ? { ...p, enabled: !p.enabled } : p)
    )
  }

  const agentList = [...new Set(permissions.map(p => p.agent))]

  return (
    <div className="p-4">
      <h2 className="text-2xl font-bold mb-4 ascii-art">AGENT PERMISSIONS</h2>
      {agentList.map(agent => (
        <div key={agent} className="mb-6">
          <h3 className="text-lg font-bold text-primary mb-2">{agent.toUpperCase()}</h3>
          <div className="space-y-2">
            {permissions.filter(p => p.agent === agent).map(perm => (
              <div key={perm.id} className="p-3 bg-surface rounded border border-border flex justify-between items-center">
                <div>
                  <div className="font-bold text-text-primary">{perm.name}</div>
                  <div className="text-xs text-text-muted">{perm.description}</div>
                </div>
                <button 
                  onClick={() => togglePermission(perm.id)}
                  className={`px-4 py-1 rounded text-sm ${perm.enabled 
                    ? 'bg-success text-white' 
                    : 'bg-surface-hover text-text-muted'
                  }`}
                >
                  {perm.enabled ? 'ALLOW' : 'DENY'}
                </button>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  )
}
