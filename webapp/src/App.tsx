import { Routes, Route, Outlet } from 'react-router-dom'
import { Navigation } from './components/Navigation'
import { Settings } from './components/Settings'
import Dashboard from './components/Dashboard' // Assuming Dashboard.tsx is created in components

function App() {
  return (
    <div className="flex min-h-screen bg-gray-50">
      <Navigation />
      <main className="flex-1">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
        <Outlet /> {/* This is where nested routes would render if App was a layout route */}
      </main>
    </div>
  )
}

export default App
