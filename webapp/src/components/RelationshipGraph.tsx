import { useState, useEffect, useMemo, useCallback } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  addEdge,
  type Connection,
  type Edge,
  type Node,
  type NodeTypes,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { Card, CardContent } from './ui/card'
import type { Entity } from '../types/database'
import type { GetRelationsResponse } from '../types/api'
import type { SearchEntitiesResponse } from '../types/api'
import type { GetEntitiesResponse } from '../types/api'

interface RelationshipGraphProps {
  searchQuery: string
  entityType: string
  onEntitySelect: (entity: Entity) => void
}

interface EntityNodeData {
  entity: Entity
  onSelect: (entity: Entity) => void
}

const API_BASE = 'http://localhost:8080/api'

// Custom Entity Node Component
function EntityNode({ data }: { data: EntityNodeData }) {
  const getEntityTypeColor = (type: string) => {
    const colors: Record<string, string> = {
      person: '#3B82F6',
      company: '#10B981',
      technology: '#8B5CF6',
      location: '#F59E0B',
      organization: '#EF4444',
      product: '#F59E0B',
      event: '#6366F1',
    }
    return colors[type] || '#6B7280'
  }

  return (
    <div
      className="px-4 py-2 shadow-md rounded-md bg-white border-2 cursor-pointer hover:shadow-lg transition-shadow"
      style={{
        borderColor: getEntityTypeColor(data.entity.entity_type),
        minWidth: '150px',
      }}
      onClick={() => data.onSelect(data.entity)}
    >
      <div className="font-medium text-sm truncate">{data.entity.entity_value}</div>
      <div
        className="text-xs px-2 py-1 rounded-full mt-1 text-center"
        style={{
          backgroundColor: getEntityTypeColor(data.entity.entity_type) + '20',
          color: getEntityTypeColor(data.entity.entity_type),
        }}
      >
        {data.entity.entity_type}
      </div>
    </div>
  )
}

const nodeTypes: NodeTypes = {
  entityNode: EntityNode,
}

async function getRelations(page = 1, limit = 100): Promise<GetRelationsResponse> {
  const params = new URLSearchParams({
    page: page.toString(),
    limit: limit.toString(),
  })

  const response = await fetch(`${API_BASE}/relations?${params}`)
  if (!response.ok) {
    throw new Error('Failed to fetch relations')
  }
  return response.json()
}

