import TerminalComponent from '@/components/Terminal'

export default function Home() {
  return (
    <div className="w-full h-screen">
      <div className="p-4 border-b border-border">
        <h1 className="text-2xl font-bold ascii-art">abox - Agent Sandbox</h1>
        <p className="text-text-muted">Embedded TUI Dashboard</p>
      </div>
      <div className="flex-1 h-[calc(100vh-80px)]">
        <TerminalComponent className="h-full" />
      </div>
    </div>
  )
}
