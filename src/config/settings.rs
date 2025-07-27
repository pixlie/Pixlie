//! Configuration settings for Pixlie TUI application
//! 
//! Defines all configuration structures for different components of the application.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::{PixlieError, ErrorContext, Result, ErrorContextExt};

/// Global application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// UI configuration
    #[serde(default)]
    pub ui: UiConfig,
    
    /// Session management configuration
    #[serde(default)]
    pub session: SessionConfig,
    
    /// LLM provider configuration
    #[serde(default)]
    pub llm: LlmConfig,
    
    /// Database configuration
    #[serde(default)]
    pub database: DatabaseConfig,
    
    /// Keyboard shortcuts configuration
    #[serde(default)]
    pub shortcuts: ShortcutsConfig,
}

/// TUI interface configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Theme name (dark, light, auto)
    #[serde(default = "default_theme")]
    pub theme: String,
    
    /// Layout style (compact, comfortable, spacious)
    #[serde(default = "default_layout")]
    pub layout: String,
    
    /// Enable colored output
    #[serde(default = "default_colored")]
    pub colored: bool,
    
    /// JSON logging format
    #[serde(default)]
    pub json_logs: bool,
    
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
    /// Maximum chat history to display in TUI
    #[serde(default = "default_max_chat_history")]
    pub max_chat_history: usize,
    
    /// Auto-save interval in seconds
    #[serde(default = "default_autosave_interval")]
    pub autosave_interval: u64,
    
    /// Enable line numbers in code blocks
    #[serde(default = "default_show_line_numbers")]
    pub show_line_numbers: bool,
    
    /// Word wrap in chat messages
    #[serde(default = "default_word_wrap")]
    pub word_wrap: bool,
    
    /// Animation duration in milliseconds
    #[serde(default = "default_animation_duration")]
    pub animation_duration: u64,
}

/// Session and workspace management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Default workspace directory
    #[serde(default)]
    pub default_workspace: Option<String>,
    
    /// Maximum number of concurrent objectives
    #[serde(default = "default_max_objectives")]
    pub max_objectives: usize,
    
    /// Chat history retention in days
    #[serde(default = "default_history_retention_days")]
    pub history_retention_days: u32,
    
    /// Maximum history file size in MB
    #[serde(default = "default_max_history_size_mb")]
    pub max_history_size_mb: u64,
    
    /// Auto-save session state
    #[serde(default = "default_auto_save")]
    pub auto_save: bool,
    
    /// Session backup frequency in minutes
    #[serde(default = "default_backup_frequency")]
    pub backup_frequency: u32,
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Default model to use
    #[serde(default = "default_model")]
    pub default_model: String,
    
    /// Maximum iterations for analysis
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    
    /// Request timeout in seconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,
    
    /// Maximum tokens per request
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    
    /// Temperature for response generation
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    
    /// Provider-specific configurations
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    
    /// Enable streaming responses
    #[serde(default = "default_enable_streaming")]
    pub enable_streaming: bool,
    
    /// Retry attempts for failed requests
    #[serde(default = "default_retry_attempts")]
    pub retry_attempts: u32,
}

/// LLM provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API endpoint URL
    pub endpoint: Option<String>,
    
    /// API key (should be set via environment variable)
    pub api_key_env: Option<String>,
    
    /// Custom headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    
    /// Rate limit (requests per minute)
    pub rate_limit: Option<u32>,
    
    /// Default model for this provider
    pub default_model: Option<String>,
}

/// Database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,
    
    /// Query timeout in seconds
    #[serde(default = "default_query_timeout")]
    pub query_timeout: u64,
    
    /// Maximum number of concurrent connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    
    /// Enable read-only mode by default
    #[serde(default = "default_read_only")]
    pub read_only: bool,
    
    /// Query result limit
    #[serde(default = "default_query_result_limit")]
    pub query_result_limit: usize,
    
    /// Enable query caching
    #[serde(default = "default_enable_caching")]
    pub enable_caching: bool,
    
    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,
}

