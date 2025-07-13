import React, { useState, useEffect, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useVirtualizer } from '@tanstack/react-virtual'
import Fuse from 'fuse.js'
import { Button } from './ui/button'
import { Card, CardContent } from './ui/card'
import type { Entity } from '../types/Entity'
import type { SearchEntitiesResponse } from '../types/SearchEntitiesResponse'
import type { GetEntitiesResponse } from '../types/GetEntitiesResponse'

interface EntityListProps {
  searchQuery: string
  entityType: string
  onEntitySelect: (entity: Entity) => void
}

const API_BASE = 'http://localhost:8080/api'

async function searchEntities(query: string, entityType?: string, page = 1, limit = 100): Promise<SearchEntitiesResponse> {
  const params = new URLSearchParams({
    q: query,
    page: page.toString(),
    limit: limit.toString(),
  })
  
  if (entityType) {
    params.set('entity_type', entityType)
  }

  const response = await fetch(`${API_BASE}/entities/search?${params}`)
  if (!response.ok) {
    throw new Error('Failed to search entities')
  }
  return response.json()
}

async function getAllEntities(page = 1, limit = 100): Promise<GetEntitiesResponse> {
  const params = new URLSearchParams({
    page: page.toString(),
    limit: limit.toString(),
  })

  const response = await fetch(`${API_BASE}/entities?${params}`)
  if (!response.ok) {
    throw new Error('Failed to fetch entities')
  }
  return response.json()
}

export function EntityList({ searchQuery, entityType, onEntitySelect }: EntityListProps) {
  const [page, setPage] = useState(1)
  const [allEntities, setAllEntities] = useState<Entity[]>([])

  // Query for search or all entities with optimized caching
  const { data, isLoading, error } = useQuery({
    queryKey: ['entities', searchQuery, entityType, page],
    queryFn: (): Promise<SearchEntitiesResponse | GetEntitiesResponse> => {
      if (searchQuery.trim()) {
        return searchEntities(searchQuery, entityType || undefined, page, 50) // Smaller batch for search
      } else {
        return getAllEntities(page, 100) // Larger batch for browsing
      }
    },
    staleTime: 5 * 60 * 1000, // Cache for 5 minutes
    gcTime: 10 * 60 * 1000, // Keep in memory for 10 minutes
    refetchOnWindowFocus: false,
  })

  // Accumulate entities for infinite scroll
  useEffect(() => {
    if (data) {
      // Handle both SearchEntitiesResponse (EntityWithStats[]) and GetEntitiesResponse (Entity[])
      const entities = data.entities.map(entity => 
        'entity' in entity ? entity.entity : entity
      ) as Entity[]
      
      if (page === 1) {
        setAllEntities(entities)
      } else {
        setAllEntities(prev => [...prev, ...entities])
      }
    }
  }, [data, page])

  // Reset pagination when search changes
  useEffect(() => {
    setPage(1)
    setAllEntities([])
  }, [searchQuery, entityType])

  // Fuzzy search for client-side filtering when not using server search
  const fuse = useMemo(() => {
    if (!searchQuery.trim()) return null
    
    return new Fuse(allEntities, {
      keys: ['entity_value', 'entity_type'],
      threshold: 0.3,
      includeScore: true,
    })
  }, [allEntities, searchQuery])

  const filteredEntities = useMemo(() => {
    if (!searchQuery.trim()) {
      return allEntities
    }
    
    if (fuse) {
      return fuse.search(searchQuery).map(result => result.item)
    }
    
    return allEntities
  }, [allEntities, searchQuery, fuse])

  // Virtual scrolling for performance
  const parentRef = React.useRef<HTMLDivElement>(null)
  const rowVirtualizer = useVirtualizer({
    count: filteredEntities.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 80,
    overscan: 5,
  })

  const handleLoadMore = () => {
    if (data) {
      // Only GetEntitiesResponse has total_pages, SearchEntitiesResponse doesn't
      const totalPages = 'total_pages' in data ? data.total_pages : Math.ceil(Number(data.total_count) / 50)
      if (page < totalPages) {
        setPage(prev => prev + 1)
      }
    }
  }

  const getEntityTypeColor = (type: string) => {
    const colors: Record<string, string> = {
      person: 'bg-blue-100 text-blue-800',
      company: 'bg-green-100 text-green-800',
      technology: 'bg-purple-100 text-purple-800',
      location: 'bg-orange-100 text-orange-800',
      organization: 'bg-red-100 text-red-800',
      product: 'bg-yellow-100 text-yellow-800',
      event: 'bg-indigo-100 text-indigo-800',
    }
    return colors[type] || 'bg-gray-100 text-gray-800'
  }

  if (isLoading && page === 1) {
    return (
      <div className="flex justify-center items-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="text-center text-red-600 p-8">
        <p>Error loading entities: {error.message}</p>
        <Button onClick={() => window.location.reload()} className="mt-4">
          Retry
        </Button>
      </div>
    )
  }

  if (!filteredEntities.length) {
    return (
      <div className="text-center text-gray-500 p-8">
        <p>No entities found</p>
        {searchQuery && (
          <p className="text-sm mt-2">Try adjusting your search query or filters</p>
        )}
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {/* Results Info */}
      <div className="text-sm text-gray-600">
        Showing {filteredEntities.length} of {data ? Number(data.total_count) : 0} entities
      </div>

      {/* Virtual List Container */}
      <div
        ref={parentRef}
        className="h-96 overflow-auto border rounded-lg"
        style={{ height: '400px' }}
      >
        <div
          style={{
            height: `${rowVirtualizer.getTotalSize()}px`,
            width: '100%',
            position: 'relative',
          }}
        >
          {rowVirtualizer.getVirtualItems().map((virtualRow) => {
            const entity = filteredEntities[virtualRow.index]
            if (!entity) return null

            return (
              <Card
                key={entity.id}
                className="absolute top-0 left-0 w-full cursor-pointer hover:shadow-md transition-shadow"
                style={{
                  height: `${virtualRow.size}px`,
                  transform: `translateY(${virtualRow.start}px)`,
                }}
                onClick={() => onEntitySelect(entity)}
              >
                <CardContent className="p-4">
                  <div className="flex justify-between items-start">
                    <div className="flex-1">
                      <h3 className="font-medium text-lg">{entity.entity_value}</h3>
                      <div className="flex items-center gap-2 mt-1">
                        <span
                          className={`px-2 py-1 text-xs rounded-full ${getEntityTypeColor(entity.entity_type)}`}
                        >
                          {entity.entity_type}
                        </span>
                        <span className="text-xs text-gray-500">
                          ID: {entity.id}
                        </span>
                      </div>
                    </div>
                    <Button variant="outline" size="sm">
                      View Details
                    </Button>
                  </div>
                </CardContent>
              </Card>
            )
          })}
        </div>
      </div>

      {/* Load More */}
      {data && page < ('total_pages' in data ? data.total_pages : Math.ceil(Number(data.total_count) / 50)) && (
        <div className="text-center">
          <Button
            onClick={handleLoadMore}
            disabled={isLoading}
            variant="outline"
          >
            {isLoading ? 'Loading...' : 'Load More'}
          </Button>
        </div>
      )}
    </div>
  )
}