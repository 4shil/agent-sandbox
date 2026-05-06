import { Link } from 'react-router-dom'

export default function Home() {
  return (
    <div className="p-8">
      <h1 className="text-3xl font-bold mb-4">abox - Agent Sandbox</h1>
      <p className="mb-4">Welcome to the agent-sandbox GUI</p>
      <nav className="space-x-4">
        <Link to="/sessions" className="text-blue-500 hover:underline">Sessions</Link>
        <Link to="/timeline" className="text-blue-500 hover:underline">Timeline</Link>
        <Link to="/stats" className="text-blue-500 hover:underline">Stats</Link>
        <Link to="/settings" className="text-blue-500 hover:underline">Settings</Link>
      </nav>
    </div>
  )
}
