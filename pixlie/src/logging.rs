use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::Level;
use tracing_subscriber::{EnvFilter, fmt, prelude::*, registry::Registry};
use uuid::Uuid;

/// Structured log entry for database storage and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub component: String,
    pub operation: String,
    pub message: String,
    pub context: Option<LogContext>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
    pub request_id: Option<String>,
}

/// Context information for log entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogContext {
    pub request_id: Option<String>,
    pub task_id: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub correlation_id: Option<String>,
}

/// Log output format configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Text,
    Json,
    Compact,
}

impl Default for LogFormat {
    fn default() -> Self {
        LogFormat::Text
    }
}

impl FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(LogFormat::Text),
            "json" => Ok(LogFormat::Json),
            "compact" => Ok(LogFormat::Compact),
            _ => Err(format!("Unknown log format: {}", s)),
        }
    }
}

/// Log output destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Console,
    File(PathBuf),
    Database,
}

/// File rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRotationConfig {
    pub max_size_mb: u64,
    pub max_files: u32,
    pub rotation_daily: bool,
}

impl Default for FileRotationConfig {
    fn default() -> Self {
        Self {
            max_size_mb: 100,
            max_files: 10,
            rotation_daily: true,
        }
    }
}

/// Main logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String, // Store as string to avoid serialization issues
    pub format: LogFormat,
    pub outputs: Vec<LogOutput>,
    pub file_rotation: Option<FileRotationConfig>,
    pub database_logging: bool,
    pub request_tracking: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "INFO".to_string(),
            format: LogFormat::Text,
            outputs: vec![LogOutput::Console],
            file_rotation: None,
            database_logging: false,
            request_tracking: true,
        }
    }
}

impl LoggingConfig {
    /// Create logging configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = LoggingConfig::default();

        // Set log level from environment
        if let Ok(level_str) = env::var("PIXLIE_LOG_LEVEL") {
            config.level = level_str.to_uppercase();
        }

        // Set log format from environment
        if let Ok(format_str) = env::var("PIXLIE_LOG_FORMAT") {
            if let Ok(format) = LogFormat::from_str(&format_str) {
                config.format = format;
            }
        }

        // Configure file output if specified
        if let Ok(log_file) = env::var("PIXLIE_LOG_FILE") {
            config.outputs.push(LogOutput::File(PathBuf::from(log_file)));
            config.file_rotation = Some(FileRotationConfig::default());
        }

        // Enable database logging if specified
        if let Ok(db_logging) = env::var("PIXLIE_LOG_DATABASE") {
            config.database_logging = db_logging.parse().unwrap_or(false);
            if config.database_logging {
                config.outputs.push(LogOutput::Database);
            }
        }

        // Configure request tracking
        if let Ok(tracking) = env::var("PIXLIE_LOG_REQUEST_TRACKING") {
            config.request_tracking = tracking.parse().unwrap_or(true);
        }

        config
    }
}

/// Logging manager for initializing and managing logging infrastructure
pub struct LoggingManager {
    config: LoggingConfig,
}

impl LoggingManager {
    /// Create a new logging manager with the given configuration
    pub fn new(config: LoggingConfig) -> Self {
        Self { config }
    }

    /// Initialize the logging infrastructure
    pub fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Parse level from string
        let level = Level::from_str(&self.config.level).unwrap_or(Level::INFO);
        
        // Create environment filter
        let env_filter = EnvFilter::builder()
            .with_default_directive(level.into())
            .from_env_lossy();

        // Create registry
        let registry = Registry::default().with(env_filter);

        // Configure outputs based on configuration
        let mut has_console = false;
        let mut has_file = false;

        for output in &self.config.outputs {
            match output {
                LogOutput::Console => has_console = true,
                LogOutput::File(_) => has_file = true,
                LogOutput::Database => {
                    // Database logging will be handled separately
                    // TODO: Implement database logging subscriber
                }
            }
        }

