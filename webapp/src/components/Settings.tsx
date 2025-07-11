import { useState, useEffect } from "react";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { Play, Pause, Square, FolderOpen } from "lucide-react";

interface Config {
  config_path: string;
  data_folder: string | null;
  download_running: boolean;
  download_paused: boolean;
}

export function Settings() {
  const [config, setConfig] = useState<Config | null>(null);
  const [dataFolder, setDataFolder] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

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

  const handleDownloadAction = async (action: "start" | "pause" | "stop") => {
    try {
      const response = await fetch(`/api/download/${action}`, {
        method: "POST",
      });
      
      if (response.ok) {
        fetchConfig(); // Refresh config
      }
    } catch (error) {
      console.error(`Failed to ${action} download:`, error);
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
            <div className="flex items-center gap-4">
              <div className="flex gap-2">
                <Button
                  onClick={() => handleDownloadAction("start")}
                  disabled={config?.download_running && !config?.download_paused}
                  variant={config?.download_running && !config?.download_paused ? "secondary" : "default"}
                >
                  <Play className="w-4 h-4 mr-2" />
                  Start
                </Button>
                <Button
                  onClick={() => handleDownloadAction("pause")}
                  disabled={!config?.download_running || config?.download_paused}
                  variant="outline"
                >
                  <Pause className="w-4 h-4 mr-2" />
                  Pause
                </Button>
                <Button
                  onClick={() => handleDownloadAction("stop")}
                  disabled={!config?.download_running}
                  variant="destructive"
                >
                  <Square className="w-4 h-4 mr-2" />
                  Stop
                </Button>
              </div>
            </div>
            <div className="text-sm">
              Status: {
                config?.download_running
                  ? config?.download_paused
                    ? "Paused"
                    : "Running"
                  : "Stopped"
              }
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}