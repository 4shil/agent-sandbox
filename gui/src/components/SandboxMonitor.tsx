import { useEffect, useState } from 'react'

interface SandboxStatus {
  cpu: number
  memory: number
  activeSessions: number
  diskUsage: string
}

export default function SandboxMonitor() {
  const [status, setStatus] = useState<SandboxStatus>({
    cpu: 0,
    memory: 0,
    activeSessions: 0,
    diskUsage: '0 GB',
  })

  useEffect(() => {
    // TODO: Connect to Tauri command to get real stats
    const interval = setInterval(() => {
      setStatus({
        cpu: Math.random() * 100,
        memory: Math.random() * 100,
        activeSessions: Math.floor(Math.random() * 5),
        diskUsage: `${(Math.random() * 10).toFixed(1)} GB`,
      })
    }, 2000)

    return () => clearInterval(interval)
  }, [])

  return (
    <div className="p-4 bg-surface rounded border border-border">
      <h2 className="text-lg font-bold mb-4 ascii-art">SANDBOX MONITOR</h2>
      <div className="grid grid-cols-2 gap-4">
        <div className="p-3 bg-surface-hover rounded">
          <div className="text-text-muted text-sm">CPU</div>
          <div className="text-2xl font-bold text-primary">{status.cpu.toFixed(1)}%</div>
        </div>
        <div className="p-3 bg-surface-hover rounded">
          <div className="text-text-muted text-sm">Memory</div>
          <div className="text-2xl font-bold text-accent">{status.memory.toFixed(1)}%</div>
        </div>
        <div className="p-3 bg-surface-hover rounded">
          <div className="text-text-muted text-sm">Active Sessions</div>
          <div className="text-2xl font-bold text-success">{status.activeSessions}</div>
        </div>
        <div className="p-3 bg-surface-hover rounded">
          <div className="text-text-muted text-sm">Disk Usage</div>
          <div className="text-2xl font-bold text-warning">{status.diskUsage}</div>
        </div>
      </div>
    </div>
  )
}
