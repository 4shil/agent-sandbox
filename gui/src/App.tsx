import { Routes, Route } from 'react-router-dom'
import Home from './pages/Home'
import Sessions from './pages/Sessions'
import Timeline from './pages/Timeline'
import Stats from './pages/Stats'
import Settings from './pages/Settings'

function App() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <Routes>
        <Route path="/" element={<Home />} />
        <Route path="/sessions" element={<Sessions />} />
        <Route path="/timeline" element={<Timeline />} />
        <Route path="/stats" element={<Stats />} />
        <Route path="/settings" element={<Settings />} />
      </Routes>
    </div>
  )
}

export default App
