import { Routes, Route, Outlet } from 'react-router-dom'
import { Navigation } from './components/Navigation'
import { Settings } from './components/Settings'
import Dashboard from './components/Dashboard' // Assuming Dashboard.tsx is created in components
import { HnItems } from './components/HnItems'
import { EntitiesPage } from './pages/EntitiesPage'
import { EntityGraphPage } from './pages/EntityGraphPage'
import { EntityDetailPage } from './pages/EntityDetailPage'

function App() {
  return (
    <div className="flex min-h-screen bg-gray-50">
      <Navigation />
      <main className="flex-1 ml-0 lg:ml-64">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/hn-items" element={<HnItems />} />
          <Route path="/entities" element={<EntitiesPage />} />
          <Route path="/entities/graph" element={<EntityGraphPage />} />
          <Route path="/entities/:id" element={<EntityDetailPage />} />
          <Route path="/settings" element={<Settings />} />
        </Routes>
        <Outlet /> {/* This is where nested routes would render if App was a layout route */}
      </main>
    </div>
  )
}

export default App
