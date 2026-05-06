import { useState, useEffect, useRef } from 'react'

interface LogEntry {
  timestamp: string
  level: 'info' | 'warn' | 'error'
  message: string
  sessionId?: string
}

export default function LogStream() {
  const [logs, setLogs] = useState<LogEntry[]>([
    { timestamp: '14:30:12', level: 'info', message: 'Session claude-001 started', sessionId: 'claude-001' },
    { timestamp: '14:30:15', level: 'info', message: 'Agent initialized', sessionId: 'claude-001' },
    { timestamp: '14:31:20', level: 'warn', message: 'Memory usage > 80%', sessionId: 'claude-001' },
    { timestamp: '14:32:45', level: 'error', message: 'API timeout', sessionId: 'opencode-002' },
  ])
  const logEndRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    // Auto-scroll to bottom
    logEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [logs])

  const levelColor = (level: string) => {
    switch(level) {
      case 'info': return 'text-primary'
      case 'warn': return 'text-warning'
      case 'error': return 'text-error'
      default: return 'text-text-muted'
    }
  }

  return (
    <div className="p-4 h-full flex flex-col">
      <h2 className="text-2xl font-bold mb-4 ascii-art">LOG STREAM</h2>
      <div className="flex-1 bg-background rounded border border-border p-2 overflow-y-auto font-mono text-sm">
        {logs.map((log, i) => (
          <div key={i} className="py-1 border-b border-border last:border-0">
            <span className="text-text-muted">[{log.timestamp}]</span>
            <span className={`ml-2 ${levelColor(log.level)}`}>[{log.level.toUpperCase()}]</span>
            <span className="ml-2 text-text-primary">{log.message}</span>
            {log.sessionId && (
              <span className="ml-2 text-xs text-accent">[{log.sessionId}]</span>
            )}
          </div>
        ))}
        <div ref={logEndRef} />
      </div>
    </div>
  )
}