async function getEntities(page = 1, limit = 200): Promise<GetEntitiesResponse> {
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

async function searchEntities(query: string, entityType?: string): Promise<SearchEntitiesResponse> {
  const params = new URLSearchParams({
    q: query,
    limit: '100',
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

export function RelationshipGraph({ searchQuery, entityType, onEntitySelect }: RelationshipGraphProps) {
  const [nodes, setNodes, onNodesChange] = useNodesState([] as Node[])
  const [edges, setEdges, onEdgesChange] = useEdgesState([] as Edge[])
  const [selectedRelationType, setSelectedRelationType] = useState<string>('')

  // Fetch entities with performance optimizations
  const { data: entitiesData } = useQuery({
    queryKey: ['graph-entities', searchQuery, entityType],
    queryFn: (): Promise<SearchEntitiesResponse | GetEntitiesResponse> => {
      if (searchQuery.trim()) {
        return searchEntities(searchQuery, entityType || undefined)
      } else {
        return getEntities(1, 50) // Reduced limit for graph performance
      }
    },
    staleTime: 5 * 60 * 1000,
    gcTime: 10 * 60 * 1000,
  })

  // Fetch relations with performance optimizations
  const { data: relationsData } = useQuery({
    queryKey: ['graph-relations'],
    queryFn: () => getRelations(1, 100), // Reduced limit for graph performance
    staleTime: 5 * 60 * 1000,
    gcTime: 10 * 60 * 1000,
  })

  // Create graph from data
  const { graphNodes, graphEdges, relationTypes } = useMemo(() => {
    if (!entitiesData || !relationsData) {
      return { graphNodes: [], graphEdges: [], relationTypes: [] }
    }

    // Handle both SearchEntitiesResponse (EntityWithStats[]) and GetEntitiesResponse (Entity[])
    const entities = entitiesData.entities.map(entity => 
      'entity' in entity ? entity.entity : entity
    ) as Entity[]
    const relations = relationsData.relations

    // Filter relations if type is selected
    const filteredRelations = selectedRelationType
      ? relations.filter(rel => rel.relation_type === selectedRelationType)
      : relations

    // Create entity map for quick lookup
    const entityMap = new Map(entities.map(entity => [entity.id, entity]))

    // Filter relations to only include entities we have
    const validRelations = filteredRelations.filter(
      rel => entityMap.has(rel.subject_entity_id) && entityMap.has(rel.object_entity_id)
    )

    // Get all unique relation types
    const types = Array.from(new Set(relations.map(rel => rel.relation_type))).sort()

    // Create nodes with circular layout
    const radius = Math.min(300, Math.max(150, entities.length * 10))
    const centerX = 400
    const centerY = 300
    
    const graphNodes: Node[] = entities.map((entity: Entity, index: number) => {
      const angle = (2 * Math.PI * index) / entities.length
      const x = centerX + radius * Math.cos(angle)
      const y = centerY + radius * Math.sin(angle)

      return {
        id: entity.id.toString(),
        type: 'entityNode',
        position: { x, y },
        data: {
          entity,
          onSelect: onEntitySelect,
        },
        draggable: true,
      }
    })

    // Create edges
    const graphEdges: Edge[] = validRelations.map(relation => ({
      id: `${relation.subject_entity_id}-${relation.object_entity_id}-${relation.relation_type}`,
      source: relation.subject_entity_id.toString(),
      target: relation.object_entity_id.toString(),
      label: relation.relation_type,
      type: 'smoothstep',
      animated: true,
      style: {
        stroke: '#6B7280',
        strokeWidth: 2,
      },
      labelStyle: {
        fontSize: 10,
        fontWeight: 600,
      },
      labelBgStyle: {
        fill: '#FFFFFF',
        fillOpacity: 0.8,
      },
    }))

    return { graphNodes, graphEdges, relationTypes: types }
  }, [entitiesData, relationsData, selectedRelationType, onEntitySelect])

  // Update React Flow state when graph data changes
  useEffect(() => {
    setNodes(graphNodes)
    setEdges(graphEdges)
  }, [graphNodes, graphEdges, setNodes, setEdges])

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  )

  if (!entitiesData || !relationsData) {
    return (
      <div className="flex justify-center items-center h-96">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {/* Controls */}
      <div className="flex gap-4 items-center">
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium">Relation Type:</label>
          <select
            value={selectedRelationType}
            onChange={(e) => setSelectedRelationType(e.target.value)}
            className="border border-gray-300 rounded px-2 py-1 text-sm"
          >
            <option value="">All Relations</option>
            {relationTypes.map(type => (
              <option key={type} value={type}>
                {type}
              </option>
            ))}
          </select>
        </div>
        
        <div className="text-sm text-gray-600">
          {nodes.length} entities • {edges.length} relations
        </div>
      </div>

      {/* Graph Container */}
      <Card>
        <CardContent className="p-0">
          <div style={{ width: '100%', height: '600px' }}>
            <ReactFlow
              nodes={nodes}
              edges={edges}
              onNodesChange={onNodesChange}
              onEdgesChange={onEdgesChange}
              onConnect={onConnect}
              nodeTypes={nodeTypes}
              fitView
              attributionPosition="bottom-left"
            >
              <Controls />
              <MiniMap
                nodeColor={(node) => {
                  const entity = (node.data as unknown as EntityNodeData).entity
                  const colors: Record<string, string> = {
                    person: '#3B82F6',
                    company: '#10B981',
                    technology: '#8B5CF6',
                    location: '#F59E0B',
                    organization: '#EF4444',
                    product: '#F59E0B',
                    event: '#6366F1',
                  }
                  return colors[entity.entity_type] || '#6B7280'
                }}
                maskColor="rgb(240, 240, 240, 0.6)"
              />
              <Background color="#aaa" gap={16} />
            </ReactFlow>
          </div>
        </CardContent>
      </Card>

      {/* Legend */}
      <Card>
        <CardContent className="p-4">
          <h3 className="font-medium mb-3">Entity Types</h3>
          <div className="flex flex-wrap gap-3">
            {['person', 'company', 'technology', 'location', 'organization', 'product', 'event'].map(type => {
              const colors: Record<string, string> = {
                person: '#3B82F6',
                company: '#10B981',
                technology: '#8B5CF6',
                location: '#F59E0B',
                organization: '#EF4444',
                product: '#F59E0B',
                event: '#6366F1',
              }
              
              return (
                <div key={type} className="flex items-center gap-2">
                  <div
                    className="w-4 h-4 rounded border-2"
                    style={{ borderColor: colors[type], backgroundColor: colors[type] + '20' }}
                  />
                  <span className="text-sm capitalize">{type}</span>
                </div>
              )
            })}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}