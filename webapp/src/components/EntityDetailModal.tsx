import { useState, useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Button } from './ui/button'
import { Card, CardContent, CardHeader, CardTitle } from './ui/card'
import { TextHighlighter, HighlightLegend } from './TextHighlighter'
import type { Entity } from '../types/Entity'
import type { EntityDetailResponse } from '../types/EntityDetailResponse'
import type { GetEntityReferencesResponse } from '../types/GetEntityReferencesResponse'
import type { GetEntityItemsResponse } from '../types/GetEntityItemsResponse'

interface EntityDetailModalProps {
  entity: Entity
  isOpen: boolean
  onClose: () => void
}

const API_BASE = 'http://localhost:8080/api'

async function getEntityDetail(entityId: bigint): Promise<EntityDetailResponse> {
  const response = await fetch(`${API_BASE}/entities/${entityId}`)
  if (!response.ok) {
    throw new Error('Failed to fetch entity details')
  }
  return response.json()
}

async function getEntityReferences(entityId: bigint, page = 1, limit = 10): Promise<GetEntityReferencesResponse> {
  const params = new URLSearchParams({
    page: page.toString(),
    limit: limit.toString(),
  })
  
  const response = await fetch(`${API_BASE}/entities/${entityId}/references?${params}`)
  if (!response.ok) {
    throw new Error('Failed to fetch entity references')
  }
  return response.json()
}

async function getEntityItems(entityId: bigint, page = 1, limit = 5): Promise<GetEntityItemsResponse> {
  const params = new URLSearchParams({
    page: page.toString(),
    limit: limit.toString(),
  })
  
  const response = await fetch(`${API_BASE}/entities/${entityId}/items?${params}`)
  if (!response.ok) {
    throw new Error('Failed to fetch entity items')
  }
  return response.json()
}

