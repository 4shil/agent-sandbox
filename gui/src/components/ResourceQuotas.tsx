import { useState } from 'react'

export default function ResourceQuotas() {
  const [quotas, setQuotas] = useState({
    cpuLimit: 50,
    memoryLimit: 2048,
    diskLimit: 10,
    maxSessions: 5,
  })

  const handleSave = () => {
    console.log('Save quotas:', quotas)
    // TODO: Tauri command to save quota settings
  }

  return (
    <div className="p-4">
      <h2 className="text-2xl font-bold mb-4 ascii-art">RESOURCE QUOTAS</h2>
      <div className="space-y-4 max-w-md">
        <div>
          <label className="block text-sm text-text-muted mb-1">CPU Limit (%)</label>
          <input 
            type="range" 
            min="10" max="100" 
            value={quotas.cpuLimit}
            onChange={(e) => setQuotas({...quotas, cpuLimit: parseInt(e.target.value)})}
            className="w-full"
          />
          <span className="text-sm text-primary">{quotas.cpuLimit}%</span>
        </div>
        <div>
          <label className="block text-sm text-text-muted mb-1">Memory Limit (MB)</label>
          <input 
            type="number" 
            value={quotas.memoryLimit}
            onChange={(e) => setQuotas({...quotas, memoryLimit: parseInt(e.target.value)})}
            className="w-full p-2 bg-surface border border-border rounded"
          />
        </div>
        <div>
          <label className="block text-sm text-text-muted mb-1">Disk Limit (GB)</label>
          <input 
            type="number" 
            value={quotas.diskLimit}
            onChange={(e) => setQuotas({...quotas, diskLimit: parseInt(e.target.value)})}
            className="w-full p-2 bg-surface border border-border rounded"
          />
        </div>
        <div>
          <label className="block text-sm text-text-muted mb-1">Max Concurrent Sessions</label>
          <input 
            type="number" 
            value={quotas.maxSessions}
            onChange={(e) => setQuotas({...quotas, maxSessions: parseInt(e.target.value)})}
            className="w-full p-2 bg-surface border border-border rounded"
          />
        </div>
        <button 
          onClick={handleSave}
          className="px-4 py-2 bg-primary text-background rounded hover:opacity-80"
        >
          Save Quotas
        </button>
      </div>
    </div>
  )
}
