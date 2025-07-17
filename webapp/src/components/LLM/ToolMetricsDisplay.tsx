import { useMemo } from 'react';
import { 
  BarChart3, 
  Clock, 
  CheckCircle, 
  XCircle, 
  Zap, 
  TrendingUp, 
  TrendingDown,
  Activity
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { cn } from '../../lib/utils';
import type { ConversationStep } from '../../types/conversation';

interface ToolMetricsDisplayProps {
  steps: ConversationStep[];
  className?: string;
}

export function ToolMetricsDisplay({ steps, className }: ToolMetricsDisplayProps) {
  const metrics = useMemo(() => {
    const allTools = steps.flatMap(step => step.tool_calls || []);
    
    const totalExecutions = allTools.length;
    const successfulExecutions = allTools.filter(tool => tool.result !== null && tool.error === null).length;
    const failedExecutions = allTools.filter(tool => tool.error !== null).length;
    
    const executionTimes = allTools
      .filter(tool => tool.execution_time_ms !== null)
      .map(tool => Number(tool.execution_time_ms));
    
    const averageExecutionTime = executionTimes.length > 0 
      ? executionTimes.reduce((sum, time) => sum + time, 0) / executionTimes.length
      : 0;
    
    const totalExecutionTime = executionTimes.reduce((sum, time) => sum + time, 0);
    
    // Tool usage statistics
    const toolUsage = allTools.reduce((acc, tool) => {
      acc[tool.tool_name] = (acc[tool.tool_name] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);
    
    const mostUsedTools = Object.entries(toolUsage)
      .sort(([, a], [, b]) => b - a)
      .slice(0, 5);
    
    // Performance categories
    const fastTools = allTools.filter(tool => 
      tool.execution_time_ms && Number(tool.execution_time_ms) < 1000
    ).length;
    
    const moderateTools = allTools.filter(tool => 
      tool.execution_time_ms && 
      Number(tool.execution_time_ms) >= 1000 && 
      Number(tool.execution_time_ms) < 5000
    ).length;
    
    const slowTools = allTools.filter(tool => 
      tool.execution_time_ms && Number(tool.execution_time_ms) >= 5000
    ).length;
    
    const successRate = totalExecutions > 0 ? (successfulExecutions / totalExecutions) * 100 : 0;
    
    return {
      totalExecutions,
      successfulExecutions,
      failedExecutions,
      averageExecutionTime,
      totalExecutionTime,
      mostUsedTools,
      fastTools,
      moderateTools,
      slowTools,
      successRate
    };
  }, [steps]);

  const formatExecutionTime = (timeMs: number) => {
    if (timeMs < 1000) return `${timeMs.toFixed(0)}ms`;
    if (timeMs < 60000) return `${(timeMs / 1000).toFixed(1)}s`;
    return `${(timeMs / 60000).toFixed(1)}m`;
  };

  const getSuccessRateColor = (rate: number) => {
    if (rate >= 90) return 'text-green-600';
    if (rate >= 70) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getSuccessRateIcon = (rate: number) => {
    if (rate >= 90) return <TrendingUp className="w-4 h-4 text-green-600" />;
    if (rate >= 70) return <Activity className="w-4 h-4 text-yellow-600" />;
    return <TrendingDown className="w-4 h-4 text-red-600" />;
  };

  return (
    <div className={cn('space-y-6', className)}>
      {/* Overview Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Executions</CardTitle>
            <Zap className="w-4 h-4 text-blue-600" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{metrics.totalExecutions}</div>
            <p className="text-xs text-gray-600">
              {metrics.successfulExecutions} successful
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Success Rate</CardTitle>
            {getSuccessRateIcon(metrics.successRate)}
          </CardHeader>
          <CardContent>
            <div className={cn('text-2xl font-bold', getSuccessRateColor(metrics.successRate))}>
              {metrics.successRate.toFixed(1)}%
            </div>
            <p className="text-xs text-gray-600">
              {metrics.failedExecutions} failed
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Avg. Execution Time</CardTitle>
            <Clock className="w-4 h-4 text-purple-600" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {formatExecutionTime(metrics.averageExecutionTime)}
            </div>
            <p className="text-xs text-gray-600">
              Total: {formatExecutionTime(metrics.totalExecutionTime)}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Performance</CardTitle>
            <BarChart3 className="w-4 h-4 text-green-600" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">
              {metrics.fastTools}
            </div>
            <p className="text-xs text-gray-600">
              fast executions
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Performance Distribution */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="w-5 h-5" />
            Performance Distribution
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {/* Fast Tools */}
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 bg-green-500 rounded-full"></div>
                <span className="text-sm">Fast (&lt;1s)</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-32 bg-gray-200 rounded-full h-2">
                  <div 
                    className="bg-green-500 h-2 rounded-full"
                    style={{ 
                      width: `${metrics.totalExecutions > 0 ? (metrics.fastTools / metrics.totalExecutions) * 100 : 0}%` 
                    }}
                  />
                </div>
                <span className="text-sm font-medium w-12 text-right">
                  {metrics.fastTools}
                </span>
              </div>
            </div>

            {/* Moderate Tools */}
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 bg-yellow-500 rounded-full"></div>
                <span className="text-sm">Moderate (1-5s)</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-32 bg-gray-200 rounded-full h-2">
                  <div 
                    className="bg-yellow-500 h-2 rounded-full"
                    style={{ 
                      width: `${metrics.totalExecutions > 0 ? (metrics.moderateTools / metrics.totalExecutions) * 100 : 0}%` 
                    }}
                  />
                </div>
                <span className="text-sm font-medium w-12 text-right">
                  {metrics.moderateTools}
                </span>
              </div>
            </div>

            {/* Slow Tools */}
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 bg-red-500 rounded-full"></div>
                <span className="text-sm">Slow (&gt;5s)</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-32 bg-gray-200 rounded-full h-2">
                  <div 
                    className="bg-red-500 h-2 rounded-full"
                    style={{ 
                      width: `${metrics.totalExecutions > 0 ? (metrics.slowTools / metrics.totalExecutions) * 100 : 0}%` 
                    }}
                  />
                </div>
                <span className="text-sm font-medium w-12 text-right">
                  {metrics.slowTools}
                </span>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Most Used Tools */}
      {metrics.mostUsedTools.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Zap className="w-5 h-5" />
              Most Used Tools
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {metrics.mostUsedTools.map(([toolName, count], index) => (
                <div key={toolName} className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <div className={cn(
                      'w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold text-white',
                      index === 0 && 'bg-yellow-500',
                      index === 1 && 'bg-gray-400',
                      index === 2 && 'bg-orange-600',
                      index > 2 && 'bg-gray-300'
                    )}>
                      {index + 1}
                    </div>
                    <span className="text-sm font-medium">{toolName}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <div className="w-24 bg-gray-200 rounded-full h-2">
                      <div 
                        className="bg-blue-500 h-2 rounded-full"
                        style={{ 
                          width: `${(count / metrics.mostUsedTools[0][1]) * 100}%` 
                        }}
                      />
                    </div>
                    <span className="text-sm font-medium w-8 text-right">
                      {count}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Status Summary */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <CheckCircle className="w-5 h-5" />
            Execution Status
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-3 gap-4">
            <div className="text-center">
              <div className="text-2xl font-bold text-green-600">
                {metrics.successfulExecutions}
              </div>
              <div className="text-sm text-gray-600 flex items-center justify-center gap-1">
                <CheckCircle className="w-3 h-3" />
                Successful
              </div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-red-600">
                {metrics.failedExecutions}
              </div>
              <div className="text-sm text-gray-600 flex items-center justify-center gap-1">
                <XCircle className="w-3 h-3" />
                Failed
              </div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-600">
                {metrics.totalExecutions - metrics.successfulExecutions - metrics.failedExecutions}
              </div>
              <div className="text-sm text-gray-600 flex items-center justify-center gap-1">
                <Clock className="w-3 h-3" />
                Pending
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}