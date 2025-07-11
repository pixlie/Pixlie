import { useState, useEffect } from "react";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { Play, Square, FolderOpen, BarChart3 } from "lucide-react";

interface DownloadStats {
  total_items: number;
  total_users: number;
  last_download_time: string | null;
  items_downloaded_today: number;
  download_errors: number;
  is_downloading: boolean;
}

interface Config {
  config_path: string;
  data_folder: string | null;
  download_stats: DownloadStats;
}

export function Settings() {
  const [config, setConfig] = useState<Config | null>(null);
  const [dataFolder, setDataFolder] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [downloadType, setDownloadType] = useState("stories");
  const [downloadLimit, setDownloadLimit] = useState(100);

  useEffect(() => {
    fetchConfig();
  }, []);

  const fetchConfig = async () => {
    try {
      const response = await fetch("/api/config");
      const data = await response.json();
      setConfig(data);
      setDataFolder(data.data_folder || "");
    } catch (error) {
      console.error("Failed to fetch config:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleSetDataFolder = async () => {
    if (!dataFolder.trim()) return;
    
    setSaving(true);
    try {
      const response = await fetch("/api/data-folder", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ folder_path: dataFolder }),
      });
      
      if (response.ok) {
        fetchConfig(); // Refresh config
      }
    } catch (error) {
      console.error("Failed to set data folder:", error);
    } finally {
      setSaving(false);
    }
  };

  const handleDownloadAction = async (action: "start" | "stop") => {
    try {
      if (action === "start") {
        const response = await fetch("/api/download/start", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            download_type: downloadType,
            limit: downloadLimit,
          }),
        });
        
        if (response.ok) {
          const result = await response.json();
          console.log(result.message);
          fetchConfig(); // Refresh config
        } else {
          const error = await response.json();
          console.error("Download start failed:", error.error);
        }
      } else {
        const response = await fetch("/api/download/stop", {
          method: "POST",
        });
        
        if (response.ok) {
          fetchConfig(); // Refresh config
        }
      }
    } catch (error) {
      console.error(`Failed to ${action} download:`, error);
    }
  };

  const formatLastDownloadTime = (timeString: string | null) => {
    if (!timeString) return "Never";
    try {
      const date = new Date(timeString);
      return date.toLocaleString();
    } catch {
      return "Invalid date";
    }
  };

  if (loading) {
    return <div className="p-8">Loading...</div>;
  }

  return (
    <div className="p-8 space-y-6">
      <h1 className="text-2xl font-bold">Settings</h1>
      
      <Card>
        <CardHeader>
          <CardTitle>Configuration</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">
                Config File Path
              </label>
              <Input
                value={config?.config_path || ""}
                readOnly
                className="bg-gray-50"
              />
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Data Storage</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">
                Hacker News Data Folder
              </label>
              <div className="flex gap-2">
                <Input
                  value={dataFolder}
                  onChange={(e) => setDataFolder(e.target.value)}
                  placeholder="Select folder for SQLite data storage"
                />
                <Button
                  onClick={handleSetDataFolder}
                  disabled={saving || !dataFolder.trim()}
                >
                  <FolderOpen className="w-4 h-4 mr-2" />
                  {saving ? "Setting..." : "Set Folder"}
                </Button>
              </div>
            </div>
            {config?.data_folder && (
              <div className="text-sm text-gray-600">
                Current: {config.data_folder}
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Download Control</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-2">
                  Download Type
                </label>
                <select
                  value={downloadType}
                  onChange={(e) => setDownloadType(e.target.value)}
                  className="w-full p-2 border border-gray-300 rounded-md"
                  disabled={config?.download_stats.is_downloading}
                >
                  <option value="stories">Top Stories</option>
                  <option value="recent">Recent Items</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium mb-2">
                  Download Limit
                </label>
                <Input
                  type="number"
                  value={downloadLimit}
                  onChange={(e) => setDownloadLimit(parseInt(e.target.value) || 100)}
                  placeholder="Number of items to download"
                  disabled={config?.download_stats.is_downloading}
                  min={1}
                  max={10000}
                />
              </div>
              <div className="flex items-center gap-4">
                <div className="flex gap-2">
                  <Button
                    onClick={() => handleDownloadAction("start")}
                    disabled={config?.download_stats.is_downloading || !config?.data_folder}
                    variant={config?.download_stats.is_downloading ? "secondary" : "default"}
                  >
                    <Play className="w-4 h-4 mr-2" />
                    Start Download
                  </Button>
                  <Button
                    onClick={() => handleDownloadAction("stop")}
                    disabled={!config?.download_stats.is_downloading}
                    variant="destructive"
                  >
                    <Square className="w-4 h-4 mr-2" />
                    Stop
                  </Button>
                </div>
              </div>
              <div className="text-sm">
                Status: {config?.download_stats.is_downloading ? "Running" : "Stopped"}
                {!config?.data_folder && (
                  <span className="text-orange-600 block mt-1">
                    Please set a data folder before downloading
                  </span>
                )}
              </div>
            </div>
            
            {config?.download_stats && (
              <div className="mt-6">
                <div className="flex items-center gap-2 mb-4">
                  <BarChart3 className="w-4 h-4" />
                  <h3 className="font-medium">Download Statistics</h3>
                </div>
                <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 text-sm">
                  <div className="bg-blue-50 p-3 rounded border border-blue-200">
                    <div className="font-medium text-blue-900">
                      {config.download_stats.total_items.toLocaleString()}
                    </div>
                    <div className="text-blue-600">Total Items</div>
                  </div>
                  <div className="bg-green-50 p-3 rounded border border-green-200">
                    <div className="font-medium text-green-900">
                      {config.download_stats.total_users.toLocaleString()}
                    </div>
                    <div className="text-green-600">Total Users</div>
                  </div>
                  <div className="bg-purple-50 p-3 rounded border border-purple-200">
                    <div className="font-medium text-purple-900">
                      {config.download_stats.items_downloaded_today.toLocaleString()}
                    </div>
                    <div className="text-purple-600">Downloaded Today</div>
                  </div>
                  <div className="bg-red-50 p-3 rounded border border-red-200">
                    <div className="font-medium text-red-900">
                      {config.download_stats.download_errors.toLocaleString()}
                    </div>
                    <div className="text-red-600">Download Errors</div>
                  </div>
                  <div className="bg-gray-50 p-3 rounded border border-gray-200 col-span-2">
                    <div className="font-medium text-gray-900">
                      {formatLastDownloadTime(config.download_stats.last_download_time)}
                    </div>
                    <div className="text-gray-600">Last Download</div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}