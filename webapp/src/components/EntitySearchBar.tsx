import { useState, useCallback, useMemo } from 'react'
import { Input } from './ui/input'
import { Button } from './ui/button'
import { Card, CardContent } from './ui/card'

interface EntitySearchBarProps {
  query: string
  entityType: string
  onQueryChange: (query: string) => void
  onEntityTypeChange: (type: string) => void
}

const ENTITY_TYPES = [
  { value: '', label: 'All Types' },
  { value: 'person', label: 'Person' },
  { value: 'company', label: 'Company' },
  { value: 'technology', label: 'Technology' },
  { value: 'location', label: 'Location' },
  { value: 'organization', label: 'Organization' },
  { value: 'product', label: 'Product' },
  { value: 'event', label: 'Event' },
]

export function EntitySearchBar({
  query,
  entityType,
  onQueryChange,
  onEntityTypeChange,
}: EntitySearchBarProps) {
  const [localQuery, setLocalQuery] = useState(query)

  const handleSearch = useCallback(() => {
    onQueryChange(localQuery)
  }, [localQuery, onQueryChange])

  const handleKeyPress = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter') {
        handleSearch()
      }
    },
    [handleSearch]
  )

  const handleClearSearch = useCallback(() => {
    setLocalQuery('')
    onQueryChange('')
  }, [onQueryChange])

  const selectedTypeLabel = useMemo(() => {
    return ENTITY_TYPES.find(type => type.value === entityType)?.label || 'All Types'
  }, [entityType])

  return (
    <Card>
      <CardContent className="p-4">
        <div className="flex flex-col sm:flex-row gap-3 items-stretch sm:items-center">
          {/* Search Input */}
          <div className="flex-1 relative">
            <Input
              type="text"
              placeholder="Search entities..."
              value={localQuery}
              onChange={(e) => setLocalQuery(e.target.value)}
              onKeyPress={handleKeyPress}
              className="pr-10"
            />
            {localQuery && (
              <Button
                variant="ghost"
                size="sm"
                onClick={handleClearSearch}
                className="absolute right-1 top-1/2 transform -translate-y-1/2 h-6 w-6 p-0"
              >
                ×
              </Button>
            )}
          </div>

          {/* Entity Type Filter */}
          <div className="relative min-w-0 sm:min-w-[150px]">
            <select
              value={entityType}
              onChange={(e) => onEntityTypeChange(e.target.value)}
              className="w-full appearance-none bg-white border border-gray-300 rounded-md px-3 py-2 pr-8 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              {ENTITY_TYPES.map(type => (
                <option key={type.value} value={type.value}>
                  {type.label}
                </option>
              ))}
            </select>
            <div className="absolute inset-y-0 right-0 flex items-center px-2 pointer-events-none">
              <svg className="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
              </svg>
            </div>
          </div>

          {/* Search Button */}
          <Button onClick={handleSearch} disabled={!localQuery.trim()} className="w-full sm:w-auto">
            Search
          </Button>
        </div>

        {/* Search Info */}
        {(query || entityType) && (
          <div className="mt-3 text-sm text-gray-600">
            {query && (
              <span>
                Searching for: <strong>{query}</strong>
              </span>
            )}
            {query && entityType && <span> • </span>}
            {entityType && (
              <span>
                Type: <strong>{selectedTypeLabel}</strong>
              </span>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  )
}