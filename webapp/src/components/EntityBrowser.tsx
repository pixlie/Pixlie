import { useState, useCallback } from 'react'
import { Button } from './ui/button'
import { Card, CardContent, CardHeader, CardTitle } from './ui/card'
import { EntitySearchBar } from './EntitySearchBar'
import { EntityList } from './EntityList'
import { EntityDetailModal } from './EntityDetailModal'
import { RelationshipGraph } from './RelationshipGraph'
import type { Entity } from '../types/database'

interface EntityBrowserProps {
  mode?: 'list' | 'graph'
  onModeChange?: (mode: 'list' | 'graph') => void
}

export function EntityBrowser({ mode = 'list', onModeChange }: EntityBrowserProps) {
  const [selectedEntity, setSelectedEntity] = useState<Entity | null>(null)
  const [searchQuery, setSearchQuery] = useState('')
  const [entityType, setEntityType] = useState<string>('')

  const handleEntitySelect = useCallback((entity: Entity) => {
    setSelectedEntity(entity)
  }, [])

  const handleCloseModal = useCallback(() => {
    setSelectedEntity(null)
  }, [])

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <h1 className="text-2xl sm:text-3xl font-bold">Entity Browser</h1>
        <div className="flex gap-2 w-full sm:w-auto">
          <Button
            variant={mode === 'list' ? 'default' : 'outline'}
            onClick={() => onModeChange?.('list')}
            className="flex-1 sm:flex-none"
          >
            List View
          </Button>
          <Button
            variant={mode === 'graph' ? 'default' : 'outline'}
            onClick={() => onModeChange?.('graph')}
            className="flex-1 sm:flex-none"
          >
            Graph View
          </Button>
        </div>
      </div>

      {/* Search Bar */}
      <EntitySearchBar
        query={searchQuery}
        entityType={entityType}
        onQueryChange={setSearchQuery}
        onEntityTypeChange={setEntityType}
      />

      {/* Main Content */}
      <Card>
        <CardHeader>
          <CardTitle>
            {mode === 'list' ? 'Entity List' : 'Relationship Graph'}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {mode === 'list' ? (
            <EntityList
              searchQuery={searchQuery}
              entityType={entityType}
              onEntitySelect={handleEntitySelect}
            />
          ) : (
            <RelationshipGraph
              searchQuery={searchQuery}
              entityType={entityType}
              onEntitySelect={handleEntitySelect}
            />
          )}
        </CardContent>
      </Card>

      {/* Entity Detail Modal */}
      {selectedEntity && (
        <EntityDetailModal
          entity={selectedEntity}
          isOpen={!!selectedEntity}
          onClose={handleCloseModal}
        />
      )}
    </div>
  )
}