import { useState } from 'react';
import { 
  Play, 
  CheckCircle, 
  XCircle, 
  Loader2, 
  Clock, 
  Eye,
  ChevronRight,
  ChevronDown,
  BarChart3,
  Zap
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Button } from '../ui/button';
import { cn } from '../../lib/utils';
import type { ConversationStep, StepStatus } from '../../types/conversation';
import type { ToolExecution } from '../../types/conversation';

interface ToolExecutionTimelineProps {
  steps: ConversationStep[];
  onStepClick?: (step: ConversationStep) => void;
  onToolClick?: (tool: ToolExecution, step: ConversationStep) => void;
  showMetrics?: boolean;
}

export function ToolExecutionTimeline({ 
  steps, 
  onStepClick, 
  onToolClick, 
  showMetrics = false 
}: ToolExecutionTimelineProps) {
  const [expandedSteps, setExpandedSteps] = useState<Set<number>>(new Set());
  const [hoveredStep, setHoveredStep] = useState<number | null>(null);

  const getStatusIcon = (status: StepStatus) => {
    switch (status) {
      case 'Pending':
        return <Loader2 className="w-4 h-4 animate-spin text-gray-500" />;
      case 'InProgress':
        return <Play className="w-4 h-4 text-blue-500" />;
      case 'Completed':
        return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'Failed':
        return <XCircle className="w-4 h-4 text-red-500" />;
      default:
        return <Loader2 className="w-4 h-4 text-gray-500" />;
    }
  };

  const getStatusColor = (status: StepStatus) => {
    switch (status) {
      case 'Pending':
        return 'border-gray-300 bg-gray-50';
      case 'InProgress':
        return 'border-blue-300 bg-blue-50';
      case 'Completed':
        return 'border-green-300 bg-green-50';
      case 'Failed':
        return 'border-red-300 bg-red-50';
      default:
        return 'border-gray-300 bg-gray-50';
    }
  };

  const getProgressPercentage = (step: ConversationStep) => {
    if (step.status === 'Completed') return 100;
    if (step.status === 'Failed') return 100;
    if (step.status === 'InProgress') return 50;
    return 0;
  };

  const toggleStepExpansion = (stepId: number) => {
    const newExpanded = new Set(expandedSteps);
    if (newExpanded.has(stepId)) {
      newExpanded.delete(stepId);
    } else {
      newExpanded.add(stepId);
    }
    setExpandedSteps(newExpanded);
  };

  const formatExecutionTime = (timeMs: bigint | null) => {
    if (!timeMs) return 'N/A';
    const ms = Number(timeMs);
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${(ms / 60000).toFixed(1)}m`;
  };

  const getToolMetrics = (step: ConversationStep) => {
    const totalTools = step.tool_calls?.length || 0;
    const completedTools = step.tool_calls?.filter(t => t.result !== null).length || 0;
    const failedTools = step.tool_calls?.filter(t => t.error !== null).length || 0;
    const totalTime = step.tool_calls?.reduce((sum, t) => 
      sum + (t.execution_time_ms ? Number(t.execution_time_ms) : 0), 0
    ) || 0;

    return { totalTools, completedTools, failedTools, totalTime };
  };

  return (
    <div className="space-y-4">
      {showMetrics && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <BarChart3 className="w-5 h-5" />
              Execution Overview
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-4 gap-4">
              <div className="text-center">
                <div className="text-2xl font-bold text-blue-600">
                  {steps.length}
                </div>
                <div className="text-sm text-gray-600">Total Steps</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-green-600">
                  {steps.filter(s => s.status === 'Completed').length}
                </div>
                <div className="text-sm text-gray-600">Completed</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-red-600">
                  {steps.filter(s => s.status === 'Failed').length}
                </div>
                <div className="text-sm text-gray-600">Failed</div>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold text-purple-600">
                  {steps.reduce((sum, s) => sum + (s.tool_calls?.length || 0), 0)}
                </div>
                <div className="text-sm text-gray-600">Total Tools</div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      <div className="relative">
        {/* Timeline line */}
        <div className="absolute left-6 top-0 bottom-0 w-0.5 bg-gray-200"></div>

        {steps.map((step) => {
          const isExpanded = expandedSteps.has(step.step_id);
          const isHovered = hoveredStep === step.step_id;
          const metrics = getToolMetrics(step);
          const progressPercentage = getProgressPercentage(step);

          return (
            <div
              key={step.step_id}
              className="relative mb-6"
              onMouseEnter={() => setHoveredStep(step.step_id)}
              onMouseLeave={() => setHoveredStep(null)}
            >
              {/* Timeline dot */}
              <div className="absolute left-4 top-3 w-4 h-4 rounded-full bg-white border-2 border-gray-300 flex items-center justify-center">
                <div className="w-2 h-2 rounded-full bg-gray-300"></div>
              </div>

              {/* Step card */}
              <Card className={cn(
                'ml-12 transition-all duration-200',
                isHovered && 'shadow-lg',
                getStatusColor(step.status)
              )}>
                <CardContent className="p-4">
                  {/* Step header */}
                  <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-3">
                      {getStatusIcon(step.status)}
                      <div>
                        <div className="font-medium text-sm">
                          Step {step.step_id} - {step.step_type}
                        </div>
                        <div className="text-xs text-gray-500">
                          {new Date(step.created_at).toLocaleString()}
                        </div>
                      </div>
                    </div>
                    
                    <div className="flex items-center gap-2">
                      {metrics.totalTools > 0 && (
                        <div className="flex items-center gap-1 text-xs text-gray-600">
                          <Zap className="w-3 h-3" />
                          {metrics.totalTools} tools
                        </div>
                      )}
                      
                      {metrics.totalTime > 0 && (
                        <div className="flex items-center gap-1 text-xs text-gray-600">
                          <Clock className="w-3 h-3" />
                          {formatExecutionTime(BigInt(metrics.totalTime))}
                        </div>
                      )}

                      {step.tool_calls && step.tool_calls.length > 0 && (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => toggleStepExpansion(step.step_id)}
                          className="p-1"
                        >
                          {isExpanded ? (
                            <ChevronDown className="w-4 h-4" />
                          ) : (
                            <ChevronRight className="w-4 h-4" />
                          )}
                        </Button>
                      )}

                      {onStepClick && (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => onStepClick(step)}
                          className="p-1"
                        >
                          <Eye className="w-4 h-4" />
                        </Button>
                      )}
                    </div>
                  </div>

                  {/* Progress bar */}
                  <div className="w-full bg-gray-200 rounded-full h-1 mb-3">
                    <div 
                      className={cn(
                        'h-1 rounded-full transition-all duration-500',
                        step.status === 'Completed' && 'bg-green-500',
                        step.status === 'Failed' && 'bg-red-500',
                        step.status === 'InProgress' && 'bg-blue-500',
                        step.status === 'Pending' && 'bg-gray-400'
                      )}
                      style={{ width: `${progressPercentage}%` }}
                    />
                  </div>

                  {/* Step summary */}
                  {step.results?.summary && (
                    <div className="text-sm text-gray-700 mb-3">
                      {step.results.summary}
                    </div>
                  )}

                  {/* Tool executions */}
                  {isExpanded && step.tool_calls && step.tool_calls.length > 0 && (
                    <div className="mt-4 space-y-3 border-t pt-3">
                      {step.tool_calls.map((tool, toolIndex) => (
                        <div 
                          key={toolIndex}
                          className="bg-white rounded-lg p-3 border border-gray-200 hover:border-gray-300 transition-colors"
                        >
                          <div className="flex items-center justify-between mb-2">
                            <div className="flex items-center gap-2">
                              <div className="text-sm font-medium">{tool.tool_name}</div>
                              {tool.execution_time_ms && (
                                <div className="text-xs text-gray-500">
                                  {formatExecutionTime(tool.execution_time_ms)}
                                </div>
                              )}
                            </div>
                            
                            {onToolClick && (
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => onToolClick(tool, step)}
                                className="p-1"
                              >
                                <Eye className="w-3 h-3" />
                              </Button>
                            )}
                          </div>

                          {/* Tool status */}
                          <div className="flex items-center gap-2 text-xs">
                            {tool.error ? (
                              <span className="text-red-600">❌ Failed</span>
                            ) : tool.result !== null ? (
                              <span className="text-green-600">✅ Completed</span>
                            ) : (
                              <span className="text-yellow-600">⏳ Running</span>
                            )}
                          </div>

                          {/* Tool error */}
                          {tool.error && (
                            <div className="mt-2 p-2 bg-red-50 rounded text-xs text-red-700">
                              {tool.error}
                            </div>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            </div>
          );
        })}
      </div>
    </div>
  );
}