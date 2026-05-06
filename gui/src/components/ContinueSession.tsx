import { useState } from 'react'
import { useNavigate } from 'react-router-dom'

interface Session {
  id: string
  agent: string
  timestamp: string
  duration: string
  canContinue: boolean
}

export default function ContinueSession() {
  const navigate = useNavigate()
  const [sessions, setSessions] = useState<Session[]>([
    { id: 'claude-20250506-001', agent: 'claude', timestamp: '2025-05-06 14:30', duration: '5m 23s', canContinue: true },
    { id: 'opencode-20250506-002', agent: 'opencode', timestamp: '2025-05-06 15:15', duration: '12m 45s', canContinue: true },
  ])

  const handleContinue = (sessionId: string) => {
    console.log('Continue session:', sessionId)
    // TODO: Tauri command to continue session with --continue flag
    // This allows resuming from where the agent left off
  }

  return (
    <div className="p-4">
      <h2 className="text-2xl font-bold mb-4 ascii-art">CONTINUE SESSION</h2>
      <p className="text-text-muted mb-4">Resume work from a previous session</p>
      <div className="space-y-2">
        {sessions.map(session => (
          <div key={session.id} className="p-3 bg-surface rounded border border-border flex justify-between items-center">
            <div>
              <div className="font-bold text-primary">{session.id}</div>
              <div className="text-sm text-text-muted">{session.agent} | {session.timestamp} | {session.duration}</div>
            </div>
            <button 
              onClick={() => handleContinue(session.id)}
              disabled={!session.canContinue}
              className={`px-4 py-2 rounded text-sm ${session.canContinue 
                ? 'bg-primary text-background hover:opacity-80' 
                : 'bg-surface-hover text-text-muted cursor-not-allowed'
              }`}
            >
              Continue
            </button>
          </div>
        ))}
      </div>
    </div>
  )
}
