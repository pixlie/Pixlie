import { useState, useEffect } from 'react';
import { 
  CheckCircle, 
  XCircle, 
  Clock, 
  Code, 
  FileText,
  Activity,
  Zap,
  Database,
  Hash,
  X
} from 'lucide-react';
import { Button } from '../ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { cn } from '../../lib/utils';
import type { ToolExecution, ConversationStep } from '../../types/conversation';

interface ToolExecutionModalProps {
  tool: ToolExecution;
  step: ConversationStep;
  isOpen: boolean;
  onClose: () => void;
}

export function ToolExecutionModal({ tool, step, isOpen, onClose }: ToolExecutionModalProps) {
  const [activeTab, setActiveTab] = useState<'overview' | 'parameters' | 'result' | 'performance'>('overview');

  // Close modal on escape key
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    if (isOpen) {
      document.addEventListener('keydown', handleEscape);
      document.body.style.overflow = 'hidden';
    }

    return () => {
      document.removeEventListener('keydown', handleEscape);
      document.body.style.overflow = 'unset';
    };
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  const formatExecutionTime = (timeMs: bigint | null) => {
    if (!timeMs) return 'N/A';
    const ms = Number(timeMs);
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${(ms / 60000).toFixed(1)}m`;
  };

  const getToolIcon = (toolName: string) => {
    const iconMap: Record<string, React.ReactNode> = {
      'database': <Database className="w-5 h-5" />,
      'search': <Hash className="w-5 h-5" />,
      'api': <Activity className="w-5 h-5" />,
      'file': <FileText className="w-5 h-5" />,
      'code': <Code className="w-5 h-5" />,
    };

    // Try to match tool name to icon
    const lowerName = toolName.toLowerCase();
    for (const [key, icon] of Object.entries(iconMap)) {
      if (lowerName.includes(key)) {
        return icon;
      }
    }

    return <Zap className="w-5 h-5" />;
  };

  const getStatusIcon = () => {
    if (tool.error) {
      return <XCircle className="w-5 h-5 text-red-500" />;
    }
    if (tool.result !== null) {
      return <CheckCircle className="w-5 h-5 text-green-500" />;
    }
    return <Clock className="w-5 h-5 text-yellow-500" />;
  };

  const getStatusText = () => {
    if (tool.error) return 'Failed';
    if (tool.result !== null) return 'Completed';
    return 'Running';
  };

  const getStatusColor = () => {
    if (tool.error) return 'text-red-600 bg-red-50';
    if (tool.result !== null) return 'text-green-600 bg-green-50';
    return 'text-yellow-600 bg-yellow-50';
  };

  const formatJson = (obj: unknown) => {
    try {
      return JSON.stringify(obj, null, 2);
    } catch {
      return String(obj);
    }
  };

  const getResultType = (result: unknown) => {
    if (result === null || result === undefined) return 'null';
    if (Array.isArray(result)) return 'array';
    if (typeof result === 'object') return 'object';
    return typeof result;
  };

  const getResultSize = (result: unknown) => {
    if (result === null || result === undefined) return 0;
    if (Array.isArray(result)) return result.length;
    if (typeof result === 'object') return Object.keys(result as object).length;
    return String(result).length;
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-lg max-w-6xl w-full max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="border-b p-6">
          <div className="flex justify-between items-start">
            <div className="flex items-center gap-3">
              {getToolIcon(tool.tool_name)}
              <div>
                <h2 className="text-2xl font-bold">{tool.tool_name}</h2>
                <div className="flex items-center gap-3 mt-2">
                  <div className={cn('px-3 py-1 text-sm rounded-full flex items-center gap-2', getStatusColor())}>
                    {getStatusIcon()}
                    {getStatusText()}
                  </div>
                  <span className="text-sm text-gray-500">
                    Step {step.step_id} • {new Date(step.created_at).toLocaleString()}
                  </span>
                </div>
              </div>
            </div>
            <Button variant="ghost" onClick={onClose} className="p-2">
              <X className="w-5 h-5" />
            </Button>
          </div>
        </div>

        {/* Tabs */}
        <div className="border-b">
          <div className="flex">
            {[
              { key: 'overview', label: 'Overview', icon: <Activity className="w-4 h-4" /> },
              { key: 'parameters', label: 'Parameters', icon: <Code className="w-4 h-4" /> },
              { key: 'result', label: 'Result', icon: <FileText className="w-4 h-4" /> },
              { key: 'performance', label: 'Performance', icon: <Clock className="w-4 h-4" /> },
            ].map(tab => (
              <button
                key={tab.key}
                onClick={() => setActiveTab(tab.key as typeof activeTab)}
                className={cn(
                  'px-6 py-3 text-sm font-medium border-b-2 transition-colors flex items-center gap-2',
                  activeTab === tab.key
                    ? 'border-blue-500 text-blue-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700'
                )}
              >
                {tab.icon}
                {tab.label}
              </button>
            ))}
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-auto p-6">
          {activeTab === 'overview' && (
            <div className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <Clock className="w-5 h-5" />
                      Execution Time
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="text-3xl font-bold text-blue-600">
                      {formatExecutionTime(tool.execution_time_ms)}
                    </div>
                    <p className="text-sm text-gray-600">Total duration</p>
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <FileText className="w-5 h-5" />
                      Result Type
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="text-3xl font-bold text-green-600">
                      {getResultType(tool.result)}
                    </div>
                    <p className="text-sm text-gray-600">Data type returned</p>
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <Hash className="w-5 h-5" />
                      Result Size
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="text-3xl font-bold text-purple-600">
                      {getResultSize(tool.result)}
                    </div>
                    <p className="text-sm text-gray-600">
                      {Array.isArray(tool.result) ? 'items' : 'properties'}
                    </p>
                  </CardContent>
                </Card>
              </div>

              {tool.error && (
                <Card className="border-red-200">
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2 text-red-600">
                      <XCircle className="w-5 h-5" />
                      Error Details
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="bg-red-50 p-4 rounded-lg">
                      <pre className="text-sm text-red-700 whitespace-pre-wrap">
                        {tool.error}
                      </pre>
                    </div>
                  </CardContent>
                </Card>
              )}

              {step.results?.summary && (
                <Card>
                  <CardHeader>
                    <CardTitle>Step Summary</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <p className="text-gray-700">{step.results.summary}</p>
                  </CardContent>
                </Card>
              )}
            </div>
          )}

          {activeTab === 'parameters' && (
            <div className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Code className="w-5 h-5" />
                    Input Parameters
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  {tool.parameters ? (
                    <pre className="text-sm bg-gray-50 p-4 rounded-lg overflow-auto">
                      {formatJson(tool.parameters)}
                    </pre>
                  ) : (
                    <div className="text-center text-gray-500 py-8">
                      No parameters provided
                    </div>
                  )}
                </CardContent>
              </Card>
            </div>
          )}

          {activeTab === 'result' && (
            <div className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <FileText className="w-5 h-5" />
                    Tool Result
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  {tool.result !== null ? (
                    <pre className="text-sm bg-gray-50 p-4 rounded-lg overflow-auto max-h-96">
                      {formatJson(tool.result)}
                    </pre>
                  ) : (
                    <div className="text-center text-gray-500 py-8">
                      No result available
                    </div>
                  )}
                </CardContent>
              </Card>
            </div>
          )}

          {activeTab === 'performance' && (
            <div className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <Card>
                  <CardHeader>
                    <CardTitle>Execution Metrics</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="flex justify-between items-center">
                      <span className="text-sm text-gray-600">Start Time</span>
                      <span className="text-sm font-medium">
                        {new Date(step.created_at).toLocaleTimeString()}
                      </span>
                    </div>
                    <div className="flex justify-between items-center">
                      <span className="text-sm text-gray-600">Duration</span>
                      <span className="text-sm font-medium">
                        {formatExecutionTime(tool.execution_time_ms)}
                      </span>
                    </div>
                    <div className="flex justify-between items-center">
                      <span className="text-sm text-gray-600">Status</span>
                      <span className={cn('text-sm font-medium', getStatusColor())}>
                        {getStatusText()}
                      </span>
                    </div>
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader>
                    <CardTitle>Data Metrics</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="flex justify-between items-center">
                      <span className="text-sm text-gray-600">Result Type</span>
                      <span className="text-sm font-medium">
                        {getResultType(tool.result)}
                      </span>
                    </div>
                    <div className="flex justify-between items-center">
                      <span className="text-sm text-gray-600">Result Size</span>
                      <span className="text-sm font-medium">
                        {getResultSize(tool.result)} {Array.isArray(tool.result) ? 'items' : 'properties'}
                      </span>
                    </div>
                    <div className="flex justify-between items-center">
                      <span className="text-sm text-gray-600">Has Error</span>
                      <span className="text-sm font-medium">
                        {tool.error ? 'Yes' : 'No'}
                      </span>
                    </div>
                  </CardContent>
                </Card>
              </div>

              {tool.execution_time_ms && (
                <Card>
                  <CardHeader>
                    <CardTitle>Performance Analysis</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-4">
                      <div className="flex items-center justify-between">
                        <span className="text-sm text-gray-600">Performance Rating</span>
                        <div className="flex items-center gap-2">
                          {Number(tool.execution_time_ms) < 1000 && (
                            <span className="text-green-600 text-sm">⚡ Fast</span>
                          )}
                          {Number(tool.execution_time_ms) >= 1000 && Number(tool.execution_time_ms) < 5000 && (
                            <span className="text-yellow-600 text-sm">⏳ Moderate</span>
                          )}
                          {Number(tool.execution_time_ms) >= 5000 && (
                            <span className="text-red-600 text-sm">🐌 Slow</span>
                          )}
                        </div>
                      </div>
                      
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div
                          className={cn(
                            'h-2 rounded-full transition-all duration-300',
                            Number(tool.execution_time_ms) < 1000 && 'bg-green-500',
                            Number(tool.execution_time_ms) >= 1000 && Number(tool.execution_time_ms) < 5000 && 'bg-yellow-500',
                            Number(tool.execution_time_ms) >= 5000 && 'bg-red-500'
                          )}
                          style={{ 
                            width: `${Math.min(100, (Number(tool.execution_time_ms) / 10000) * 100)}%` 
                          }}
                        />
                      </div>
                      
                      <div className="text-xs text-gray-500">
                        Execution time: {formatExecutionTime(tool.execution_time_ms)}
                      </div>
                    </div>
                  </CardContent>
                </Card>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}