        // Configure console output
        if has_console {
            let console_layer = match self.config.format {
                LogFormat::Json => fmt::layer()
                    .json()
                    .with_current_span(true)
                    .with_span_list(true)
                    .boxed(),
                LogFormat::Compact => fmt::layer()
                    .compact()
                    .with_target(false)
                    .boxed(),
                LogFormat::Text => fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_thread_names(true)
                    .boxed(),
            };

            if has_file {
                // Configure file output
                if let Some(file_path) = self.get_file_path() {
                    let file_appender = tracing_appender::rolling::daily(
                        file_path.parent().unwrap_or(&PathBuf::from(".")),
                        file_path.file_name().unwrap_or(std::ffi::OsStr::new("pixlie.log")),
                    );
                    let (non_blocking_file, _guard) = tracing_appender::non_blocking(file_appender);
                    
                    let file_layer = fmt::layer()
                        .json()
                        .with_writer(non_blocking_file)
                        .with_ansi(false)
                        .boxed();

                    registry.with(console_layer).with(file_layer).init();
                } else {
                    registry.with(console_layer).init();
                }
            } else {
                registry.with(console_layer).init();
            }
        }

        tracing::info!(
            "Logging initialized with level: {} format: {:?} database_logging: {} request_tracking: {}",
            self.config.level,
            self.config.format,
            self.config.database_logging,
            self.config.request_tracking,
        );

        Ok(())
    }

    /// Get the file path for file output
    fn get_file_path(&self) -> Option<PathBuf> {
        for output in &self.config.outputs {
            if let LogOutput::File(path) = output {
                return Some(path.clone());
            }
        }
        None
    }

    /// Get current logging configuration
    pub fn get_config(&self) -> &LoggingConfig {
        &self.config
    }

    /// Update logging configuration (runtime configuration changes)
    pub fn update_config(&mut self, new_config: LoggingConfig) {
        self.config = new_config;
        // TODO: Implement runtime configuration updates
        tracing::info!("Logging configuration updated: {:?}", self.config);
    }
}

/// Generate a unique request ID
pub fn generate_request_id() -> String {
    Uuid::new_v4().to_string()
}

/// Create a log context with request tracking information
pub fn create_log_context(request_id: Option<String>) -> LogContext {
    LogContext {
        request_id,
        task_id: None,
        user_id: None,
        session_id: None,
        correlation_id: None,
    }
}

/// Structured logging macros for consistent log entry creation
#[macro_export]
macro_rules! log_operation_start {
    ($operation:expr, $component:expr, $($key:ident = $value:expr),*) => {
        tracing::info!(
            operation = $operation,
            component = $component,
            operation_status = "start",
            $($key = $value),*
        );
    };
}

#[macro_export]
macro_rules! log_operation_end {
    ($operation:expr, $component:expr, $duration_ms:expr, $($key:ident = $value:expr),*) => {
        tracing::info!(
            operation = $operation,
            component = $component,
            operation_status = "end",
            duration_ms = $duration_ms,
            $($key = $value),*
        );
    };
}

#[macro_export]
macro_rules! log_operation_error {
    ($operation:expr, $component:expr, $error:expr, $($key:ident = $value:expr),*) => {
        tracing::error!(
            operation = $operation,
            component = $component,
            operation_status = "error",
            error = %$error,
            $($key = $value),*
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_format_from_str() {
        assert!(matches!(LogFormat::from_str("json"), Ok(LogFormat::Json)));
        assert!(matches!(LogFormat::from_str("text"), Ok(LogFormat::Text)));
        assert!(matches!(LogFormat::from_str("compact"), Ok(LogFormat::Compact)));
        assert!(LogFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_default_logging_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "INFO");
        assert!(matches!(config.format, LogFormat::Text));
        assert_eq!(config.outputs.len(), 1);
        assert!(!config.database_logging);
        assert!(config.request_tracking);
    }

    #[test]
    fn test_generate_request_id() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();
        
        assert_ne!(id1, id2);
        assert!(Uuid::from_str(&id1).is_ok());
        assert!(Uuid::from_str(&id2).is_ok());
    }

    #[test]
    fn test_create_log_context() {
        let request_id = Some("test-request-id".to_string());
        let context = create_log_context(request_id.clone());
        
        assert_eq!(context.request_id, request_id);
        assert!(context.task_id.is_none());
        assert!(context.user_id.is_none());
    }
}