import { Bot } from 'lucide-react';

export function TypingIndicator() {
  return (
    <div className="flex gap-3 p-3 rounded-lg bg-gray-50 border-l-4 border-gray-500 mr-8">
      <div className="flex-shrink-0">
        <div className="w-8 h-8 bg-gray-500 rounded-full flex items-center justify-center">
          <Bot className="w-4 h-4 text-white" />
        </div>
      </div>
      
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <span className="text-sm font-medium">Assistant</span>
          <div className="flex items-center gap-1 text-xs text-gray-500">
            <div className="w-2 h-2 bg-gray-500 rounded-full animate-pulse" />
            Thinking...
          </div>
        </div>
        
        <div className="flex items-center gap-1 text-sm text-gray-600">
          <div className="flex gap-1">
            <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '0ms' }} />
            <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '150ms' }} />
            <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '300ms' }} />
          </div>
        </div>
      </div>
    </div>
  );
}