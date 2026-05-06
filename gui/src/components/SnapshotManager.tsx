import { useState } from 'react'

interface Snapshot {
  id: string
  sessionId: string
  timestamp: string
  size: string
}

export default function SnapshotManager() {
  const [snapshots, setSnapshots] = useState<Snapshot[]>([
    { id: 'snap-001', sessionId: 'claude-20250506', timestamp: '2025-05-06 14:30', size: '2.3 MB' },
    { id: 'snap-002', sessionId: 'opencode-20250506', timestamp: '2025-05-06 15:15', size: '1.8 MB' },
  ])

  const handleRestore = (id: string) => {
    console.log('Restore snapshot:', id)
    // TODO: Tauri command to restore snapshot
  }

  const handleDelete = (id: string) => {
    setSnapshots(snapshots.filter(s => s.id !== id))
  }

  return (
    <div className="p-4">
      <h2 className="text-2xl font-bold mb-4 ascii-art">SNAPSHOT MANAGER</h2>
      <div className="space-y-2">
        {snapshots.map(snap => (
          <div key={snap.id} className="p-3 bg-surface rounded border border-border flex justify-between items-center">
            <div>
              <div className="font-bold text-primary">{snap.id}</div>
              <div className="text-sm text-text-muted">Session: {snap.sessionId}</div>
              <div className="text-xs text-text-muted">{snap.timestamp} | {snap.size}</div>
            </div>
            <div className="space-x-2">
              <button 
                onClick={() => handleRestore(snap.id)}
                className="px-3 py-1 bg-primary text-background rounded hover:opacity-80 text-sm"
              >
                Restore
              </button>
              <button 
                onClick={() => handleDelete(snap.id)}
                className="px-3 py-1 bg-error text-white rounded hover:opacity-80 text-sm"
              >
                Delete
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
