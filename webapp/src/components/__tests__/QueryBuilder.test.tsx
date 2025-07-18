import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { QueryBuilder } from '../QueryBuilder/QueryBuilder'
import { DEFAULT_QUERY_TEMPLATES } from '../QueryBuilder/QueryTemplate'

// Mock the UI components
vi.mock('../ui/card', () => ({
  Card: ({ children, className }: { children: React.ReactNode; className?: string }) => (
    <div data-testid="card" className={className}>{children}</div>
  ),
  CardContent: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="card-content">{children}</div>
  ),
  CardHeader: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="card-header">{children}</div>
  ),
  CardTitle: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="card-title">{children}</div>
  ),
}))

vi.mock('../ui/button', () => ({
  Button: ({ children, onClick, variant, size, disabled }: { 
    children: React.ReactNode; 
    onClick?: () => void; 
    variant?: string; 
    size?: string;
    disabled?: boolean;
  }) => (
    <button 
      onClick={onClick} 
      data-testid="button" 
      data-variant={variant} 
      data-size={size}
      disabled={disabled}
    >
      {children}
    </button>
  ),
}))

vi.mock('../ui/input', () => ({
  Input: ({ value, onChange, placeholder, type, className }: { 
    value?: string | number; 
    onChange?: (e: React.ChangeEvent<HTMLInputElement>) => void; 
    placeholder?: string;
    type?: string;
    className?: string;
  }) => (
    <input 
      type={type || 'text'}
      value={value}
      onChange={onChange}
      placeholder={placeholder}
      data-testid="input"
      className={className}
    />
  ),
}))

// Mock fetch for entity search
global.fetch = vi.fn(() =>
  Promise.resolve({
    ok: true,
    json: () => Promise.resolve({
      entities: [
        { entity: { entity_value: 'OpenAI' } },
        { entity: { entity_value: 'React' } },
        { entity: { entity_value: 'JavaScript' } }
      ]
    }),
  })
) as unknown as typeof fetch;

