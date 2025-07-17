import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ToolExecutionTimeline } from '../LLM/ToolExecutionTimeline'
import type { ConversationStep } from '../../types/conversation'

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
  Button: ({ children, onClick, variant, size }: { children: React.ReactNode; onClick?: () => void; variant?: string; size?: string }) => (
    <button onClick={onClick} data-testid="button" data-variant={variant} data-size={size}>
      {children}
    </button>
  ),
}))

const mockSteps: ConversationStep[] = [
  {
    step_id: 1,
    step_type: 'Planning',
    llm_request: 'Test request',
    llm_response: 'Test response',
    tool_calls: [
      {
        tool_name: 'test_tool',
        parameters: { query: 'test' },
        result: { data: 'test result' },
        error: null,
        execution_time_ms: BigInt(500),
      },
    ],
    results: {
      data: null,
      summary: 'Test summary',
      next_action: null,
    },
    status: 'Completed',
    created_at: '2023-01-01T00:00:00Z',
  },
  {
    step_id: 2,
    step_type: 'ToolExecution',
    llm_request: null,
    llm_response: null,
    tool_calls: [
      {
        tool_name: 'search_tool',
        parameters: { query: 'search' },
        result: null,
        error: 'Tool failed',
        execution_time_ms: BigInt(1000),
      },
    ],
    results: null,
    status: 'Failed',
    created_at: '2023-01-01T00:01:00Z',
  },
]

describe('ToolExecutionTimeline', () => {
  it('renders steps with correct status icons', () => {
    render(<ToolExecutionTimeline steps={mockSteps} />)
    
    expect(screen.getByText('Step 1 - Planning')).toBeInTheDocument()
    expect(screen.getByText('Step 2 - ToolExecution')).toBeInTheDocument()
  })

  it('displays tool execution metrics', () => {
    render(<ToolExecutionTimeline steps={mockSteps} />)
    
    expect(screen.getByText('1 tools')).toBeInTheDocument()
    expect(screen.getByText('500ms')).toBeInTheDocument()
  })

  it('shows metrics overview when showMetrics is true', () => {
    render(<ToolExecutionTimeline steps={mockSteps} showMetrics={true} />)
    
    expect(screen.getByText('Execution Overview')).toBeInTheDocument()
    expect(screen.getByText('Total Steps')).toBeInTheDocument()
    expect(screen.getByText('Completed')).toBeInTheDocument()
    expect(screen.getByText('Failed')).toBeInTheDocument()
  })

  it('expands step details when clicked', () => {
    render(<ToolExecutionTimeline steps={mockSteps} />)
    
    // Find and click the expand button for the first step
    const expandButtons = screen.getAllByTestId('button')
    const expandButton = expandButtons.find(btn => btn.textContent?.includes('›'))
    
    if (expandButton) {
      fireEvent.click(expandButton)
      expect(screen.getByText('test_tool')).toBeInTheDocument()
    }
  })

  it('calls onStepClick when step is clicked', () => {
    const onStepClick = vi.fn()
    render(<ToolExecutionTimeline steps={mockSteps} onStepClick={onStepClick} />)
    
    // Find and click the eye button
    const eyeButtons = screen.getAllByTestId('button')
    const eyeButton = eyeButtons.find(btn => btn.textContent?.includes('👁'))
    
    if (eyeButton) {
      fireEvent.click(eyeButton)
      expect(onStepClick).toHaveBeenCalledWith(mockSteps[0])
    }
  })

  it('calls onToolClick when tool is clicked', () => {
    const onToolClick = vi.fn()
    render(<ToolExecutionTimeline steps={mockSteps} onToolClick={onToolClick} />)
    
    // Expand the first step to show tools
    const expandButtons = screen.getAllByTestId('button')
    const expandButton = expandButtons.find(btn => btn.textContent?.includes('›'))
    
    if (expandButton) {
      fireEvent.click(expandButton)
      
      // Find and click the tool eye button
      const toolEyeButtons = screen.getAllByTestId('button')
      const toolEyeButton = toolEyeButtons.find(btn => btn.textContent?.includes('👁'))
      
      if (toolEyeButton) {
        fireEvent.click(toolEyeButton)
        expect(onToolClick).toHaveBeenCalledWith(mockSteps[0].tool_calls[0], mockSteps[0])
      }
    }
  })

  it('displays error state for failed tools', () => {
    render(<ToolExecutionTimeline steps={mockSteps} />)
    
    // Expand the second step to show the failed tool
    const expandButtons = screen.getAllByTestId('button')
    const expandButton = expandButtons[1] // Second step expand button
    
    if (expandButton) {
      fireEvent.click(expandButton)
      expect(screen.getByText('❌ Failed')).toBeInTheDocument()
      expect(screen.getByText('Tool failed')).toBeInTheDocument()
    }
  })

  it('formats execution time correctly', () => {
    const stepsWithDifferentTimes: ConversationStep[] = [
      {
        ...mockSteps[0],
        tool_calls: [
          {
            ...mockSteps[0].tool_calls[0],
            execution_time_ms: BigInt(500), // 500ms
          },
        ],
      },
      {
        ...mockSteps[1],
        tool_calls: [
          {
            ...mockSteps[1].tool_calls[0],
            execution_time_ms: BigInt(2500), // 2.5s
          },
        ],
      },
    ]
    
    render(<ToolExecutionTimeline steps={stepsWithDifferentTimes} />)
    
    expect(screen.getByText('500ms')).toBeInTheDocument()
    expect(screen.getByText('2.5s')).toBeInTheDocument()
  })

  it('handles empty steps array', () => {
    render(<ToolExecutionTimeline steps={[]} />)
    
    // Should not crash and should render empty timeline
    expect(screen.queryByText('Step 1')).not.toBeInTheDocument()
  })
})