/// Keyboard shortcuts configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutsConfig {
    /// Quit application
    #[serde(default = "default_quit_key")]
    pub quit: String,
    
    /// Create new objective
    #[serde(default = "default_new_objective_key")]
    pub new_objective: String,
    
    /// Delete current objective
    #[serde(default = "default_delete_objective_key")]
    pub delete_objective: String,
    
    /// Toggle chat history
    #[serde(default = "default_toggle_history_key")]
    pub toggle_history: String,
    
    /// Save session
    #[serde(default = "default_save_session_key")]
    pub save_session: String,
    
    /// Load session
    #[serde(default = "default_load_session_key")]
    pub load_session: String,
    
    /// Switch to next objective
    #[serde(default = "default_next_objective_key")]
    pub next_objective: String,
    
    /// Switch to previous objective
    #[serde(default = "default_prev_objective_key")]
    pub prev_objective: String,
    
    /// Open settings
    #[serde(default = "default_settings_key")]
    pub settings: String,
    
    /// Send message/execute command
    #[serde(default = "default_send_key")]
    pub send: String,
    
    /// Clear input
    #[serde(default = "default_clear_input_key")]
    pub clear_input: String,
    
    /// Navigate up
    #[serde(default = "default_nav_up_key")]
    pub nav_up: String,
    
    /// Navigate down
    #[serde(default = "default_nav_down_key")]
    pub nav_down: String,
    
    /// Navigate left
    #[serde(default = "default_nav_left_key")]
    pub nav_left: String,
    
    /// Navigate right
    #[serde(default = "default_nav_right_key")]
    pub nav_right: String,
}

// Default value functions for serde defaults

fn default_theme() -> String { "dark".to_string() }
fn default_layout() -> String { "comfortable".to_string() }
fn default_colored() -> bool { true }
fn default_log_level() -> String { "info".to_string() }
fn default_max_chat_history() -> usize { 1000 }
fn default_autosave_interval() -> u64 { 30 }
fn default_show_line_numbers() -> bool { true }
fn default_word_wrap() -> bool { true }
fn default_animation_duration() -> u64 { 200 }

fn default_max_objectives() -> usize { 10 }
fn default_history_retention_days() -> u32 { 30 }
fn default_max_history_size_mb() -> u64 { 100 }
fn default_auto_save() -> bool { true }
fn default_backup_frequency() -> u32 { 15 }

fn default_model() -> String { "gpt-3.5-turbo".to_string() }
fn default_max_iterations() -> u32 { 10 }
fn default_request_timeout() -> u64 { 30 }
fn default_max_tokens() -> u32 { 4096 }
fn default_temperature() -> f32 { 0.7 }
fn default_enable_streaming() -> bool { true }
fn default_retry_attempts() -> u32 { 3 }

fn default_connection_timeout() -> u64 { 30 }
fn default_query_timeout() -> u64 { 60 }
fn default_max_connections() -> u32 { 10 }
fn default_read_only() -> bool { true }
fn default_query_result_limit() -> usize { 1000 }
fn default_enable_caching() -> bool { true }
fn default_cache_ttl() -> u64 { 300 }

fn default_quit_key() -> String { "Ctrl+Q".to_string() }
fn default_new_objective_key() -> String { "Ctrl+N".to_string() }
fn default_delete_objective_key() -> String { "Ctrl+D".to_string() }
fn default_toggle_history_key() -> String { "Ctrl+H".to_string() }
fn default_save_session_key() -> String { "Ctrl+S".to_string() }
fn default_load_session_key() -> String { "Ctrl+L".to_string() }
fn default_next_objective_key() -> String { "Tab".to_string() }
fn default_prev_objective_key() -> String { "Shift+Tab".to_string() }
fn default_settings_key() -> String { "Ctrl+,".to_string() }
fn default_send_key() -> String { "Enter".to_string() }
fn default_clear_input_key() -> String { "Ctrl+U".to_string() }
fn default_nav_up_key() -> String { "Up".to_string() }
fn default_nav_down_key() -> String { "Down".to_string() }
fn default_nav_left_key() -> String { "Left".to_string() }
fn default_nav_right_key() -> String { "Right".to_string() }

