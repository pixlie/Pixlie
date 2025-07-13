import { useParams, useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { useState } from 'react'
import { Button } from '../components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card'
import { EntityDetailModal } from '../components/EntityDetailModal'
import type { EntityDetailResponse } from '../types/api'

const API_BASE = 'http://localhost:8080/api'

async function getEntityDetail(entityId: bigint): Promise<EntityDetailResponse> {
  const response = await fetch(`${API_BASE}/entities/${entityId}`)
  if (!response.ok) {
    throw new Error('Failed to fetch entity details')
  }
  return response.json()
}

export function EntityDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [showModal, setShowModal] = useState(false)

  const entityId = id ? BigInt(id) : null

  const { data: entityDetail, isLoading, error } = useQuery({
    queryKey: ['entity-detail', entityId],
    queryFn: () => getEntityDetail(entityId!),
    enabled: !!entityId,
  })

  if (!entityId) {
    return (
      <div className="p-6">
        <div className="text-center text-red-600">
          <h1 className="text-2xl font-bold mb-4">Invalid Entity ID</h1>
          <Button onClick={() => navigate('/entities')}>
            Back to Entities
          </Button>
        </div>
      </div>
    )
  }

  if (isLoading) {
    return (
      <div className="p-6">
        <div className="flex justify-center items-center h-64">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
        </div>
      </div>
    )
  }

  if (error || !entityDetail) {
    return (
      <div className="p-6">
        <div className="text-center text-red-600">
          <h1 className="text-2xl font-bold mb-4">Entity Not Found</h1>
          <p className="mb-4">The entity with ID {entityId} could not be found.</p>
          <Button onClick={() => navigate('/entities')}>
            Back to Entities
          </Button>
        </div>
      </div>
    )
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

  return (
    <div className="p-6 max-w-6xl mx-auto">
      {/* Header */}
      <div className="mb-6">
        <div className="flex items-center gap-4 mb-4">
          <Button variant="outline" onClick={() => navigate('/entities')}>
            ← Back to Entities
          </Button>
          <Button onClick={() => setShowModal(true)}>
            View in Modal
          </Button>
        </div>
        
        <div className="flex items-start justify-between">
          <div>
            <h1 className="text-3xl font-bold mb-3">{entityDetail.entity.entity_value}</h1>
            <div className="flex items-center gap-3">
              <span
                className={`px-3 py-1 text-sm rounded-full ${getEntityTypeColor(entityDetail.entity.entity_type)}`}
              >
                {entityDetail.entity.entity_type}
              </span>
              <span className="text-sm text-gray-500">ID: {entityDetail.entity.id}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Statistics Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">References</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-blue-600 mb-2">
              {entityDetail.references_count}
            </div>
            <p className="text-sm text-gray-600">Total references in text</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Items</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-green-600 mb-2">
              {entityDetail.items_count}
            </div>
            <p className="text-sm text-gray-600">HN items mentioning this entity</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Relations</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-purple-600 mb-2">
              {entityDetail.relations_count}
            </div>
            <p className="text-sm text-gray-600">Relationships with other entities</p>
          </CardContent>
        </Card>
      </div>

      {/* Action Buttons */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card className="cursor-pointer hover:shadow-md transition-shadow">
          <CardContent className="p-6 text-center">
            <h3 className="text-lg font-medium mb-2">Explore References</h3>
            <p className="text-sm text-gray-600 mb-4">
              View all text references and their contexts
            </p>
            <Button onClick={() => setShowModal(true)}>
              View References
            </Button>
          </CardContent>
        </Card>

        <Card className="cursor-pointer hover:shadow-md transition-shadow">
          <CardContent className="p-6 text-center">
            <h3 className="text-lg font-medium mb-2">Relationship Graph</h3>
            <p className="text-sm text-gray-600 mb-4">
              Visualize connections with other entities
            </p>
            <Button 
              onClick={() => navigate(`/entities/graph?focus=${entityDetail.entity.id}`)}
              variant="outline"
            >
              View Graph
            </Button>
          </CardContent>
        </Card>
      </div>

      {/* Modal */}
      {showModal && (
        <EntityDetailModal
          entity={entityDetail.entity}
          isOpen={showModal}
          onClose={() => setShowModal(false)}
        />
      )}
    </div>
  )
}