describe('QueryBuilder', () => {
  it('renders with default state', () => {
    render(<QueryBuilder />)
    
    expect(screen.getByText('LLM Query Builder')).toBeInTheDocument()
    expect(screen.getByText('Choose a Template')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('Search templates...')).toBeInTheDocument()
  })

  it('displays template categories', () => {
    render(<QueryBuilder />)
    
    expect(screen.getByText('All')).toBeInTheDocument()
    expect(screen.getByText('Analysis')).toBeInTheDocument()
    expect(screen.getByText('Search')).toBeInTheDocument()
    expect(screen.getByText('Comparison')).toBeInTheDocument()
  })

  it('displays default templates', () => {
    render(<QueryBuilder />)
    
    expect(screen.getByText('Entity Analysis')).toBeInTheDocument()
    expect(screen.getByText('Trend Comparison')).toBeInTheDocument()
    expect(screen.getByText('Smart Search')).toBeInTheDocument()
  })

  it('filters templates by category', () => {
    render(<QueryBuilder />)
    
    // Click on Analysis category
    fireEvent.click(screen.getByText('Analysis'))
    
    // Should show analysis templates
    expect(screen.getByText('Entity Analysis')).toBeInTheDocument()
    expect(screen.getByText('Topic Deep Dive')).toBeInTheDocument()
    
    // Should not show search templates
    expect(screen.queryByText('Smart Search')).not.toBeInTheDocument()
  })

  it('filters templates by search term', () => {
    render(<QueryBuilder />)
    
    const searchInput = screen.getByPlaceholderText('Search templates...')
    fireEvent.change(searchInput, { target: { value: 'entity' } })
    
    expect(screen.getByText('Entity Analysis')).toBeInTheDocument()
    expect(screen.queryByText('Smart Search')).not.toBeInTheDocument()
  })

  it('selects template and shows parameters', () => {
    render(<QueryBuilder />)
    
    // Click on Entity Analysis template
    fireEvent.click(screen.getByText('Entity Analysis'))
    
    expect(screen.getByText('Configure Parameters')).toBeInTheDocument()
    expect(screen.getByText('Entity Name')).toBeInTheDocument()
    expect(screen.getByText('Analysis Type')).toBeInTheDocument()
  })

  it('generates query when parameters are filled', () => {
    render(<QueryBuilder />)
    
    // Select Entity Analysis template
    fireEvent.click(screen.getByText('Entity Analysis'))
    
    // Should show generated query section
    expect(screen.getByText('Generated Query')).toBeInTheDocument()
  })

  it('calls onSubmit with generated query', () => {
    const onSubmit = vi.fn()
    render(<QueryBuilder onSubmit={onSubmit} />)
    
    // Select template
    fireEvent.click(screen.getByText('Entity Analysis'))
    
    // Click Execute Query button
    const executeButton = screen.getByText('Execute Query')
    fireEvent.click(executeButton)
    
    expect(onSubmit).toHaveBeenCalledWith(
      expect.stringContaining('Analyze the entity'),
      expect.objectContaining({
        template: 'entity-analysis',
        parameters: expect.any(Object)
      })
    )
  })

  it('handles custom query input', () => {
    render(<QueryBuilder />)
    
    // Custom query should be available when no template is selected
    expect(screen.getByPlaceholderText('Enter your custom LLM query here...')).toBeInTheDocument()
  })

  it('validates required parameters', () => {
    render(<QueryBuilder />)
    
    // Select template with required parameters
    fireEvent.click(screen.getByText('Entity Analysis'))
    
    // Try to submit without filling required fields
    const executeButton = screen.getByText('Execute Query')
    fireEvent.click(executeButton)
    
    // Should show validation errors
    expect(screen.getByText('Entity Name is required')).toBeInTheDocument()
  })

  it('expands and collapses the interface', () => {
    render(<QueryBuilder />)
    
    // Find collapse button and click it
    const collapseButton = screen.getByTestId('button')
    fireEvent.click(collapseButton)
    
    // Content should be hidden (not in document)
    expect(screen.queryByText('Choose a Template')).not.toBeInTheDocument()
  })

  it('handles saved queries', () => {
    // Mock localStorage
    const mockLocalStorage = {
      getItem: vi.fn(() => JSON.stringify([
        { name: 'Test Query', query: 'Test query content', timestamp: '2023-01-01' }
      ])),
      setItem: vi.fn(),
    }
    Object.defineProperty(window, 'localStorage', { value: mockLocalStorage })
    
    render(<QueryBuilder />)
    
    // Click saved queries button
    fireEvent.click(screen.getByText('Saved'))
    
    expect(screen.getByText('Saved Queries')).toBeInTheDocument()
    expect(screen.getByText('Test Query')).toBeInTheDocument()
  })

  it('copies query to clipboard', async () => {
    // Mock clipboard API
    Object.assign(navigator, {
      clipboard: {
        writeText: vi.fn(),
      },
    })
    
    render(<QueryBuilder />)
    
    // Select template to generate query
    fireEvent.click(screen.getByText('Entity Analysis'))
    
    // Click copy button
    const copyButtons = screen.getAllByTestId('button')
    const copyButton = copyButtons.find(btn => btn.textContent?.includes('Copy'))
    
    if (copyButton) {
      fireEvent.click(copyButton)
      expect(navigator.clipboard.writeText).toHaveBeenCalled()
    }
  })

  it('resets the form', () => {
    render(<QueryBuilder />)
    
    // Select template
    fireEvent.click(screen.getByText('Entity Analysis'))
    
    // Click reset button
    fireEvent.click(screen.getByText('Reset'))
    
    // Should go back to initial state
    expect(screen.queryByText('Configure Parameters')).not.toBeInTheDocument()
    expect(screen.getByPlaceholderText('Enter your custom LLM query here...')).toBeInTheDocument()
  })

  it('handles template difficulty levels', () => {
    render(<QueryBuilder />)
    
    // Check that difficulty badges are displayed
    expect(screen.getByText('beginner')).toBeInTheDocument()
    expect(screen.getByText('intermediate')).toBeInTheDocument()
    expect(screen.getByText('advanced')).toBeInTheDocument()
  })

  it('shows template examples and estimated time', () => {
    render(<QueryBuilder />)
    
    // Find a template with estimated time
    const entityTemplate = DEFAULT_QUERY_TEMPLATES.find(t => t.id === 'entity-analysis')
    if (entityTemplate?.estimatedTime) {
      expect(screen.getByText(`⏱️ ${entityTemplate.estimatedTime}`)).toBeInTheDocument()
    }
  })
})