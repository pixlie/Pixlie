//! Configuration loading utilities for Pixlie TUI application
//!
//! Handles loading and merging configuration from multiple sources with proper precedence.

use super::{CliArgs, ConfigPaths, GlobalConfig, WorkspaceConfig};
use crate::error::{ErrorContext, ErrorContextExt, Result};
use std::path::Path;
use tracing::{debug, info, warn};

/// Configuration loader with support for multiple sources and precedence
pub struct ConfigLoader {
    /// Configuration file paths
    paths: ConfigPaths,
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new() -> Result<Self> {
        let paths = ConfigPaths::new()?;
        Ok(Self { paths })
    }

    /// Load global configuration from file
    pub async fn load_global_config(&self) -> Result<GlobalConfig> {
        let context = ErrorContext::new().with_context("Global configuration loading");

        if !self.paths.global_config.exists() {
            info!(
                "Global config file not found, using defaults: {:?}",
                self.paths.global_config
            );
            return Ok(GlobalConfig::default());
        }

        debug!("Loading global config from: {:?}", self.paths.global_config);

        let content = tokio::fs::read_to_string(&self.paths.global_config)
            .await
            .with_context(|| context.clone())?;

        let config: GlobalConfig = toml::from_str(&content)
            .with_context_msg("Failed to parse global configuration TOML")?;

        config.validate().with_context(|| context.clone())?;

        info!("Global configuration loaded successfully");
        Ok(config)
    }

    /// Load workspace configuration from file
    pub async fn load_workspace_config<P: AsRef<Path>>(
        &self,
        workspace_path: P,
    ) -> Result<Option<WorkspaceConfig>> {
        let context = ErrorContext::new().with_context("Workspace configuration loading");

        let workspace_config_path = workspace_path.as_ref().join(".pixlie-workspace.toml");

        if !workspace_config_path.exists() {
            debug!(
                "Workspace config file not found: {:?}",
                workspace_config_path
            );
            return Ok(None);
        }

        debug!("Loading workspace config from: {:?}", workspace_config_path);

        let content = tokio::fs::read_to_string(&workspace_config_path)
            .await
            .with_context(|| context.clone())?;

        let config: WorkspaceConfig = toml::from_str(&content)
            .with_context_msg("Failed to parse workspace configuration TOML")?;

        config.validate().with_context(|| context.clone())?;

        info!("Workspace configuration loaded successfully");
        Ok(Some(config))
    }