// Default implementations

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            ui: UiConfig::default(),
            session: SessionConfig::default(),
            llm: LlmConfig::default(),
            database: DatabaseConfig::default(),
            shortcuts: ShortcutsConfig::default(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            layout: default_layout(),
            colored: default_colored(),
            json_logs: false,
            log_level: default_log_level(),
            max_chat_history: default_max_chat_history(),
            autosave_interval: default_autosave_interval(),
            show_line_numbers: default_show_line_numbers(),
            word_wrap: default_word_wrap(),
            animation_duration: default_animation_duration(),
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            default_workspace: None,
            max_objectives: default_max_objectives(),
            history_retention_days: default_history_retention_days(),
            max_history_size_mb: default_max_history_size_mb(),
            auto_save: default_auto_save(),
            backup_frequency: default_backup_frequency(),
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            default_model: default_model(),
            max_iterations: default_max_iterations(),
            request_timeout: default_request_timeout(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            providers: HashMap::new(),
            enable_streaming: default_enable_streaming(),
            retry_attempts: default_retry_attempts(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            connection_timeout: default_connection_timeout(),
            query_timeout: default_query_timeout(),
            max_connections: default_max_connections(),
            read_only: default_read_only(),
            query_result_limit: default_query_result_limit(),
            enable_caching: default_enable_caching(),
            cache_ttl: default_cache_ttl(),
        }
    }
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        Self {
            quit: default_quit_key(),
            new_objective: default_new_objective_key(),
            delete_objective: default_delete_objective_key(),
            toggle_history: default_toggle_history_key(),
            save_session: default_save_session_key(),
            load_session: default_load_session_key(),
            next_objective: default_next_objective_key(),
            prev_objective: default_prev_objective_key(),
            settings: default_settings_key(),
            send: default_send_key(),
            clear_input: default_clear_input_key(),
            nav_up: default_nav_up_key(),
            nav_down: default_nav_down_key(),
            nav_left: default_nav_left_key(),
            nav_right: default_nav_right_key(),
        }
    }
}

// Validation implementations

impl GlobalConfig {
    /// Validate the global configuration
    pub fn validate(&self) -> Result<()> {
        let context = ErrorContext::new().with_context("Global configuration validation");
        
        self.ui.validate().with_context(|| context.clone())?;
        self.session.validate().with_context(|| context.clone())?;
        self.llm.validate().with_context(|| context.clone())?;
        self.database.validate().with_context(|| context.clone())?;
        self.shortcuts.validate().with_context(|| context.clone())?;
        
        Ok(())
    }
}

impl UiConfig {
    /// Validate UI configuration
    pub fn validate(&self) -> Result<()> {
        let context = ErrorContext::new().with_context("UI configuration validation");
        
        // Validate theme
        if !["dark", "light", "auto"].contains(&self.theme.as_str()) {
            return Err(PixlieError::validation(
                "ui.theme",
                "Theme must be 'dark', 'light', or 'auto'",
                context,
            ));
        }
        
        // Validate layout
        if !["compact", "comfortable", "spacious"].contains(&self.layout.as_str()) {
            return Err(PixlieError::validation(
                "ui.layout",
                "Layout must be 'compact', 'comfortable', or 'spacious'",
                context,
            ));
        }
        
        // Validate log level
        if !["trace", "debug", "info", "warn", "error"].contains(&self.log_level.as_str()) {
            return Err(PixlieError::validation(
                "ui.log_level",
                "Log level must be 'trace', 'debug', 'info', 'warn', or 'error'",
                context,
            ));
        }
        
        // Validate reasonable values
        if self.max_chat_history > 10000 {
            return Err(PixlieError::validation(
                "ui.max_chat_history",
                "Maximum chat history cannot exceed 10,000 messages",
                context,
            ));
        }
        
        if self.autosave_interval > 3600 {
            return Err(PixlieError::validation(
                "ui.autosave_interval",
                "Autosave interval cannot exceed 1 hour (3600 seconds)",
                context,
            ));
        }
        
        Ok(())
    }
}

