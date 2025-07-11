import { Settings as SettingsIcon, LayoutDashboard, Download, BarChart3 } from "lucide-react"; // Renamed Settings to SettingsIcon
import { Link, useLocation } from "react-router-dom";
import { cn } from "../lib/utils"; // For conditional classes

// Re-using Button styling but as a Link
const NavLink = ({ to, children, icon: Icon }: { to: string; children: React.ReactNode; icon?: React.ElementType }) => {
  const location = useLocation();
  const isActive = location.pathname === to;

  return (
    <Link
      to={to}
      className={cn(
        "flex items-center w-full justify-start text-white py-2 px-3 rounded-md text-sm font-medium",
        isActive ? "bg-gray-800" : "hover:bg-gray-800 hover:text-white"
      )}
    >
      {Icon && <Icon className="w-4 h-4 mr-2" />}
      {children}
    </Link>
  );
};

export function Navigation() {
  return (
    <nav className="w-64 h-screen bg-gray-900 text-white flex flex-col">
      <div className="p-4">
        <h1 className="text-xl font-bold">Pixlie</h1>
        <p className="text-sm text-gray-400">HN Entity Analysis</p>
      </div>
      
      <div className="flex-1 px-4">
        <ul className="space-y-2">
          <li>
            <NavLink to="/" icon={LayoutDashboard}>Dashboard</NavLink>
          </li>
          <li>
            {/* These are not part of the issue, so they won't navigate for now */}
            <button
              className="flex items-center w-full justify-start text-white py-2 px-3 rounded-md text-sm font-medium hover:bg-gray-800 hover:text-white opacity-50 cursor-not-allowed"
              disabled
            >
              <Download className="w-4 h-4 mr-2" />
              Downloads
            </button>
          </li>
          <li>
            <button
              className="flex items-center w-full justify-start text-white py-2 px-3 rounded-md text-sm font-medium hover:bg-gray-800 hover:text-white opacity-50 cursor-not-allowed"
              disabled
            >
              <BarChart3 className="w-4 h-4 mr-2" />
              Analytics
            </button>
          </li>
        </ul>
      </div>
      
      <div className="p-4 border-t border-gray-700">
        <NavLink to="/settings" icon={SettingsIcon}>Settings</NavLink>
      </div>
    </nav>
  );
}