export function EntityDetailModal({ entity, isOpen, onClose }: EntityDetailModalProps) {
  const [activeTab, setActiveTab] = useState<'overview' | 'references' | 'items'>('overview')

  // Query entity details
  const { data: entityDetail, isLoading: isLoadingDetail } = useQuery({
    queryKey: ['entity-detail', entity.id],
    queryFn: () => getEntityDetail(entity.id),
    enabled: isOpen,
  })

  // Query entity references
  const { data: references } = useQuery({
    queryKey: ['entity-references', entity.id],
    queryFn: () => getEntityReferences(entity.id),
    enabled: isOpen && activeTab === 'references',
  })

  // Query entity items
  const { data: items } = useQuery({
    queryKey: ['entity-items', entity.id],
    queryFn: () => getEntityItems(entity.id),
    enabled: isOpen && activeTab === 'items',
  })

  // Close modal on escape key
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose()
      }
    }

    if (isOpen) {
      document.addEventListener('keydown', handleEscape)
      document.body.style.overflow = 'hidden'
    }

    return () => {
      document.removeEventListener('keydown', handleEscape)
      document.body.style.overflow = 'unset'
    }
  }, [isOpen, onClose])

  if (!isOpen) return null

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

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-lg max-w-4xl w-full max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="border-b p-6">
          <div className="flex justify-between items-start">
            <div>
              <h2 className="text-2xl font-bold">{entity.entity_value}</h2>
              <div className="flex items-center gap-2 mt-2">
                <span
                  className={`px-3 py-1 text-sm rounded-full ${getEntityTypeColor(entity.entity_type)}`}
                >
                  {entity.entity_type}
                </span>
                <span className="text-sm text-gray-500">ID: {entity.id}</span>
              </div>
            </div>
            <Button variant="outline" onClick={onClose}>
              Close
            </Button>
          </div>
        </div>

        {/* Tabs */}
        <div className="border-b">
          <div className="flex">
            {[
              { key: 'overview', label: 'Overview' },
              { key: 'references', label: 'References' },
              { key: 'items', label: 'Items' },
            ].map(tab => (
              <button
                key={tab.key}
                onClick={() => setActiveTab(tab.key as 'overview' | 'references' | 'items')}
                className={`px-6 py-3 text-sm font-medium border-b-2 transition-colors ${
                  activeTab === tab.key
                    ? 'border-blue-500 text-blue-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700'
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-auto p-6">
          {activeTab === 'overview' && (
            <div className="space-y-6">
              {isLoadingDetail ? (
                <div className="flex justify-center py-8">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
                </div>
              ) : entityDetail ? (
                <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                  <Card>
                    <CardHeader>
                      <CardTitle>References</CardTitle>
                    </CardHeader>
                    <CardContent>
                      <div className="text-3xl font-bold text-blue-600">
                        {entityDetail.references_count}
                      </div>
                      <p className="text-sm text-gray-600">Total references in text</p>
                    </CardContent>
                  </Card>

                  <Card>
                    <CardHeader>
                      <CardTitle>Items</CardTitle>
                    </CardHeader>
                    <CardContent>
                      <div className="text-3xl font-bold text-green-600">
                        {entityDetail.items_count}
                      </div>
                      <p className="text-sm text-gray-600">HN items mentioning this entity</p>
                    </CardContent>
                  </Card>

                  <Card>
                    <CardHeader>
                      <CardTitle>Relations</CardTitle>
                    </CardHeader>
                    <CardContent>
                      <div className="text-3xl font-bold text-purple-600">
                        {entityDetail.relations_count}
                      </div>
                      <p className="text-sm text-gray-600">Relationships with other entities</p>
                    </CardContent>
                  </Card>
                </div>
              ) : (
                <div className="text-center text-gray-500 py-8">
                  Failed to load entity details
                </div>
              )}
            </div>
          )}

          {activeTab === 'references' && (
            <div className="space-y-4">
              {references ? (
                <>
                  <div className="text-sm text-gray-600">
                    Showing {references.references.length} of {references.total_count} references
                  </div>
                  <div className="space-y-3">
                    {references.references.map(ref => (
                      <Card key={ref.id}>
                        <CardContent className="p-4">
                          <div className="text-sm font-mono bg-gray-50 p-3 rounded border">
                            {ref.original_text}
                          </div>
                          <div className="flex justify-between items-center mt-2 text-xs text-gray-500">
                            <span>Item ID: {ref.item_id}</span>
                            <span>
                              Position: {ref.start_offset}-{ref.end_offset}
                            </span>
                            {ref.confidence && (
                              <span>Confidence: {(ref.confidence * 100).toFixed(1)}%</span>
                            )}
                          </div>
                        </CardContent>
                      </Card>
                    ))}
                  </div>
                </>
              ) : (
                <div className="flex justify-center py-8">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
                </div>
              )}
            </div>
          )}

          {activeTab === 'items' && (
            <div className="space-y-4">
              {items ? (
                <>
                  <div className="flex justify-between items-center">
                    <div className="text-sm text-gray-600">
                      Showing {items.items.length} of {items.total_count} items
                    </div>
                    <HighlightLegend />
                  </div>
                  <div className="space-y-4">
                    {items.items.map(itemWithHighlights => (
                      <Card key={itemWithHighlights.item.id}>
                        <CardContent className="p-4">
                          <div className="flex justify-between items-start mb-3">
                            <div>
                              <h4 className="font-medium">
                                {itemWithHighlights.item.title || `Item ${itemWithHighlights.item.id}`}
                              </h4>
                              <div className="text-sm text-gray-500">
                                By: {itemWithHighlights.item.by || 'Unknown'} • 
                                Type: {itemWithHighlights.item.item_type}
                              </div>
                            </div>
                            <span className="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded">
                              {itemWithHighlights.highlights.length} highlights
                            </span>
                          </div>
                          
                          {itemWithHighlights.item.text && (
                            <div className="text-sm bg-gray-50 p-3 rounded border max-h-32 overflow-y-auto">
                              <TextHighlighter
                                text={itemWithHighlights.item.text}
                                highlights={itemWithHighlights.highlights}
                              />
                            </div>
                          )}
                          
                          {itemWithHighlights.item.url && (
                            <div className="mt-2">
                              <a
                                href={itemWithHighlights.item.url}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-blue-600 hover:underline text-sm"
                              >
                                View original →
                              </a>
                            </div>
                          )}
                        </CardContent>
                      </Card>
                    ))}
                  </div>
                </>
              ) : (
                <div className="flex justify-center py-8">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}