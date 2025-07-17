import { User, Bot, Clock } from 'lucide-react';
import { cn } from '../../lib/utils';

interface Message {
  id: string;
  type: 'user' | 'assistant';
  content: string;
  timestamp: Date;
  isStreaming?: boolean;
}

interface MessageListProps {
  messages: Message[];
}

export function MessageList({ messages }: MessageListProps) {
  const formatTime = (timestamp: Date) => {
    return new Intl.DateTimeFormat('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      hour12: true,
    }).format(timestamp);
  };

  return (
    <div className="space-y-4">
      {messages.map((message) => (
        <div
          key={message.id}
          className={cn(
            'flex gap-3 p-3 rounded-lg',
            message.type === 'user' 
              ? 'bg-blue-50 border-l-4 border-blue-500 ml-8' 
              : 'bg-gray-50 border-l-4 border-gray-500 mr-8'
          )}
        >
          <div className="flex-shrink-0">
            {message.type === 'user' ? (
              <div className="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center">
                <User className="w-4 h-4 text-white" />
              </div>
            ) : (
              <div className="w-8 h-8 bg-gray-500 rounded-full flex items-center justify-center">
                <Bot className="w-4 h-4 text-white" />
              </div>
            )}
          </div>
          
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <span className="text-sm font-medium">
                {message.type === 'user' ? 'You' : 'Assistant'}
              </span>
              <div className="flex items-center gap-1 text-xs text-gray-500">
                <Clock className="w-3 h-3" />
                {formatTime(message.timestamp)}
              </div>
              {message.isStreaming && (
                <div className="flex items-center gap-1 text-xs text-blue-500">
                  <div className="w-2 h-2 bg-blue-500 rounded-full animate-pulse" />
                  Streaming...
                </div>
              )}
            </div>
            
            <div className="text-sm text-gray-800 whitespace-pre-wrap">
              {message.content}
              {message.isStreaming && (
                <span className="inline-block w-2 h-4 bg-gray-400 ml-1 animate-pulse" />
              )}
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}