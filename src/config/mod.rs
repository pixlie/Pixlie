//! Configuration management system for Pixlie TUI application
//!
//! This module provides comprehensive configuration management including:
//! - TUI-specific settings (theme, layout, shortcuts)
//! - Workspace-specific configurations
//! - Database and LLM provider settings
//! - Configuration file, environment variables, and CLI argument handling
//! - Configuration validation and merging with proper precedence

pub mod loader;
pub mod settings;
pub mod workspace;

pub use loader::*;
pub use settings::*;
pub use workspace::*;

use crate::error::{ErrorContext, ErrorContextExt, PixlieError, Result};
use std::path::PathBuf;

/// Trait representing CLI arguments needed by the configuration system
pub trait CliArgs {
    fn workspace(&self) -> Option<&str>;
    fn log_level(&self) -> &str;
    fn json_logs(&self) -> bool;
    fn model(&self) -> &str;
    fn max_iterations(&self) -> u32;
}

/// Main configuration manager for the application
#[derive(Debug, Clone)]
pub struct ConfigManager {
    /// Global application settings
    pub global: GlobalConfig,
    /// Current workspace configuration
    pub workspace: Option<WorkspaceConfig>,
    /// Configuration file paths
    pub paths: ConfigPaths,
}

/// Configuration file paths
#[derive(Debug, Clone)]
pub struct ConfigPaths {
    /// Global configuration file (~/.pixlie/config.toml)
    pub global_config: PathBuf,
    /// Global configuration directory (~/.pixlie/)
    pub config_dir: PathBuf,
    /// Workspace-specific config file (workspace/.pixlie-workspace.toml)
    pub workspace_config: Option<PathBuf>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self> {
        let paths = ConfigPaths::new()?;
        let global = GlobalConfig::default();

        Ok(Self {
            global,
            workspace: None,
            paths,
        })
    }

    /// Load configuration from all sources with proper precedence
    pub async fn load(&mut self, cli_args: &dyn CliArgs) -> Result<()> {
        let context = ErrorContext::new().with_context("Configuration loading");

        // Load environment variables first
        self.load_environment_variables()?;

        // Load global configuration file
        self.load_global_config_file().await?;

        // Load workspace configuration if specified
        if let Some(workspace_path) = cli_args.workspace() {
            self.load_workspace_config(workspace_path).await?;
        }

        // Apply CLI argument overrides
        self.apply_cli_overrides(cli_args)?;

        // Validate the final configuration
        self.validate().with_context(|| context.clone())?;

        Ok(())
    }

    /// Validate the current configuration
    pub fn validate(&self) -> Result<()> {
        let context = ErrorContext::new().with_context("Configuration validation");

        // Validate global configuration
        self.global.validate().with_context(|| context.clone())?;

        // Validate workspace configuration if present
        if let Some(workspace) = &self.workspace {
            workspace.validate().with_context(|| context.clone())?;
        }

        Ok(())
    }

    /// Save current configuration to files
    pub async fn save(&self) -> Result<()> {
        let context = ErrorContext::new().with_context("Configuration saving");

        // Save global configuration
        self.save_global_config()
            .await
            .with_context(|| context.clone())?;

        // Save workspace configuration if present
        if let Some(workspace) = &self.workspace {
            self.save_workspace_config(workspace)
                .await
                .with_context(|| context.clone())?;
        }

        Ok(())
    }

    /// Get the effective UI configuration for the current context
    pub fn effective_ui_config(&self) -> &UiConfig {
        // Workspace UI settings override global ones
        if let Some(workspace) = &self.workspace {
            if let Some(ui) = &workspace.ui {
                return ui;
            }
        }
        &self.global.ui
    }

    /// Get the effective session configuration for the current context
    pub fn effective_session_config(&self) -> &SessionConfig {
        if let Some(workspace) = &self.workspace {
            if let Some(session) = &workspace.session {
                return session;
            }
        }
        &self.global.session
    }

    /// Get the effective LLM configuration for the current context
    pub fn effective_llm_config(&self) -> &LlmConfig {
        if let Some(workspace) = &self.workspace {
            if let Some(llm) = &workspace.llm {
                return llm;
            }
        }
        &self.global.llm
    }

