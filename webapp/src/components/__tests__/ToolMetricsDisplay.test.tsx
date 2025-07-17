import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { ToolMetricsDisplay } from '../LLM/ToolMetricsDisplay'
import type { ConversationStep } from '../../types/conversation'

// Mock the UI components
vi.mock('../ui/card', () => ({
  Card: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="card">{children}</div>
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

const mockSteps: ConversationStep[] = [
  {
    step_id: 1,
    step_type: 'Planning',
    llm_request: 'Test request',
    llm_response: 'Test response',
    tool_calls: [
      {
        tool_name: 'search_tool',
        parameters: { query: 'test' },
        result: { data: 'test result' },
        error: null,
        execution_time_ms: BigInt(500), // Fast
      },
      {
        tool_name: 'database_tool',
        parameters: { query: 'test' },
        result: { data: 'test result' },
        error: null,
        execution_time_ms: BigInt(2500), // Moderate
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
        execution_time_ms: BigInt(6000), // Slow
      },
    ],
    results: null,
    status: 'Failed',
    created_at: '2023-01-01T00:01:00Z',
  },
]

describe('ToolMetricsDisplay', () => {
  it('displays total executions correctly', () => {
    render(<ToolMetricsDisplay steps={mockSteps} />)
    
    expect(screen.getByText('3')).toBeInTheDocument() // Total executions
    expect(screen.getByText('Total Executions')).toBeInTheDocument()
  })

  it('calculates success rate correctly', () => {
    render(<ToolMetricsDisplay steps={mockSteps} />)
    
    // 2 successful out of 3 total = 66.7%
    expect(screen.getByText('66.7%')).toBeInTheDocument()
    expect(screen.getByText('Success Rate')).toBeInTheDocument()
  })

  it('displays average execution time', () => {
    render(<ToolMetricsDisplay steps={mockSteps} />)
    
    // (500 + 2500 + 6000) / 3 = 3000ms = 3.0s
    expect(screen.getByText('3.0s')).toBeInTheDocument()
    expect(screen.getByText('Avg. Execution Time')).toBeInTheDocument()
  })

  it('shows performance distribution', () => {
    render(<ToolMetricsDisplay steps={mockSteps} />)
    
    expect(screen.getByText('Performance Distribution')).toBeInTheDocument()
    expect(screen.getByText('Fast (<1s)')).toBeInTheDocument()
    expect(screen.getByText('Moderate (1-5s)')).toBeInTheDocument()
    expect(screen.getByText('Slow (>5s)')).toBeInTheDocument()
  })

  it('displays most used tools', () => {
    render(<ToolMetricsDisplay steps={mockSteps} />)
    
    expect(screen.getByText('Most Used Tools')).toBeInTheDocument()
    expect(screen.getByText('search_tool')).toBeInTheDocument()
    expect(screen.getByText('database_tool')).toBeInTheDocument()
  })

  it('shows execution status summary', () => {
    render(<ToolMetricsDisplay steps={mockSteps} />)
    
    expect(screen.getByText('Execution Status')).toBeInTheDocument()
    expect(screen.getByText('Successful')).toBeInTheDocument()
    expect(screen.getByText('Failed')).toBeInTheDocument()
    expect(screen.getByText('Pending')).toBeInTheDocument()
  })

  it('handles empty steps array', () => {
    render(<ToolMetricsDisplay steps={[]} />)
    
    expect(screen.getByText('0')).toBeInTheDocument() // Total executions
    expect(screen.getByText('0.0%')).toBeInTheDocument() // Success rate
  })

  it('handles steps with no tool calls', () => {
    const stepsWithNoTools: ConversationStep[] = [
      {
        step_id: 1,
        step_type: 'Planning',
        llm_request: 'Test request',
        llm_response: 'Test response',
        tool_calls: [],
        results: null,
        status: 'Completed',
        created_at: '2023-01-01T00:00:00Z',
      },
    ]
    
    render(<ToolMetricsDisplay steps={stepsWithNoTools} />)
    
    expect(screen.getByText('0')).toBeInTheDocument() // Total executions
    expect(screen.getByText('0.0%')).toBeInTheDocument() // Success rate
  })

  it('formats execution times correctly', () => {
    const stepsWithVariousTimes: ConversationStep[] = [
      {
        step_id: 1,
        step_type: 'Planning',
        llm_request: 'Test request',
        llm_response: 'Test response',
        tool_calls: [
          {
            tool_name: 'fast_tool',
            parameters: {},
            result: {},
            error: null,
            execution_time_ms: BigInt(500), // 500ms
          },
          {
            tool_name: 'slow_tool',
            parameters: {},
            result: {},
            error: null,
            execution_time_ms: BigInt(65000), // 65s = 1.1m
          },
        ],
        results: null,
        status: 'Completed',
        created_at: '2023-01-01T00:00:00Z',
      },
    ]
    
    render(<ToolMetricsDisplay steps={stepsWithVariousTimes} />)
    
    // Should show average time in appropriate format
    expect(screen.getByText('Total:')).toBeInTheDocument()
  })

  it('categorizes performance correctly', () => {
    render(<ToolMetricsDisplay steps={mockSteps} />)
    
    // Should have 1 fast, 1 moderate, 1 slow tool
    expect(screen.getByText('1')).toBeInTheDocument() // Fast tools count
  })
})