import { Routes, Route, Outlet } from 'react-router-dom'
import { Navigation } from './components/Navigation'
import { Settings } from './components/Settings'
import Dashboard from './components/Dashboard' // Assuming Dashboard.tsx is created in components
import { HnItems } from './components/HnItems'
import { Entities } from './components/Entities'

function App() {
  return (
    <div className="flex min-h-screen bg-gray-50">
      <Navigation />
      <main className="flex-1 ml-64">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/hn-items" element={<HnItems />} />
          <Route path="/entities" element={<Entities />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
        <Outlet /> {/* This is where nested routes would render if App was a layout route */}
      </main>
    </div>
  )
}

export default App
