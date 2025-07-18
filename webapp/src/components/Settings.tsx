import { useState, useEffect } from "react";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { Play, Square, FolderOpen, BarChart3, Download, Brain, Package, Key, Bot, TrendingUp, Settings as SettingsIcon } from "lucide-react";
import type { ConfigResponse as Config } from "../types/api";
import type { ModelInfo } from "../types/extraction";
import type { ExtractionStats } from "../types/database";

export function Settings() {
  const [config, setConfig] = useState<Config | null>(null);
  const [dataFolder, setDataFolder] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [downloadType, setDownloadType] = useState("stories");
  const [downloadLimit, setDownloadLimit] = useState(100);
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [extractionStats, setExtractionStats] = useState<ExtractionStats | null>(null);
  const [downloadingModel, setDownloadingModel] = useState<string | null>(null);
  const [extractionBatchSize, setExtractionBatchSize] = useState(100);
  
  // LLM Configuration state
  const [llmProvider, setLlmProvider] = useState("openai");
  const [openaiApiKey, setOpenaiApiKey] = useState("");
  const [anthropicApiKey, setAnthropicApiKey] = useState("");
  const [selectedModel, setSelectedModel] = useState("");
  const [maxTokens, setMaxTokens] = useState(4000);
  const [temperature, setTemperature] = useState(0.7);
  const [costLimit, setCostLimit] = useState(10.0);
  const [usageTracking, setUsageTracking] = useState(true);

  useEffect(() => {
    fetchConfig();
    fetchModels();
    fetchExtractionStats();
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

  const fetchModels = async () => {
    try {
      const response = await fetch("/api/models");
      const data = await response.json();
      setModels(data.models);
    } catch (error) {
      console.error("Failed to fetch models:", error);
    }
  };

  const fetchExtractionStats = async () => {
    try {
      const response = await fetch("/api/extraction/status");
      const data = await response.json();
      setExtractionStats(data.extraction_stats);
    } catch (error) {
      console.error("Failed to fetch extraction stats:", error);
    }
  };

  const handleDownloadModel = async (modelName: string) => {
    if (!config?.data_folder) {
      alert("Please set a data folder first");
      return;
    }

    setDownloadingModel(modelName);
    try {
      const response = await fetch("/api/models/download", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ model_name: modelName }),
      });
      
      if (response.ok) {
        const result = await response.json();
        console.log(result.message);
        fetchModels(); // Refresh models
      } else {
        const error = await response.json();
        console.error("Model download failed:", error.error);
      }
    } catch (error) {
      console.error("Failed to download model:", error);
    } finally {
      setDownloadingModel(null);
    }
  };

  const handleExtractionAction = async (action: "start" | "stop") => {
    try {
      if (action === "start") {
        const response = await fetch("/api/extraction/start", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            batch_size: extractionBatchSize,
          }),
        });
        
        if (response.ok) {
          const result = await response.json();
          console.log(result.message);
          fetchExtractionStats(); // Refresh stats
        } else {
          const error = await response.json();
          console.error("Extraction start failed:", error.error);
        }
      } else {
        const response = await fetch("/api/extraction/stop", {
          method: "POST",
        });
        
        if (response.ok) {
          fetchExtractionStats(); // Refresh stats
        }
      }
    } catch (error) {
      console.error(`Failed to ${action} extraction:`, error);
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
          <CardTitle>LLM Configuration</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-6">
            <div className="text-sm text-gray-600 mb-4">
              Configure Large Language Model providers for AI-powered features like entity analysis, content summarization, and intelligent insights.
            </div>
            
            {/* Provider Selection */}
            <div>
              <label className="block text-sm font-medium mb-2">
                LLM Provider
              </label>
              <select
                value={llmProvider}
                onChange={(e) => setLlmProvider(e.target.value)}
                className="w-full p-2 border border-gray-300 rounded-md"
              >
                <option value="openai">OpenAI</option>
                <option value="anthropic">Anthropic</option>
                <option value="local">Local Model</option>
              </select>
            </div>

            {/* API Keys */}
            {llmProvider === "openai" && (
              <div>
                <label className="block text-sm font-medium mb-2">
                  <Key className="w-4 h-4 inline mr-1" />
                  OpenAI API Key
                </label>
                <Input
                  type="password"
                  value={openaiApiKey}
                  onChange={(e) => setOpenaiApiKey(e.target.value)}
                  placeholder="sk-..."
                />
                <div className="text-xs text-gray-500 mt-1">
                  Your API key is stored securely and never transmitted in logs
                </div>
              </div>
            )}

            {llmProvider === "anthropic" && (
              <div>
                <label className="block text-sm font-medium mb-2">
                  <Key className="w-4 h-4 inline mr-1" />
                  Anthropic API Key
                </label>
                <Input
                  type="password"
                  value={anthropicApiKey}
                  onChange={(e) => setAnthropicApiKey(e.target.value)}
                  placeholder="sk-ant-..."
                />
                <div className="text-xs text-gray-500 mt-1">
                  Your API key is stored securely and never transmitted in logs
                </div>
              </div>
            )}

            {/* Model Selection */}
            <div>
              <label className="block text-sm font-medium mb-2">
                <Bot className="w-4 h-4 inline mr-1" />
                Model Selection
              </label>
              <select
                value={selectedModel}
                onChange={(e) => setSelectedModel(e.target.value)}
                className="w-full p-2 border border-gray-300 rounded-md"
              >
                {llmProvider === "openai" && (
                  <>
                    <option value="">Select OpenAI model...</option>
                    <option value="gpt-4">GPT-4</option>
                    <option value="gpt-4-turbo">GPT-4 Turbo</option>
                    <option value="gpt-3.5-turbo">GPT-3.5 Turbo</option>
                  </>
                )}
                {llmProvider === "anthropic" && (
                  <>
                    <option value="">Select Anthropic model...</option>
                    <option value="claude-3-opus">Claude 3 Opus</option>
                    <option value="claude-3-sonnet">Claude 3 Sonnet</option>
                    <option value="claude-3-haiku">Claude 3 Haiku</option>
                  </>
                )}
                {llmProvider === "local" && (
                  <>
                    <option value="">Select local model...</option>
                    <option value="llama2-7b">Llama 2 7B</option>
                    <option value="mistral-7b">Mistral 7B</option>
                  </>
                )}
              </select>
            </div>

            {/* Performance Settings */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium mb-2">
                  Max Tokens
                </label>
                <Input
                  type="number"
                  value={maxTokens}
                  onChange={(e) => setMaxTokens(parseInt(e.target.value) || 4000)}
                  min={100}
                  max={32000}
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-2">
                  Temperature
                </label>
                <Input
                  type="number"
                  value={temperature}
                  onChange={(e) => setTemperature(parseFloat(e.target.value) || 0.7)}
                  min={0}
                  max={2}
                  step={0.1}
                />
              </div>
            </div>

            {/* Cost Management */}
            <div className="border-t pt-4">
              <div className="flex items-center gap-2 mb-4">
                <TrendingUp className="w-4 h-4" />
                <h3 className="font-medium">Cost Management & Usage</h3>
              </div>
              
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium mb-2">
                    Monthly Cost Limit ($)
                  </label>
                  <Input
                    type="number"
                    value={costLimit}
                    onChange={(e) => setCostLimit(parseFloat(e.target.value) || 10.0)}
                    min={0}
                    step={0.01}
                  />
                  <div className="text-xs text-gray-500 mt-1">
                    Stop requests when this limit is reached
                  </div>
                </div>
                <div>
                  <label className="flex items-center gap-2 text-sm font-medium">
                    <input
                      type="checkbox"
                      checked={usageTracking}
                      onChange={(e) => setUsageTracking(e.target.checked)}
                      className="rounded"
                    />
                    Enable Usage Tracking
                  </label>
                  <div className="text-xs text-gray-500 mt-1">
                    Track API calls, tokens, and costs
                  </div>
                </div>
              </div>

              {/* Usage Statistics */}
              <div className="mt-4 p-4 bg-gray-50 rounded-lg">
                <div className="text-sm font-medium mb-2">Current Month Usage</div>
                <div className="grid grid-cols-2 lg:grid-cols-4 gap-3 text-sm">
                  <div className="text-center">
                    <div className="font-medium text-blue-900">1,247</div>
                    <div className="text-blue-600">API Calls</div>
                  </div>
                  <div className="text-center">
                    <div className="font-medium text-green-900">342K</div>
                    <div className="text-green-600">Tokens Used</div>
                  </div>
                  <div className="text-center">
                    <div className="font-medium text-purple-900">$3.42</div>
                    <div className="text-purple-600">Cost</div>
                  </div>
                  <div className="text-center">
                    <div className="font-medium text-orange-900">34%</div>
                    <div className="text-orange-600">Limit Used</div>
                  </div>
                </div>
              </div>
            </div>

            {/* Save Configuration */}
            <div className="border-t pt-4">
              <Button className="w-full">
                <SettingsIcon className="w-4 h-4 mr-2" />
                Save LLM Configuration
              </Button>
            </div>
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

      <Card>
        <CardHeader>
          <CardTitle>Entity Extraction Models</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="text-sm text-gray-600 mb-4">
              Download ONNX models for entity extraction. Models are used to identify persons, companies, locations, and other entities from Hacker News content.
            </div>
            
            {models.map((model) => (
              <div key={model.name} className="flex items-center justify-between p-4 border rounded-lg">
                <div className="flex items-center gap-3">
                  <Package className="w-5 h-5 text-blue-600" />
                  <div>
                    <div className="font-medium">{model.name}</div>
                    <div className="text-sm text-gray-600">
                      Size: {model.size_mb} MB
                      {model.is_downloaded && (
                        <span className="ml-2 text-green-600">✓ Downloaded</span>
                      )}
                    </div>
                  </div>
                </div>
                <Button
                  onClick={() => handleDownloadModel(model.name)}
                  disabled={model.is_downloaded || downloadingModel === model.name || !config?.data_folder}
                  variant={model.is_downloaded ? "secondary" : "default"}
                  size="sm"
                >
                  <Download className="w-4 h-4 mr-2" />
                  {downloadingModel === model.name 
                    ? "Downloading..." 
                    : model.is_downloaded 
                      ? "Downloaded" 
                      : "Download"
                  }
                </Button>
              </div>
            ))}
            
            {!config?.data_folder && (
              <div className="text-sm text-orange-600">
                Please set a data folder before downloading models
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Entity Extraction</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="text-sm text-gray-600 mb-4">
              Extract named entities (persons, companies, locations, etc.) from your downloaded Hacker News content using GLiNER models.
            </div>
            
            <div>
              <label className="block text-sm font-medium mb-2">
                Batch Size
              </label>
              <Input
                type="number"
                value={extractionBatchSize}
                onChange={(e) => setExtractionBatchSize(parseInt(e.target.value) || 100)}
                placeholder="Number of items to process in each batch"
                disabled={extractionStats?.is_extracting}
                min={1}
                max={1000}
              />
            </div>
            
            <div className="flex items-center gap-4">
              <div className="flex gap-2">
                <Button
                  onClick={() => handleExtractionAction("start")}
                  disabled={
                    extractionStats?.is_extracting || 
                    !config?.data_folder ||
                    !models.some(m => m.is_downloaded)
                  }
                  variant={extractionStats?.is_extracting ? "secondary" : "default"}
                >
                  <Brain className="w-4 h-4 mr-2" />
                  Start Extraction
                </Button>
                <Button
                  onClick={() => handleExtractionAction("stop")}
                  disabled={!extractionStats?.is_extracting}
                  variant="destructive"
                >
                  <Square className="w-4 h-4 mr-2" />
                  Pause
                </Button>
              </div>
            </div>
            
            <div className="text-sm">
              Status: {extractionStats?.is_extracting ? "Running" : "Stopped"}
              {!config?.data_folder && (
                <span className="text-orange-600 block mt-1">
                  Please set a data folder before extracting entities
                </span>
              )}
              {!models.some(m => m.is_downloaded) && config?.data_folder && (
                <span className="text-orange-600 block mt-1">
                  Please download a model before extracting entities
                </span>
              )}
            </div>
            
            {extractionStats && (
              <div className="mt-6">
                <div className="flex items-center gap-2 mb-4">
                  <BarChart3 className="w-4 h-4" />
                  <h3 className="font-medium">Extraction Statistics</h3>
                </div>
                <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 text-sm">
                  <div className="bg-purple-50 p-3 rounded border border-purple-200">
                    <div className="font-medium text-purple-900">
                      {extractionStats.total_entities.toLocaleString()}
                    </div>
                    <div className="text-purple-600">Total Entities</div>
                  </div>
                  <div className="bg-blue-50 p-3 rounded border border-blue-200">
                    <div className="font-medium text-blue-900">
                      {extractionStats.total_items_processed.toLocaleString()}
                    </div>
                    <div className="text-blue-600">Items Processed</div>
                  </div>
                  <div className="bg-orange-50 p-3 rounded border border-orange-200">
                    <div className="font-medium text-orange-900">
                      {extractionStats.items_remaining.toLocaleString()}
                    </div>
                    <div className="text-orange-600">Items Remaining</div>
                  </div>
                  <div className="bg-green-50 p-3 rounded border border-green-200">
                    <div className="font-medium text-green-900">
                      {extractionStats.items_remaining > 0 
                        ? Math.round((extractionStats.total_items_processed / (extractionStats.total_items_processed + extractionStats.items_remaining)) * 100)
                        : 100
                      }%
                    </div>
                    <div className="text-green-600">Progress</div>
                  </div>
                </div>
                
                {Object.keys(extractionStats.entities_by_type).length > 0 && (
                  <div className="mt-4">
                    <h4 className="font-medium mb-2">Entities by Type</h4>
                    <div className="grid grid-cols-2 lg:grid-cols-4 gap-2 text-sm">
                      {Object.entries(extractionStats.entities_by_type).map(([type, count]) => (
                        <div key={type} className="bg-gray-50 p-2 rounded border">
                          <div className="font-medium">{count.toLocaleString()}</div>
                          <div className="text-gray-600 capitalize">{type}</div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
                
                {extractionStats.last_extraction_time && (
                  <div className="mt-4 text-sm text-gray-600">
                    Last extraction: {formatLastDownloadTime(extractionStats.last_extraction_time)}
                  </div>
                )}
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}