import React, { useState, useEffect, useRef } from 'react';
import { Send, Loader2 } from 'lucide-react';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Card } from '../ui/card';
import { cn } from '../../lib/utils';
import { MessageList } from './MessageList';
import { TypingIndicator } from './TypingIndicator';
import { ToolExecutionDisplay } from './ToolExecutionDisplay';
import type { Conversation, ConversationStep } from '../../types/conversation';

interface ChatInterfaceProps {
  conversationId?: string;
  className?: string;
}

interface Message {
  id: string;
  type: 'user' | 'assistant';
  content: string;
  timestamp: Date;
  isStreaming?: boolean;
}

const API_BASE_URL = 'http://localhost:8080/api';

export function ChatInterface({ conversationId, className }: ChatInterfaceProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputValue, setInputValue] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isTyping, setIsTyping] = useState(false);
  const [currentConversation, setCurrentConversation] = useState<Conversation | null>(null);
  const [toolExecution, setToolExecution] = useState<ConversationStep | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  useEffect(() => {
    if (conversationId) {
      loadConversation(conversationId);
    }
  }, [conversationId]);

  const loadConversation = async (id: string) => {
    try {
      const response = await fetch(`${API_BASE_URL}/conversations/${id}`);
      if (response.ok) {
        const conversation: Conversation = await response.json();
        setCurrentConversation(conversation);
        
        // Convert conversation steps to messages
        const convertedMessages: Message[] = [];
        convertedMessages.push({
          id: `user-${conversation.id}`,
          type: 'user',
          content: conversation.user_query,
          timestamp: new Date(conversation.created_at),
        });

        conversation.steps.forEach((step, index) => {
          if (step.results && step.results.summary) {
            convertedMessages.push({
              id: `assistant-${conversation.id}-${index}`,
              type: 'assistant',
              content: step.results.summary,
              timestamp: new Date(conversation.updated_at),
            });
          }
        });

        setMessages(convertedMessages);
      }
    } catch (error) {
      console.error('Failed to load conversation:', error);
    }
  };

  const startNewConversation = async (query: string) => {
    try {
      const response = await fetch(`${API_BASE_URL}/conversations/start`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ user_query: query }),
      });

      if (response.ok) {
        const result = await response.json();
        setCurrentConversation(result.conversation);
        return result.conversation.id;
      }
    } catch (error) {
      console.error('Failed to start conversation:', error);
    }
    return null;
  };

  const continueConversation = async (conversationId: string, query: string) => {
    try {
      const response = await fetch(`${API_BASE_URL}/conversations/${conversationId}/continue`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ user_query: query }),
      });

      if (response.ok) {
        const result = await response.json();
        setCurrentConversation(result.conversation);
        return result.conversation;
      }
    } catch (error) {
      console.error('Failed to continue conversation:', error);
    }
    return null;
  };

  const handleSendMessage = async () => {
    if (!inputValue.trim() || isLoading) return;

    const userMessage: Message = {
      id: `user-${Date.now()}`,
      type: 'user',
      content: inputValue,
      timestamp: new Date(),
    };

    setMessages(prev => [...prev, userMessage]);
    setInputValue('');
    setIsLoading(true);
    setIsTyping(true);

    try {
      let conversationIdToUse = conversationId || currentConversation?.id;

      if (!conversationIdToUse) {
        conversationIdToUse = await startNewConversation(inputValue);
      } else {
        await continueConversation(conversationIdToUse, inputValue);
      }

      if (conversationIdToUse) {
        // Start streaming response
        const streamResponse = await fetch(`${API_BASE_URL}/conversations/${conversationIdToUse}/stream`, {
          method: 'GET',
          headers: {
            'Accept': 'text/event-stream',
          },
        });

        if (streamResponse.ok && streamResponse.body) {
          const reader = streamResponse.body.getReader();
          const decoder = new TextDecoder();
          const assistantMessage: Message = {
            id: `assistant-${Date.now()}`,
            type: 'assistant',
            content: '',
            timestamp: new Date(),
            isStreaming: true,
          };

          setMessages(prev => [...prev, assistantMessage]);
          setIsTyping(false);

          try {
            while (true) {
              const { done, value } = await reader.read();
              if (done) break;

              const chunk = decoder.decode(value);
              const lines = chunk.split('\n');

              for (const line of lines) {
                if (line.startsWith('data: ')) {
                  const data = line.slice(6);
                  if (data === '[DONE]') {
                    // Mark streaming as complete
                    setMessages(prev => prev.map(msg => 
                      msg.id === assistantMessage.id 
                        ? { ...msg, isStreaming: false }
                        : msg
                    ));
                    break;
                  }

                  try {
                    const parsed = JSON.parse(data);
                    if (parsed.type === 'content') {
                      assistantMessage.content += parsed.content;
                      setMessages(prev => prev.map(msg => 
                        msg.id === assistantMessage.id 
                          ? { ...msg, content: assistantMessage.content }
                          : msg
                      ));
                    } else if (parsed.type === 'tool_execution') {
                      setToolExecution(parsed.step);
                    }
                  } catch (e) {
                    console.error('Error parsing streaming data:', e);
                  }
                }
              }
            }
          } catch (error) {
            console.error('Error reading stream:', error);
          } finally {
            reader.releaseLock();
          }
        }
      }
    } catch (error) {
      console.error('Failed to send message:', error);
      setMessages(prev => [...prev, {
        id: `error-${Date.now()}`,
        type: 'assistant',
        content: 'Sorry, I encountered an error while processing your message. Please try again.',
        timestamp: new Date(),
      }]);
    } finally {
      setIsLoading(false);
      setIsTyping(false);
      setToolExecution(null);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  return (
    <Card className={cn('flex flex-col h-full', className)}>
      <div className="flex-1 overflow-hidden flex flex-col">
        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          <MessageList messages={messages} />
          {isTyping && <TypingIndicator />}
          {toolExecution && <ToolExecutionDisplay step={toolExecution} />}
          <div ref={messagesEndRef} />
        </div>
        
        <div className="border-t p-4">
          <div className="flex gap-2">
            <Input
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              onKeyPress={handleKeyPress}
              placeholder="Ask about entities, relations, or HN discussions..."
              disabled={isLoading}
              className="flex-1"
            />
            <Button
              onClick={handleSendMessage}
              disabled={!inputValue.trim() || isLoading}
              size="sm"
            >
              {isLoading ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Send className="h-4 w-4" />
              )}
            </Button>
          </div>
        </div>
      </div>
    </Card>
  );
}