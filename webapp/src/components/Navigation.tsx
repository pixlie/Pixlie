import { Settings } from "lucide-react";
import { Button } from "./ui/button";

interface NavigationProps {
  onSettingsClick: () => void;
}

export function Navigation({ onSettingsClick }: NavigationProps) {
  return (
    <nav className="w-64 h-screen bg-gray-900 text-white flex flex-col">
      <div className="p-4">
        <h1 className="text-xl font-bold">Pixlie</h1>
        <p className="text-sm text-gray-400">HN Entity Analysis</p>
      </div>
      
      <div className="flex-1 px-4">
        <ul className="space-y-2">
          <li>
            <Button
              variant="ghost"
              className="w-full justify-start text-white hover:bg-gray-800"
            >
              Dashboard
            </Button>
          </li>
          <li>
            <Button
              variant="ghost"
              className="w-full justify-start text-white hover:bg-gray-800"
            >
              Downloads
            </Button>
          </li>
          <li>
            <Button
              variant="ghost"
              className="w-full justify-start text-white hover:bg-gray-800"
            >
              Analytics
            </Button>
          </li>
        </ul>
      </div>
      
      <div className="p-4 border-t border-gray-700">
        <Button
          variant="ghost"
          className="w-full justify-start text-white hover:bg-gray-800"
          onClick={onSettingsClick}
        >
          <Settings className="w-4 h-4 mr-2" />
          Settings
        </Button>
      </div>
    </nav>
  );
}