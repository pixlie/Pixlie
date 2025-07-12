import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { TextHighlighter, HighlightLegend } from '../TextHighlighter'
import type { EntityReference } from '../../types/EntityReference'

describe('TextHighlighter', () => {
  const mockText = "This is a test sentence with multiple words to highlight."
  
  const mockHighlights: EntityReference[] = [
    {
      id: BigInt(1),
      item_id: BigInt(1),
      entity_id: BigInt(1),
      original_text: "test",
      start_offset: BigInt(10),
      end_offset: BigInt(14),
      confidence: 0.9,
      created_at: new Date().toISOString(),
    },
    {
      id: BigInt(2),
      item_id: BigInt(1),
      entity_id: BigInt(2),
      original_text: "words",
      start_offset: BigInt(39),
      end_offset: BigInt(44),
      confidence: 0.7,
      created_at: new Date().toISOString(),
    },
  ]

  it('renders text without highlights', () => {
    render(<TextHighlighter text={mockText} highlights={[]} />)
    expect(screen.getByText(mockText)).toBeInTheDocument()
  })

  it('highlights text segments correctly', () => {
    render(<TextHighlighter text={mockText} highlights={mockHighlights} />)
    
    // Check that highlighted words are marked
    expect(screen.getByText('test')).toBeInTheDocument()
    expect(screen.getByText('words')).toBeInTheDocument()
    
    // Check that non-highlighted text is still present
    expect(screen.getByText(/This is a/)).toBeInTheDocument()
    expect(screen.getByText(/sentence with multiple/)).toBeInTheDocument()
  })

  it('applies correct confidence-based styling', () => {
    render(<TextHighlighter text={mockText} highlights={mockHighlights} />)
    
    const testHighlight = screen.getByText('test')
    const wordsHighlight = screen.getByText('words')
    
    // High confidence (0.9) should have green styling
    expect(testHighlight).toHaveClass('bg-green-200', 'border-green-400')
    
    // Medium confidence (0.7) should have blue styling  
    expect(wordsHighlight).toHaveClass('bg-blue-200', 'border-blue-400')
  })

  it('includes confidence information in title attribute', () => {
    render(<TextHighlighter text={mockText} highlights={mockHighlights} />)
    
    const testHighlight = screen.getByText('test')
    expect(testHighlight).toHaveAttribute('title', expect.stringContaining('Confidence: 90.0%'))
  })

  it('handles overlapping highlights gracefully', () => {
    const overlappingHighlights: EntityReference[] = [
      {
        id: BigInt(1),
        item_id: BigInt(1),
        entity_id: BigInt(1),
        original_text: "test sentence",
        start_offset: BigInt(10),
        end_offset: BigInt(23),
        confidence: 0.9,
        created_at: new Date().toISOString(),
      },
      {
        id: BigInt(2),
        item_id: BigInt(1),
        entity_id: BigInt(2),
        original_text: "sentence with",
        start_offset: BigInt(15),
        end_offset: BigInt(28),
        confidence: 0.7,
        created_at: new Date().toISOString(),
      },
    ]

    render(<TextHighlighter text={mockText} highlights={overlappingHighlights} />)
    
    // Should still render without crashing
    expect(screen.getByText(/test sentence/)).toBeInTheDocument()
  })

  it('handles empty highlights array', () => {
    render(<TextHighlighter text={mockText} highlights={[]} />)
    expect(screen.getByText(mockText)).toBeInTheDocument()
  })

  it('handles highlights with no confidence', () => {
    const noConfidenceHighlights: EntityReference[] = [
      {
        id: BigInt(1),
        item_id: BigInt(1),
        entity_id: BigInt(1),
        original_text: "test",
        start_offset: BigInt(10),
        end_offset: BigInt(14),
        confidence: null,
        created_at: new Date().toISOString(),
      },
    ]

    render(<TextHighlighter text={mockText} highlights={noConfidenceHighlights} />)
    
    const highlight = screen.getByText('test')
    // Should use default yellow styling for no confidence
    expect(highlight).toHaveClass('bg-yellow-200', 'border-yellow-400')
  })
})

describe('HighlightLegend', () => {
  it('renders confidence levels', () => {
    render(<HighlightLegend />)
    
    expect(screen.getByText('Highlight Confidence')).toBeInTheDocument()
    expect(screen.getByText('High (80%+)')).toBeInTheDocument()
    expect(screen.getByText('Good (60%+)')).toBeInTheDocument()
    expect(screen.getByText('Fair (40%+)')).toBeInTheDocument()
    expect(screen.getByText('Low (<40%)')).toBeInTheDocument()
  })
})