import { useState } from 'react'
import { Navigation } from './components/Navigation'
import { Settings } from './components/Settings'

function App() {
  const [currentView, setCurrentView] = useState<'dashboard' | 'settings'>('dashboard')

  return (
    <div className="flex min-h-screen bg-gray-50">
      <Navigation onSettingsClick={() => setCurrentView('settings')} />
      <main className="flex-1">
        {currentView === 'settings' ? (
          <Settings />
        ) : (
          <div className="p-8">
            <h1 className="text-3xl font-bold mb-6">Dashboard</h1>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              <div className="bg-white p-6 rounded-lg shadow">
                <h2 className="text-xl font-semibold mb-2">Welcome to Pixlie</h2>
                <p className="text-gray-600">
                  Smart Entity Analysis for Hacker News Discussions
                </p>
              </div>
              <div className="bg-white p-6 rounded-lg shadow">
                <h2 className="text-xl font-semibold mb-2">Data Collection</h2>
                <p className="text-gray-600">
                  Configure your Hacker News data collection in Settings
                </p>
              </div>
              <div className="bg-white p-6 rounded-lg shadow">
                <h2 className="text-xl font-semibold mb-2">Entity Analysis</h2>
                <p className="text-gray-600">
                  Advanced NLP for startup and investor insights
                </p>
              </div>
            </div>
          </div>
        )}
      </main>
    </div>
  )
}

export default App
