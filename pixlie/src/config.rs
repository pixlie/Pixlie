use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LLMConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub data_folder: Option<PathBuf>,
    #[serde(default)]
    pub download_running: bool,
    #[serde(default)]
    pub download_paused: bool,
    #[serde(default)]
    pub llm: LLMConfig,
}

impl Config {
    pub fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir().ok_or("Could not find config directory")?;

        let pixlie_config_dir = config_dir.join("Pixlie");

        // Create the config directory if it doesn't exist
        if !pixlie_config_dir.exists() {
            fs::create_dir_all(&pixlie_config_dir)?;
        }

        Ok(pixlie_config_dir.join("config.json"))
    }

    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            // Create default config if it doesn't exist
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let config_content = fs::read_to_string(&config_path)?;
        let config: Config = serde_json::from_str(&config_content)?;

        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;
        let config_json = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, config_json)?;

        Ok(())
    }

    pub fn set_data_folder(&mut self, folder: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Validate that the folder exists or can be created
        if !folder.exists() {
            fs::create_dir_all(&folder)?;
        }

        self.data_folder = Some(folder);
        self.save()?;

        Ok(())
    }
}