    /// Get the effective database configuration for the current context
    pub fn effective_database_config(&self) -> &DatabaseConfig {
        if let Some(workspace) = &self.workspace {
            if let Some(database) = &workspace.database {
                return database;
            }
        }
        &self.global.database
    }

    /// Get the effective shortcuts configuration for the current context
    pub fn effective_shortcuts_config(&self) -> &ShortcutsConfig {
        if let Some(workspace) = &self.workspace {
            if let Some(shortcuts) = &workspace.shortcuts {
                return shortcuts;
            }
        }
        &self.global.shortcuts
    }

    /// Load environment variables
    fn load_environment_variables(&mut self) -> Result<()> {
        // Load .env file if it exists
        let _ = dotenvy::dotenv(); // Ignore errors if .env file doesn't exist

        // Apply environment variable overrides
        if let Ok(theme) = std::env::var("PIXLIE_THEME") {
            self.global.ui.theme = theme;
        }

        if let Ok(log_level) = std::env::var("PIXLIE_LOG_LEVEL") {
            self.global.ui.log_level = log_level;
        }

        if let Ok(model) = std::env::var("PIXLIE_DEFAULT_MODEL") {
            self.global.llm.default_model = model;
        }

        Ok(())
    }

    /// Load global configuration file
    async fn load_global_config_file(&mut self) -> Result<()> {
        if self.paths.global_config.exists() {
            let content = tokio::fs::read_to_string(&self.paths.global_config).await?;
            let config: GlobalConfig = toml::from_str(&content)?;
            self.global = config;
        }
        Ok(())
    }

    /// Load workspace-specific configuration
    async fn load_workspace_config(&mut self, workspace_path: &str) -> Result<()> {
        let workspace_config_path = PathBuf::from(workspace_path).join(".pixlie-workspace.toml");

        if workspace_config_path.exists() {
            let content = tokio::fs::read_to_string(&workspace_config_path).await?;
            let config: WorkspaceConfig = toml::from_str(&content)?;
            self.workspace = Some(config);
            self.paths.workspace_config = Some(workspace_config_path);
        }

        Ok(())
    }

    /// Apply CLI argument overrides
    fn apply_cli_overrides(&mut self, args: &dyn CliArgs) -> Result<()> {
        // Override log level if specified
        self.global.ui.log_level = args.log_level().to_string();

        // Override JSON logs setting
        self.global.ui.json_logs = args.json_logs();

        // Override model if specified and different from default
        if args.model() != "gpt-3.5-turbo" {
            self.global.llm.default_model = args.model().to_string();
        }

        // Override max iterations if specified and different from default
        if args.max_iterations() != 10 {
            self.global.llm.max_iterations = args.max_iterations();
        }

        Ok(())
    }

    /// Save global configuration to file
    async fn save_global_config(&self) -> Result<()> {
        // Ensure config directory exists
        if let Some(parent) = self.paths.global_config.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = toml::to_string_pretty(&self.global)?;
        tokio::fs::write(&self.paths.global_config, content).await?;

        Ok(())
    }

    /// Save workspace configuration to file
    async fn save_workspace_config(&self, workspace: &WorkspaceConfig) -> Result<()> {
        if let Some(workspace_config_path) = &self.paths.workspace_config {
            let content = toml::to_string_pretty(workspace)?;
            tokio::fs::write(workspace_config_path, content).await?;
        }

        Ok(())
    }
}

impl ConfigPaths {
    /// Create new configuration paths
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| {
                PixlieError::configuration(
                    "Could not determine user configuration directory",
                    ErrorContext::new().with_context("Configuration directory resolution"),
                )
            })?
            .join("pixlie");

        let global_config = config_dir.join("config.toml");

        Ok(Self {
            global_config,
            config_dir,
            workspace_config: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_config_manager_creation() {
        let config_manager = ConfigManager::new();
        assert!(config_manager.is_ok());

        let manager = config_manager.unwrap();
        assert!(manager.workspace.is_none());
        assert_eq!(manager.global.ui.theme, "dark");
    }

    #[test]
    fn test_config_paths_creation() {
        let paths = ConfigPaths::new();
        assert!(paths.is_ok());

        let paths = paths.unwrap();
        assert!(paths.global_config.to_string_lossy().contains("pixlie"));
        assert!(paths.config_dir.to_string_lossy().contains("pixlie"));
    }
}
