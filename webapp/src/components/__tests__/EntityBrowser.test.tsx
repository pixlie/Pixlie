import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { EntityBrowser } from '../EntityBrowser'

// Mock the child components
vi.mock('../EntitySearchBar', () => ({
  EntitySearchBar: ({ onQueryChange, onEntityTypeChange }: any) => (
    <div data-testid="search-bar">
      <button onClick={() => onQueryChange('test query')}>Update Query</button>
      <button onClick={() => onEntityTypeChange('person')}>Update Type</button>
    </div>
  ),
}))

vi.mock('../EntityList', () => ({
  EntityList: ({ searchQuery, entityType, onEntitySelect }: any) => (
    <div data-testid="entity-list">
      <span>Query: {searchQuery}</span>
      <span>Type: {entityType}</span>
      <button onClick={() => onEntitySelect({ id: 1, entity_value: 'Test Entity', entity_type: 'person' })}>
        Select Entity
      </button>
    </div>
  ),
}))

vi.mock('../RelationshipGraph', () => ({
  RelationshipGraph: ({ searchQuery, entityType }: any) => (
    <div data-testid="relationship-graph">
      <span>Query: {searchQuery}</span>
      <span>Type: {entityType}</span>
    </div>
  ),
}))

vi.mock('../EntityDetailModal', () => ({
  EntityDetailModal: ({ entity, isOpen, onClose }: any) => (
    isOpen ? (
      <div data-testid="entity-modal">
        <span>Entity: {entity.entity_value}</span>
        <button onClick={onClose}>Close</button>
      </div>
    ) : null
  ),
}))

const createTestQueryClient = () => new QueryClient({
  defaultOptions: {
    queries: { retry: false },
    mutations: { retry: false },
  },
})

const renderWithQuery = (component: React.ReactElement) => {
  const queryClient = createTestQueryClient()
  return render(
    <QueryClientProvider client={queryClient}>
      {component}
    </QueryClientProvider>
  )
}

describe('EntityBrowser', () => {
  it('renders with default list mode', () => {
    renderWithQuery(<EntityBrowser />)
    
    expect(screen.getByText('Entity Browser')).toBeInTheDocument()
    expect(screen.getByText('List View')).toBeInTheDocument()
    expect(screen.getByText('Graph View')).toBeInTheDocument()
    expect(screen.getByTestId('entity-list')).toBeInTheDocument()
    expect(screen.queryByTestId('relationship-graph')).not.toBeInTheDocument()
  })

  it('switches to graph mode when button is clicked', () => {
    const onModeChange = vi.fn()
    renderWithQuery(<EntityBrowser mode="list" onModeChange={onModeChange} />)
    
    fireEvent.click(screen.getByText('Graph View'))
    expect(onModeChange).toHaveBeenCalledWith('graph')
  })

  it('shows graph view when mode is graph', () => {
    renderWithQuery(<EntityBrowser mode="graph" />)
    
    expect(screen.getByTestId('relationship-graph')).toBeInTheDocument()
    expect(screen.queryByTestId('entity-list')).not.toBeInTheDocument()
  })

  it('opens entity detail modal when entity is selected', () => {
    renderWithQuery(<EntityBrowser mode="list" />)
    
    fireEvent.click(screen.getByText('Select Entity'))
    expect(screen.getByTestId('entity-modal')).toBeInTheDocument()
    expect(screen.getByText('Entity: Test Entity')).toBeInTheDocument()
  })

  it('closes entity detail modal when close is clicked', () => {
    renderWithQuery(<EntityBrowser mode="list" />)
    
    fireEvent.click(screen.getByText('Select Entity'))
    expect(screen.getByTestId('entity-modal')).toBeInTheDocument()
    
    fireEvent.click(screen.getByText('Close'))
    expect(screen.queryByTestId('entity-modal')).not.toBeInTheDocument()
  })

  it('updates search query when search bar changes', () => {
    renderWithQuery(<EntityBrowser mode="list" />)
    
    fireEvent.click(screen.getByText('Update Query'))
    expect(screen.getByText('Query: test query')).toBeInTheDocument()
  })

  it('updates entity type when filter changes', () => {
    renderWithQuery(<EntityBrowser mode="list" />)
    
    fireEvent.click(screen.getByText('Update Type'))
    expect(screen.getByText('Type: person')).toBeInTheDocument()
  })
})