    /// Save global configuration to file
    pub async fn save_global_config(&self, config: &GlobalConfig) -> Result<()> {
        let context = ErrorContext::new().with_context("Global configuration saving");

        // Validate before saving
        config.validate().with_context(|| context.clone())?;

        // Ensure config directory exists
        if let Some(parent) = self.paths.global_config.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| context.clone())?;
        }

        let content = toml::to_string_pretty(config)
            .with_context_msg("Failed to serialize global configuration to TOML")?;

        tokio::fs::write(&self.paths.global_config, content)
            .await
            .with_context(|| context.clone())?;

        info!(
            "Global configuration saved to: {:?}",
            self.paths.global_config
        );
        Ok(())
    }

    /// Save workspace configuration to file
    pub async fn save_workspace_config<P: AsRef<Path>>(
        &self,
        workspace_path: P,
        config: &WorkspaceConfig,
    ) -> Result<()> {
        let context = ErrorContext::new().with_context("Workspace configuration saving");

        let workspace_config_path = workspace_path.as_ref().join(".pixlie-workspace.toml");

        // Validate before saving
        config.validate().with_context(|| context.clone())?;

        let content = toml::to_string_pretty(config)
            .with_context_msg("Failed to serialize workspace configuration to TOML")?;

        tokio::fs::write(&workspace_config_path, content)
            .await
            .with_context(|| context.clone())?;

        info!(
            "Workspace configuration saved to: {:?}",
            workspace_config_path
        );
        Ok(())
    }

    /// Create a default global configuration file
    pub async fn create_default_global_config(&self) -> Result<()> {
        let context = ErrorContext::new().with_context("Default global configuration creation");

        if self.paths.global_config.exists() {
            warn!(
                "Global config file already exists: {:?}",
                self.paths.global_config
            );
            return Ok(());
        }

        let default_config = GlobalConfig::default();
        self.save_global_config(&default_config)
            .await
            .with_context(|| context)?;

        info!("Default global configuration created");
        Ok(())
    }

    /// Create a default workspace configuration file
    pub async fn create_default_workspace_config<P: AsRef<Path>>(
        &self,
        workspace_path: P,
        name: Option<String>,
    ) -> Result<()> {
        let context = ErrorContext::new().with_context("Default workspace configuration creation");

        let workspace_config_path = workspace_path.as_ref().join(".pixlie-workspace.toml");

        if workspace_config_path.exists() {
            warn!(
                "Workspace config file already exists: {:?}",
                workspace_config_path
            );
            return Ok(());
        }

        let mut default_config = WorkspaceConfig::default();

        // Set workspace name if provided
        if let Some(name) = name {
            default_config.metadata.name = Some(name);
        }

        // Set creation timestamp
        default_config.metadata.created_at = Some(chrono::Utc::now());
        default_config.metadata.last_modified = Some(chrono::Utc::now());

        self.save_workspace_config(workspace_path, &default_config)
            .await
            .with_context(|| context)?;

        info!("Default workspace configuration created");
        Ok(())
    }

    /// Load environment variables and apply to global config
    pub fn apply_environment_overrides(&self, config: &mut GlobalConfig) -> Result<()> {
        let context = ErrorContext::new().with_context("Environment variable processing");

        // Load .env file if it exists
        match dotenvy::dotenv() {
            Ok(path) => debug!("Loaded .env file from: {:?}", path),
            Err(_) => debug!("No .env file found or failed to load"),
        }

        // Apply environment variable overrides
        if let Ok(theme) = std::env::var("PIXLIE_THEME") {
            debug!("Applying PIXLIE_THEME override: {}", theme);
            config.ui.theme = theme;
        }

        if let Ok(log_level) = std::env::var("PIXLIE_LOG_LEVEL") {
            debug!("Applying PIXLIE_LOG_LEVEL override: {}", log_level);
            config.ui.log_level = log_level;
        }

        if let Ok(model) = std::env::var("PIXLIE_DEFAULT_MODEL") {
            debug!("Applying PIXLIE_DEFAULT_MODEL override: {}", model);
            config.llm.default_model = model;
        }

        if let Ok(max_iterations) = std::env::var("PIXLIE_MAX_ITERATIONS") {
            match max_iterations.parse::<u32>() {
                Ok(value) => {
                    debug!("Applying PIXLIE_MAX_ITERATIONS override: {}", value);
                    config.llm.max_iterations = value;
                }
                Err(e) => {
                    warn!(
                        "Invalid PIXLIE_MAX_ITERATIONS value '{}': {}",
                        max_iterations, e
                    );
                }
            }
        }

        if let Ok(timeout) = std::env::var("PIXLIE_REQUEST_TIMEOUT") {
            match timeout.parse::<u64>() {
                Ok(value) => {
                    debug!("Applying PIXLIE_REQUEST_TIMEOUT override: {}", value);
                    config.llm.request_timeout = value;
                }
                Err(e) => {
                    warn!("Invalid PIXLIE_REQUEST_TIMEOUT value '{}': {}", timeout, e);
                }
            }
        }

        if let Ok(read_only) = std::env::var("PIXLIE_DATABASE_READ_ONLY") {
            match read_only.to_lowercase().as_str() {
                "true" | "1" | "yes" => {
                    debug!("Applying PIXLIE_DATABASE_READ_ONLY override: true");
                    config.database.read_only = true;
                }
                "false" | "0" | "no" => {
                    debug!("Applying PIXLIE_DATABASE_READ_ONLY override: false");
                    config.database.read_only = false;
                }
                _ => {
                    warn!("Invalid PIXLIE_DATABASE_READ_ONLY value: {}", read_only);
                }
            }
        }

        // Validate after applying environment overrides
        config.validate().with_context(|| context)?;

        Ok(())
    }

    /// Apply CLI argument overrides to configuration
    pub fn apply_cli_overrides(&self, config: &mut GlobalConfig, args: &dyn CliArgs) -> Result<()> {
        let context = ErrorContext::new().with_context("CLI argument processing");

        debug!("Applying CLI argument overrides");

        // Override log level
        config.ui.log_level = args.log_level().to_string();

        // Override JSON logs setting
        config.ui.json_logs = args.json_logs();

        // Override model if different from default
        if args.model() != "gpt-3.5-turbo" {
            config.llm.default_model = args.model().to_string();
        }

        // Override max iterations if different from default
        if args.max_iterations() != 10 {
            config.llm.max_iterations = args.max_iterations();
        }

        // Validate after applying CLI overrides
        config.validate().with_context(|| context)?;

        Ok(())
    }

    /// Get configuration paths
    pub fn paths(&self) -> &ConfigPaths {
        &self.paths
    }

    /// Check if global configuration file exists
    pub fn global_config_exists(&self) -> bool {
        self.paths.global_config.exists()
    }

    /// Check if workspace configuration exists for a given path
    pub fn workspace_config_exists<P: AsRef<Path>>(&self, workspace_path: P) -> bool {
        workspace_path
            .as_ref()
            .join(".pixlie-workspace.toml")
            .exists()
    }

    /// Backup existing configuration file
    pub async fn backup_global_config(&self) -> Result<()> {
        let context = ErrorContext::new().with_context("Global configuration backup");

        if !self.paths.global_config.exists() {
            debug!("No global config file to backup");
            return Ok(());
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = self
            .paths
            .global_config
            .with_extension(format!("toml.backup.{}", timestamp));

        tokio::fs::copy(&self.paths.global_config, &backup_path)
            .await
            .with_context(|| context)?;

        info!("Global configuration backed up to: {:?}", backup_path);
        Ok(())
    }

    /// Backup existing workspace configuration file
    pub async fn backup_workspace_config<P: AsRef<Path>>(&self, workspace_path: P) -> Result<()> {
        let context = ErrorContext::new().with_context("Workspace configuration backup");

        let workspace_config_path = workspace_path.as_ref().join(".pixlie-workspace.toml");

        if !workspace_config_path.exists() {
            debug!("No workspace config file to backup");
            return Ok(());
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path =
            workspace_config_path.with_extension(format!("toml.backup.{}", timestamp));

        tokio::fs::copy(&workspace_config_path, &backup_path)
            .await
            .with_context(|| context)?;

        info!("Workspace configuration backed up to: {:?}", backup_path);
        Ok(())
    }

    /// Merge workspace configuration into global configuration
    /// This is used for getting effective configuration values
    pub fn merge_workspace_into_global(
        &self,
        global: &GlobalConfig,
        workspace: &WorkspaceConfig,
    ) -> GlobalConfig {
        let mut merged = global.clone();

        // Merge UI configuration
        if let Some(ui) = &workspace.ui {
            merged.ui = ui.clone();
        }

        // Merge session configuration
        if let Some(session) = &workspace.session {
            merged.session = session.clone();
        }

        // Merge LLM configuration
        if let Some(llm) = &workspace.llm {
            merged.llm = llm.clone();
        }

        // Merge database configuration
        if let Some(database) = &workspace.database {
            merged.database = database.clone();
        }

        // Merge shortcuts configuration
        if let Some(shortcuts) = &workspace.shortcuts {
            merged.shortcuts = shortcuts.clone();
        }

        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_load_nonexistent_global_config() {
        let loader = ConfigLoader::new().unwrap();
        let config = loader.load_global_config().await.unwrap();

        // Should return default configuration
        assert_eq!(config.ui.theme, "dark");
        assert_eq!(config.llm.default_model, "gpt-3.5-turbo");
    }

    #[tokio::test]
    async fn test_create_and_load_global_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.toml");

        // Create a test configuration
        let test_config = GlobalConfig::default();
        let content = toml::to_string(&test_config).unwrap();
        tokio::fs::write(&config_file, content).await.unwrap();

        // We can't easily test ConfigLoader directly with custom paths
        // but we can test the serialization/deserialization
        let loaded_content = tokio::fs::read_to_string(&config_file).await.unwrap();
        let loaded_config: GlobalConfig = toml::from_str(&loaded_content).unwrap();

        assert_eq!(test_config.ui.theme, loaded_config.ui.theme);
        assert_eq!(
            test_config.llm.default_model,
            loaded_config.llm.default_model
        );
    }

    #[tokio::test]
    async fn test_load_workspace_config() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();

        let loader = ConfigLoader::new().unwrap();

        // Should return None when no config exists
        let config = loader.load_workspace_config(workspace_path).await.unwrap();
        assert!(config.is_none());

        // Create a workspace config
        let test_config = WorkspaceConfig::default();
        loader
            .save_workspace_config(workspace_path, &test_config)
            .await
            .unwrap();

        // Should now load the config
        let config = loader.load_workspace_config(workspace_path).await.unwrap();
        assert!(config.is_some());
    }

    #[test]
    fn test_apply_environment_overrides() {
        let loader = ConfigLoader::new().unwrap();
        let mut config = GlobalConfig::default();

        // Set some environment variables
        std::env::set_var("PIXLIE_THEME", "light");
        std::env::set_var("PIXLIE_LOG_LEVEL", "debug");
        std::env::set_var("PIXLIE_DEFAULT_MODEL", "gpt-4");

        loader.apply_environment_overrides(&mut config).unwrap();

        assert_eq!(config.ui.theme, "light");
        assert_eq!(config.ui.log_level, "debug");
        assert_eq!(config.llm.default_model, "gpt-4");

        // Clean up
        std::env::remove_var("PIXLIE_THEME");
        std::env::remove_var("PIXLIE_LOG_LEVEL");
        std::env::remove_var("PIXLIE_DEFAULT_MODEL");
    }

    #[test]
    fn test_merge_workspace_into_global() {
        let loader = ConfigLoader::new().unwrap();
        let global = GlobalConfig::default();
        let mut workspace = WorkspaceConfig::default();

        // Set some workspace overrides
        let mut ui_override = global.ui.clone();
        ui_override.theme = "light".to_string();
        workspace.ui = Some(ui_override);

        let merged = loader.merge_workspace_into_global(&global, &workspace);

        assert_eq!(merged.ui.theme, "light");
        assert_eq!(merged.llm.default_model, global.llm.default_model); // Should remain unchanged
    }
}
