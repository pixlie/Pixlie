import { useMemo } from 'react'
import type { EntityReference } from '../types/database'

interface TextHighlighterProps {
  text: string
  highlights: EntityReference[]
  className?: string
}

interface HighlightSegment {
  text: string
  isHighlighted: boolean
  reference?: EntityReference
}

export function TextHighlighter({ text, highlights, className = '' }: TextHighlighterProps) {
  const segments = useMemo(() => {
    if (!highlights.length) {
      return [{ text, isHighlighted: false }]
    }

    // Sort highlights by start position
    const sortedHighlights = [...highlights].sort((a, b) => Number(a.start_offset) - Number(b.start_offset))
    
    const segments: HighlightSegment[] = []
    let currentPos = 0

    for (const highlight of sortedHighlights) {
      const start = Number(highlight.start_offset)
      const end = Number(highlight.end_offset)

      // Add text before highlight if there's a gap
      if (currentPos < start) {
        segments.push({
          text: text.slice(currentPos, start),
          isHighlighted: false,
        })
      }

      // Add highlighted text
      if (start < text.length && end <= text.length && start < end) {
        segments.push({
          text: text.slice(start, end),
          isHighlighted: true,
          reference: highlight,
        })
        currentPos = Math.max(currentPos, end)
      }
    }

    // Add remaining text
    if (currentPos < text.length) {
      segments.push({
        text: text.slice(currentPos),
        isHighlighted: false,
      })
    }

    return segments
  }, [text, highlights])

  const getHighlightColor = (confidence?: number | null) => {
    if (!confidence) return 'bg-yellow-200 border-yellow-400'
    
    if (confidence >= 0.8) return 'bg-green-200 border-green-400'
    if (confidence >= 0.6) return 'bg-blue-200 border-blue-400'
    if (confidence >= 0.4) return 'bg-yellow-200 border-yellow-400'
    return 'bg-red-200 border-red-400'
  }

  return (
    <div className={`leading-relaxed ${className}`}>
      {segments.map((segment, index) => {
        if (!segment.isHighlighted) {
          return <span key={index}>{segment.text}</span>
        }

        return (
          <mark
            key={index}
            className={`px-1 py-0.5 rounded border-b-2 transition-all hover:shadow-sm ${getHighlightColor(segment.reference?.confidence)}`}
            title={
              segment.reference
                ? `Entity ID: ${segment.reference.entity_id}${
                    segment.reference.confidence
                      ? ` • Confidence: ${(segment.reference.confidence * 100).toFixed(1)}%`
                      : ''
                  }`
                : undefined
            }
          >
            {segment.text}
          </mark>
        )
      })}
    </div>
  )
}

// Utility component for showing highlight legend
export function HighlightLegend() {
  return (
    <div className="bg-gray-50 p-3 rounded-md text-sm">
      <h4 className="font-medium mb-2">Highlight Confidence</h4>
      <div className="flex flex-wrap gap-3">
        <div className="flex items-center gap-1">
          <div className="w-4 h-3 bg-green-200 border-b-2 border-green-400 rounded"></div>
          <span>High (80%+)</span>
        </div>
        <div className="flex items-center gap-1">
          <div className="w-4 h-3 bg-blue-200 border-b-2 border-blue-400 rounded"></div>
          <span>Good (60%+)</span>
        </div>
        <div className="flex items-center gap-1">
          <div className="w-4 h-3 bg-yellow-200 border-b-2 border-yellow-400 rounded"></div>
          <span>Fair (40%+)</span>
        </div>
        <div className="flex items-center gap-1">
          <div className="w-4 h-3 bg-red-200 border-b-2 border-red-400 rounded"></div>
          <span>Low (&lt;40%)</span>
        </div>
      </div>
    </div>
  )
}