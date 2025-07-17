import { ChatInterface } from '../components/Chat';

export function ChatPage() {
  return (
    <div className="h-screen p-4 bg-gray-50">
      <div className="max-w-4xl mx-auto h-full">
        <div className="bg-white rounded-lg shadow-sm h-full">
          <div className="p-6 border-b">
            <h1 className="text-2xl font-bold text-gray-900">Chat with Pixlie</h1>
            <p className="text-gray-600 mt-1">
              Ask questions about entities, relations, and Hacker News discussions
            </p>
          </div>
          
          <div className="h-full pb-6">
            <ChatInterface className="h-full border-0 shadow-none" />
          </div>
        </div>
      </div>
    </div>
  );
}