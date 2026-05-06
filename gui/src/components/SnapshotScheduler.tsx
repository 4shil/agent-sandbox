import { useState } from 'react'

export default function SnapshotScheduler() {
  const [schedule, setSchedule] = useState({
    enabled: true,
    interval: 30,
    maxSnapshots: 10,
    retainDays: 7,
  })

  const handleSave = () => {
    console.log('Save snapshot schedule:', schedule)
    // TODO: Tauri command to save snapshot schedule
  }

  return (
    <div className="p-4">
      <h2 className="text-2xl font-bold mb-4 ascii-art">SNAPSHOT SCHEDULE</h2>
      <div className="space-y-4 max-w-md">
        <div className="flex items-center justify-between">
          <span className="text-text-muted">Enable Auto-Snapshots</span>
          <button 
            onClick={() => setSchedule({...schedule, enabled: !schedule.enabled})}
            className={`px-4 py-1 rounded ${schedule.enabled ? 'bg-success text-white' : 'bg-surface-hover text-text-muted'}`}
          >
            {schedule.enabled ? 'ON' : 'OFF'}
          </button>
        </div>
        <div>
          <label className="block text-sm text-text-muted mb-1">Interval (seconds)</label>
          <input 
            type="number" 
            value={schedule.interval}
            onChange={(e) => setSchedule({...schedule, interval: parseInt(e.target.value)})}
            className="w-full p-2 bg-surface border border-border rounded"
          />
          <div className="text-xs text-text-muted mt-1">Current: Every {schedule.interval}s (rolling snapshots to .snapshot/latest/)</div>
        </div>
        <div>
          <label className="block text-sm text-text-muted mb-1">Max Snapshots</label>
          <input 
            type="number" 
            value={schedule.maxSnapshots}
            onChange={(e) => setSchedule({...schedule, maxSnapshots: parseInt(e.target.value)})}
            className="w-full p-2 bg-surface border border-border rounded"
          />
        </div>
        <div>
          <label className="block text-sm text-text-muted mb-1">Retain Days</label>
          <input 
            type="number" 
            value={schedule.retainDays}
            onChange={(e) => setSchedule({...schedule, retainDays: parseInt(e.target.value)})}
            className="w-full p-2 bg-surface border border-border rounded"
          />
        </div>
        <button 
          onClick={handleSave}
          className="px-4 py-2 bg-primary text-background rounded hover:opacity-80"
        >
          Save Schedule
        </button>
      </div>
    </div>
  )
}
