import { useState } from 'react'

interface Template {
  id: string
  name: string
  description: string
  language: string
}

export default function SandboxTemplates() {
  const [templates, setTemplates] = useState<Template[]>([
    { id: 'node', name: 'Node.js', description: 'JavaScript/TypeScript sandbox', language: 'javascript' },
    { id: 'python', name: 'Python', description: 'Python sandbox with venv', language: 'python' },
    { id: 'rust', name: 'Rust', description: 'Rust project sandbox', language: 'rust' },
    { id: 'empty', name: 'Empty', description: 'Blank workspace', language: 'none' },
  ])
  const [selected, setSelected] = useState('node')

  const handleCreate = () => {
    console.log('Create from template:', selected)
    // TODO: Tauri command to create sandbox from template
  }

  return (
    <div className="p-4">
      <h2 className="text-2xl font-bold mb-4 ascii-art">SANDBOX TEMPLATES</h2>
      <div className="grid grid-cols-2 gap-4 mb-4">
        {templates.map(t => (
          <div 
            key={t.id}
            onClick={() => setSelected(t.id)}
            className={`p-4 border rounded cursor-pointer transition-colors ${
              selected === t.id 
                ? 'border-primary bg-surface-hover' 
                : 'border-border bg-surface hover:border-primary'
            }`}
          >
            <div className="font-bold text-primary">{t.name}</div>
            <div className="text-sm text-text-muted">{t.description}</div>
            <div className="text-xs text-accent mt-2">{t.language}</div>
          </div>
        ))}
      </div>
      <button 
        onClick={handleCreate}
        className="px-4 py-2 bg-primary text-background rounded hover:opacity-80"
      >
        Create from Template
      </button>
    </div>
  )
}