impl SessionConfig {
    /// Validate session configuration
    pub fn validate(&self) -> Result<()> {
        let context = ErrorContext::new().with_context("Session configuration validation");
        
        if self.max_objectives > 50 {
            return Err(PixlieError::validation(
                "session.max_objectives",
                "Maximum objectives cannot exceed 50",
                context,
            ));
        }
        
        if self.history_retention_days > 365 {
            return Err(PixlieError::validation(
                "session.history_retention_days",
                "History retention cannot exceed 365 days",
                context,
            ));
        }
        
        if self.max_history_size_mb > 1000 {
            return Err(PixlieError::validation(
                "session.max_history_size_mb",
                "Maximum history size cannot exceed 1000 MB",
                context,
            ));
        }
        
        Ok(())
    }
}

impl LlmConfig {
    /// Validate LLM configuration
    pub fn validate(&self) -> Result<()> {
        let context = ErrorContext::new().with_context("LLM configuration validation");
        
        if self.max_iterations > 100 {
            return Err(PixlieError::validation(
                "llm.max_iterations",
                "Maximum iterations cannot exceed 100",
                context,
            ));
        }
        
        if self.request_timeout > 300 {
            return Err(PixlieError::validation(
                "llm.request_timeout",
                "Request timeout cannot exceed 300 seconds",
                context,
            ));
        }
        
        if self.temperature < 0.0 || self.temperature > 2.0 {
            return Err(PixlieError::validation(
                "llm.temperature",
                "Temperature must be between 0.0 and 2.0",
                context,
            ));
        }
        
        if self.max_tokens > 100000 {
            return Err(PixlieError::validation(
                "llm.max_tokens",
                "Maximum tokens cannot exceed 100,000",
                context,
            ));
        }
        
        Ok(())
    }
}

impl DatabaseConfig {
    /// Validate database configuration
    pub fn validate(&self) -> Result<()> {
        let context = ErrorContext::new().with_context("Database configuration validation");
        
        if self.connection_timeout > 300 {
            return Err(PixlieError::validation(
                "database.connection_timeout",
                "Connection timeout cannot exceed 300 seconds",
                context,
            ));
        }
        
        if self.query_timeout > 3600 {
            return Err(PixlieError::validation(
                "database.query_timeout",
                "Query timeout cannot exceed 3600 seconds",
                context,
            ));
        }
        
        if self.max_connections > 100 {
            return Err(PixlieError::validation(
                "database.max_connections",
                "Maximum connections cannot exceed 100",
                context,
            ));
        }
        
        if self.query_result_limit > 100000 {
            return Err(PixlieError::validation(
                "database.query_result_limit",
                "Query result limit cannot exceed 100,000 rows",
                context,
            ));
        }
        
        Ok(())
    }
}

impl ShortcutsConfig {
    /// Validate shortcuts configuration
    pub fn validate(&self) -> Result<()> {
        // For now, we'll accept any string as a valid shortcut
        // In a real implementation, we might validate against known key combinations
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_configs() {
        let global = GlobalConfig::default();
        assert_eq!(global.ui.theme, "dark");
        assert_eq!(global.ui.layout, "comfortable");
        assert_eq!(global.llm.default_model, "gpt-3.5-turbo");
        assert!(global.database.read_only);
    }

    #[test]
    fn test_ui_config_validation() {
        let mut ui = UiConfig::default();
        assert!(ui.validate().is_ok());
        
        ui.theme = "invalid".to_string();
        assert!(ui.validate().is_err());
        
        ui.theme = "dark".to_string();
        ui.max_chat_history = 20000;
        assert!(ui.validate().is_err());
    }

    #[test]
    fn test_llm_config_validation() {
        let mut llm = LlmConfig::default();
        assert!(llm.validate().is_ok());
        
        llm.temperature = 3.0;
        assert!(llm.validate().is_err());
        
        llm.temperature = 0.5;
        llm.max_iterations = 200;
        assert!(llm.validate().is_err());
    }

    #[test]
    fn test_serialization() {
        let config = GlobalConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: GlobalConfig = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.ui.theme, parsed.ui.theme);
        assert_eq!(config.llm.default_model, parsed.llm.default_model);
    }
}