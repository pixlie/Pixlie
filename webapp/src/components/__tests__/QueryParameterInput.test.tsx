import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { QueryParameterInput } from '../QueryBuilder/QueryParameterInput'
import type { QueryParameter } from '../QueryBuilder/QueryTemplate'

// Mock the UI components
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

vi.mock('../ui/button', () => ({
  Button: ({ children, onClick, variant, size }: { 
    children: React.ReactNode; 
    onClick?: () => void; 
    variant?: string; 
    size?: string;
  }) => (
    <button 
      onClick={onClick} 
      data-testid="button" 
      data-variant={variant} 
      data-size={size}
    >
      {children}
    </button>
  ),
}))

vi.mock('../ui/card', () => ({
  Card: ({ children, className }: { children: React.ReactNode; className?: string }) => (
    <div data-testid="card" className={className}>{children}</div>
  ),
  CardContent: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="card-content">{children}</div>
  ),
}))

describe('QueryParameterInput', () => {
  const mockOnChange = vi.fn()
  const mockOnEntitySearch = vi.fn()

  beforeEach(() => {
    mockOnChange.mockClear()
    mockOnEntitySearch.mockClear()
  })

  it('renders text input correctly', () => {
    const parameter: QueryParameter = {
      name: 'testParam',
      label: 'Test Parameter',
      type: 'text',
      required: true,
      placeholder: 'Enter text...',
    }

    render(
      <QueryParameterInput
        parameter={parameter}
        value=""
        onChange={mockOnChange}
      />
    )

    expect(screen.getByText('Test Parameter')).toBeInTheDocument()
    expect(screen.getByText('*')).toBeInTheDocument() // Required indicator
    expect(screen.getByPlaceholderText('Enter text...')).toBeInTheDocument()
  })

  it('renders number input correctly', () => {
    const parameter: QueryParameter = {
      name: 'numberParam',
      label: 'Number Parameter',
      type: 'number',
      required: false,
      validation: { min: 1, max: 100 },
    }

    render(
      <QueryParameterInput
        parameter={parameter}
        value={50}
        onChange={mockOnChange}
      />
    )

    const input = screen.getByTestId('input')
    expect(input).toHaveAttribute('type', 'number')
    expect(input).toHaveAttribute('min', '1')
    expect(input).toHaveAttribute('max', '100')
  })

  it('renders select input correctly', () => {
    const parameter: QueryParameter = {
      name: 'selectParam',
      label: 'Select Parameter',
      type: 'select',
      required: true,
      options: ['option1', 'option2', 'option3'],
    }

    render(
      <QueryParameterInput
        parameter={parameter}
        value="option1"
        onChange={mockOnChange}
      />
    )

    expect(screen.getByDisplayValue('option1')).toBeInTheDocument()
    expect(screen.getByText('Option1')).toBeInTheDocument()
    expect(screen.getByText('Option2')).toBeInTheDocument()
  })

  it('renders multiselect input correctly', () => {
    const parameter: QueryParameter = {
      name: 'multiselectParam',
      label: 'Multiselect Parameter',
      type: 'multiselect',
      required: false,
      options: ['tag1', 'tag2', 'tag3'],
    }

    render(
      <QueryParameterInput
        parameter={parameter}
        value={['tag1', 'tag3']}
        onChange={mockOnChange}
      />
    )

    expect(screen.getByText('Tag1')).toBeInTheDocument()
    expect(screen.getByText('Tag2')).toBeInTheDocument()
    expect(screen.getByText('Tag3')).toBeInTheDocument()
    expect(screen.getByText('Selected: tag1, tag3')).toBeInTheDocument()
  })

  it('handles multiselect toggle correctly', () => {
    const parameter: QueryParameter = {
      name: 'multiselectParam',
      label: 'Multiselect Parameter',
      type: 'multiselect',
      required: false,
      options: ['tag1', 'tag2', 'tag3'],
    }

    render(
      <QueryParameterInput
        parameter={parameter}
        value={['tag1']}
        onChange={mockOnChange}
      />
    )

    // Click on tag2 to add it
    fireEvent.click(screen.getByText('Tag2'))
    expect(mockOnChange).toHaveBeenCalledWith(['tag1', 'tag2'])

    // Reset mock
    mockOnChange.mockClear()

    // Click on tag1 to remove it
    fireEvent.click(screen.getByText('Tag1'))
    expect(mockOnChange).toHaveBeenCalledWith([])
  })

  it('renders entity input with search functionality', async () => {
    const parameter: QueryParameter = {
      name: 'entityParam',
      label: 'Entity Parameter',
      type: 'entity',
      required: true,
      placeholder: 'Search entities...',
    }

    mockOnEntitySearch.mockResolvedValue(['OpenAI', 'React', 'JavaScript'])

    render(
      <QueryParameterInput
        parameter={parameter}
        value=""
        onChange={mockOnChange}
        onEntitySearch={mockOnEntitySearch}
      />
    )

    const input = screen.getByPlaceholderText('Search entities...')
    
    // Type to trigger search
    fireEvent.change(input, { target: { value: 'Open' } })
    
    expect(mockOnEntitySearch).toHaveBeenCalledWith('Open')
    
    // Wait for suggestions to appear
    await waitFor(() => {
      expect(screen.getByText('OpenAI')).toBeInTheDocument()
    })
  })

  it('renders date range input correctly', () => {
    const parameter: QueryParameter = {
      name: 'dateParam',
      label: 'Date Range',
      type: 'date-range',
      required: false,
    }

    render(
      <QueryParameterInput
        parameter={parameter}
        value={{ start: '2023-01-01', end: '2023-12-31' }}
        onChange={mockOnChange}
      />
    )

    expect(screen.getByText('Start Date')).toBeInTheDocument()
    expect(screen.getByText('End Date')).toBeInTheDocument()
    expect(screen.getByDisplayValue('2023-01-01')).toBeInTheDocument()
    expect(screen.getByDisplayValue('2023-12-31')).toBeInTheDocument()
  })

  it('displays error messages', () => {
    const parameter: QueryParameter = {
      name: 'testParam',
      label: 'Test Parameter',
      type: 'text',
      required: true,
    }

    render(
      <QueryParameterInput
        parameter={parameter}
        value=""
        onChange={mockOnChange}
        error="This field is required"
      />
    )

    expect(screen.getByText('This field is required')).toBeInTheDocument()
  })

  it('displays validation message when no error', () => {
    const parameter: QueryParameter = {
      name: 'testParam',
      label: 'Test Parameter',
      type: 'text',
      required: true,
      validation: {
        message: 'Please enter a valid value',
      },
    }

    render(
      <QueryParameterInput
        parameter={parameter}
        value=""
        onChange={mockOnChange}
      />
    )

    expect(screen.getByText('Please enter a valid value')).toBeInTheDocument()
  })

  it('handles text input changes', () => {
    const parameter: QueryParameter = {
      name: 'testParam',
      label: 'Test Parameter',
      type: 'text',
      required: false,
    }

    render(
      <QueryParameterInput
        parameter={parameter}
        value=""
        onChange={mockOnChange}
      />
    )

    const input = screen.getByTestId('input')
    fireEvent.change(input, { target: { value: 'test value' } })
    
    expect(mockOnChange).toHaveBeenCalledWith('test value')
  })

  it('handles number input changes', () => {
    const parameter: QueryParameter = {
      name: 'numberParam',
      label: 'Number Parameter',
      type: 'number',
      required: false,
    }

    render(
      <QueryParameterInput
        parameter={parameter}
        value={0}
        onChange={mockOnChange}
      />
    )

    const input = screen.getByTestId('input')
    fireEvent.change(input, { target: { value: '42' } })
    
    expect(mockOnChange).toHaveBeenCalledWith(42)
  })

  it('handles select input changes', () => {
    const parameter: QueryParameter = {
      name: 'selectParam',
      label: 'Select Parameter',
      type: 'select',
      required: false,
      options: ['option1', 'option2'],
    }

    render(
      <QueryParameterInput
        parameter={parameter}
        value=""
        onChange={mockOnChange}
      />
    )

    const select = screen.getByRole('combobox')
    fireEvent.change(select, { target: { value: 'option1' } })
    
    expect(mockOnChange).toHaveBeenCalledWith('option1')
  })

  it('handles date range changes', () => {
    const parameter: QueryParameter = {
      name: 'dateParam',
      label: 'Date Range',
      type: 'date-range',
      required: false,
    }

    render(
      <QueryParameterInput
        parameter={parameter}
        value={{}}
        onChange={mockOnChange}
      />
    )

    const inputs = screen.getAllByTestId('input')
    const startInput = inputs[0]
    const endInput = inputs[1]

    // Change start date
    fireEvent.change(startInput, { target: { value: '2023-01-01' } })
    expect(mockOnChange).toHaveBeenCalledWith({ start: '2023-01-01' })

    // Reset mock
    mockOnChange.mockClear()

    // Change end date
    fireEvent.change(endInput, { target: { value: '2023-12-31' } })
    expect(mockOnChange).toHaveBeenCalledWith({ end: '2023-12-31' })
  })
})