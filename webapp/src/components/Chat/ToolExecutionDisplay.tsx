import { Play, CheckCircle, XCircle, Loader2 } from 'lucide-react';
import { Card } from '../ui/card';
import { cn } from '../../lib/utils';
import type { ConversationStep, StepStatus } from '../../types/conversation';

interface ToolExecutionDisplayProps {
  step: ConversationStep;
}

export function ToolExecutionDisplay({ step }: ToolExecutionDisplayProps) {
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
        return 'border-gray-200 bg-gray-50';
      case 'InProgress':
        return 'border-blue-200 bg-blue-50';
      case 'Completed':
        return 'border-green-200 bg-green-50';
      case 'Failed':
        return 'border-red-200 bg-red-50';
      default:
        return 'border-gray-200 bg-gray-50';
    }
  };

  const getStatusText = (status: StepStatus) => {
    switch (status) {
      case 'Pending':
        return 'Pending';
      case 'InProgress':
        return 'Running';
      case 'Completed':
        return 'Completed';
      case 'Failed':
        return 'Failed';
      default:
        return 'Unknown';
    }
  };

  return (
    <Card className={cn('p-3 border-l-4', getStatusColor(step.status))}>
      <div className="flex items-center gap-2 mb-2">
        {getStatusIcon(step.status)}
        <span className="text-sm font-medium">Tool Execution</span>
        <span className="text-xs text-gray-500">
          {getStatusText(step.status)}
        </span>
      </div>
      
      {step.tool_calls && step.tool_calls.length > 0 && (
        <div className="space-y-2">
          {step.tool_calls.map((toolCall, index) => (
            <div key={index} className="space-y-2">
              <div className="text-sm">
                <span className="font-medium">Tool:</span> {toolCall.tool_name}
              </div>
              
              {toolCall.parameters !== null && toolCall.parameters !== undefined && (
                <div className="text-sm">
                  <span className="font-medium">Parameters:</span>
                  <pre className="mt-1 text-xs bg-gray-100 p-2 rounded overflow-x-auto">
                    {JSON.stringify(toolCall.parameters, null, 2)}
                  </pre>
                </div>
              )}
              
              {toolCall.result !== null && toolCall.result !== undefined && (
                <div className="text-sm">
                  <span className="font-medium">Result:</span>
                  <div className="mt-1 text-xs bg-gray-100 p-2 rounded">
                    {JSON.stringify(toolCall.result, null, 2)}
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
      
      {step.results && step.results.summary && (
        <div className="text-sm">
          <span className="font-medium">Summary:</span>
          <div className="mt-1 text-xs bg-gray-100 p-2 rounded">
            {step.results.summary}
          </div>
        </div>
      )}
      
      {step.status === 'InProgress' && (
        <div className="mt-2 flex items-center gap-2 text-xs text-blue-600">
          <Loader2 className="w-3 h-3 animate-spin" />
          Executing tool...
        </div>
      )}
    </Card>
  );
}