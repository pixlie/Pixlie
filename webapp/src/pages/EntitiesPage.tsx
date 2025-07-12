import { useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { EntityBrowser } from '../components/EntityBrowser'

export function EntitiesPage() {
  const [searchParams, setSearchParams] = useSearchParams()
  const [mode, setMode] = useState<'list' | 'graph'>(
    (searchParams.get('mode') as 'list' | 'graph') || 'list'
  )

  const handleModeChange = (newMode: 'list' | 'graph') => {
    setMode(newMode)
    setSearchParams(prev => {
      const newParams = new URLSearchParams(prev)
      newParams.set('mode', newMode)
      return newParams
    })
  }

  return (
    <div className="p-6">
      <EntityBrowser mode={mode} onModeChange={handleModeChange} />
    </div>
  )
}