//! Comprehensive error handling system for Pixlie TUI application
//! 
//! This module provides a structured error hierarchy using thiserror,
//! with proper error context and correlation IDs for debugging and monitoring.

use std::fmt;
use thiserror::Error;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Error context with correlation ID and timestamp for debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Unique correlation ID for tracking errors across components
    pub correlation_id: Uuid,
    /// Timestamp when the error occurred
    pub timestamp: DateTime<Utc>,
    /// Optional session ID for tracking user sessions
    pub session_id: Option<Uuid>,
    /// Optional objective ID for tracking analysis objectives
    pub objective_id: Option<Uuid>,
    /// Additional context information
    pub context: Option<String>,
}

impl ErrorContext {
    /// Create a new error context with generated correlation ID
    pub fn new() -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            session_id: None,
            objective_id: None,
            context: None,
        }
    }

    /// Create error context with session information
    pub fn with_session(session_id: Uuid) -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            session_id: Some(session_id),
            objective_id: None,
            context: None,
        }
    }

    /// Create error context with objective information
    pub fn with_objective(session_id: Uuid, objective_id: Uuid) -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            session_id: Some(session_id),
            objective_id: Some(objective_id),
            context: None,
        }
    }

    /// Add additional context information
    pub fn with_context<S: Into<String>>(mut self, context: S) -> Self {
        self.context = Some(context.into());
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.correlation_id)?;
        if let Some(session_id) = &self.session_id {
            write!(f, " session:{}", session_id)?;
        }
        if let Some(objective_id) = &self.objective_id {
            write!(f, " objective:{}", objective_id)?;
        }
        if let Some(context) = &self.context {
            write!(f, " {}", context)?;
        }
        Ok(())
    }
}

/// Main error type for the Pixlie application
#[derive(Error, Debug)]
pub enum PixlieError {
    /// TUI interface rendering, input handling, and layout errors
    #[error("TUI error: {message}")]
    Tui {
        message: String,
        context: ErrorContext,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Workspace, history, and persistence errors
    #[error("Session error: {message}")]
    Session {
        message: String,
        context: ErrorContext,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Connection, query, and schema errors
    #[error("Database error: {message}")]
    Database {
        message: String,
        context: ErrorContext,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// API, authentication, and rate limit errors
    #[error("LLM provider error: {message}")]
    LlmProvider {
        message: String,
        context: ErrorContext,
        provider: String,
        retryable: bool,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Tool-specific execution failures
    #[error("Tool execution error: {tool_name} - {message}")]
    ToolExecution {
        tool_name: String,
        message: String,
        context: ErrorContext,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Invalid configuration and missing parameters
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        context: ErrorContext,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Multi-objective analysis workflow failures
    #[error("Analysis error: {message}")]
    Analysis {
        message: String,
        context: ErrorContext,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Input validation and schema violations
    #[error("Validation error: {field} - {message}")]
    Validation {
        field: String,
        message: String,
        context: ErrorContext,
    },
}

impl PixlieError {
    /// Get the error context
    pub fn context(&self) -> &ErrorContext {
        match self {
            PixlieError::Tui { context, .. } => context,
            PixlieError::Session { context, .. } => context,
            PixlieError::Database { context, .. } => context,
            PixlieError::LlmProvider { context, .. } => context,
            PixlieError::ToolExecution { context, .. } => context,
            PixlieError::Configuration { context, .. } => context,
            PixlieError::Analysis { context, .. } => context,
            PixlieError::Validation { context, .. } => context,
        }
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            PixlieError::LlmProvider { retryable, .. } => *retryable,
            PixlieError::Database { .. } => false, // Database errors typically not retryable
            PixlieError::ToolExecution { .. } => true, // Tool execution might be retryable
            _ => false,
        }
    }

    /// Get user-friendly error message for TUI display
    pub fn user_message(&self) -> String {
        match self {
            PixlieError::Tui { message, .. } => {
                format!("Interface Error: {}", message)
            }
            PixlieError::Session { message, .. } => {
                format!("Session Error: {}", message)
            }
            PixlieError::Database { message, .. } => {
                format!("Database Error: {}", message)
            }
            PixlieError::LlmProvider { message, provider, .. } => {
                format!("AI Provider Error ({}): {}", provider, message)
            }
            PixlieError::ToolExecution { tool_name, message, .. } => {
                format!("Tool Error ({}): {}", tool_name, message)
            }
            PixlieError::Configuration { message, .. } => {
                format!("Configuration Error: {}", message)
            }
            PixlieError::Analysis { message, .. } => {
                format!("Analysis Error: {}", message)
            }
            PixlieError::Validation { field, message, .. } => {
                format!("Validation Error ({}): {}", field, message)
            }
        }
    }

    /// Create a TUI error
    pub fn tui<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        PixlieError::Tui {
            message: message.into(),
            context,
            source: None,
        }
    }

    /// Create a TUI error with source
    pub fn tui_with_source<S: Into<String>>(
        message: S,
        context: ErrorContext,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        PixlieError::Tui {
            message: message.into(),
            context,
            source: Some(source),
        }
    }

    /// Create a session error
    pub fn session<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        PixlieError::Session {
            message: message.into(),
            context,
            source: None,
        }
    }

    /// Create a database error
    pub fn database<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        PixlieError::Database {
            message: message.into(),
            context,
            source: None,
        }
    }

    /// Create an LLM provider error
    pub fn llm_provider<S: Into<String>>(
        message: S,
        provider: S,
        retryable: bool,
        context: ErrorContext,
    ) -> Self {
        PixlieError::LlmProvider {
            message: message.into(),
            provider: provider.into(),
            retryable,
            context,
            source: None,
        }
    }

    /// Create a tool execution error
    pub fn tool_execution<S: Into<String>>(
        tool_name: S,
        message: S,
        context: ErrorContext,
    ) -> Self {
        PixlieError::ToolExecution {
            tool_name: tool_name.into(),
            message: message.into(),
            context,
            source: None,
        }
    }

    /// Create a configuration error
    pub fn configuration<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        PixlieError::Configuration {
            message: message.into(),
            context,
            source: None,
        }
    }

    /// Create an analysis error
    pub fn analysis<S: Into<String>>(message: S, context: ErrorContext) -> Self {
        PixlieError::Analysis {
            message: message.into(),
            context,
            source: None,
        }
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(field: S, message: S, context: ErrorContext) -> Self {
        PixlieError::Validation {
            field: field.into(),
            message: message.into(),
            context,
        }
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, PixlieError>;

/// Error severity levels for logging and display
#[derive(Debug, Clone, Copy)]
pub enum ErrorSeverity {
    /// Low severity - informational errors
    Low,
    /// Medium severity - recoverable errors
    Medium,
    /// High severity - critical errors that may affect functionality
    High,
    /// Critical severity - errors that require immediate attention
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "LOW"),
            ErrorSeverity::Medium => write!(f, "MEDIUM"),
            ErrorSeverity::High => write!(f, "HIGH"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl PixlieError {
    /// Get the severity level of the error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            PixlieError::Validation { .. } => ErrorSeverity::Low,
            PixlieError::ToolExecution { .. } => ErrorSeverity::Medium,
            PixlieError::LlmProvider { .. } => ErrorSeverity::Medium,
            PixlieError::Tui { .. } => ErrorSeverity::High,
            PixlieError::Analysis { .. } => ErrorSeverity::High,
            PixlieError::Database { .. } => ErrorSeverity::High,
            PixlieError::Session { .. } => ErrorSeverity::High,
            PixlieError::Configuration { .. } => ErrorSeverity::Critical,
        }
    }
}

// Error conversion implementations for external dependencies

/// Convert from std::io::Error
impl From<std::io::Error> for PixlieError {
    fn from(err: std::io::Error) -> Self {
        let context = ErrorContext::new().with_context("IO operation failed");
        match err.kind() {
            std::io::ErrorKind::NotFound => PixlieError::Session {
                message: "File or directory not found".to_string(),
                context,
                source: Some(Box::new(err)),
            },
            std::io::ErrorKind::PermissionDenied => PixlieError::Session {
                message: "Permission denied".to_string(),
                context,
                source: Some(Box::new(err)),
            },
            _ => PixlieError::Session {
                message: format!("IO error: {}", err),
                context,
                source: Some(Box::new(err)),
            },
        }
    }
}

/// Convert from serde_json::Error
impl From<serde_json::Error> for PixlieError {
    fn from(err: serde_json::Error) -> Self {
        let context = ErrorContext::new().with_context("JSON serialization/deserialization failed");
        PixlieError::Session {
            message: format!("JSON error: {}", err),
            context,
            source: Some(Box::new(err)),
        }
    }
}

/// Convert from tokio task join errors
impl From<tokio::task::JoinError> for PixlieError {
    fn from(err: tokio::task::JoinError) -> Self {
        let context = ErrorContext::new().with_context("Async task join failed");
        PixlieError::Analysis {
            message: format!("Task join error: {}", err),
            context,
            source: Some(Box::new(err)),
        }
    }
}

/// Convert from UUID parsing errors
impl From<uuid::Error> for PixlieError {
    fn from(err: uuid::Error) -> Self {
        let context = ErrorContext::new().with_context("UUID parsing failed");
        PixlieError::Validation {
            field: "uuid".to_string(),
            message: format!("Invalid UUID: {}", err),
            context,
        }
    }
}

/// Convert from tracing subscriber initialization errors
impl From<tracing_subscriber::util::TryInitError> for PixlieError {
    fn from(err: tracing_subscriber::util::TryInitError) -> Self {
        let context = ErrorContext::new().with_context("Logging initialization failed");
        PixlieError::Configuration {
            message: format!("Failed to initialize logging: {}", err),
            context,
            source: Some(Box::new(err)),
        }
    }
}

/// Convert from environment variable errors
impl From<std::env::VarError> for PixlieError {
    fn from(err: std::env::VarError) -> Self {
        let context = ErrorContext::new().with_context("Environment variable access failed");
        PixlieError::Configuration {
            message: format!("Environment variable error: {}", err),
            context,
            source: Some(Box::new(err)),
        }
    }
}

/// Helper trait for adding error context to Results
pub trait ErrorContextExt<T> {
    /// Add context to an error
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> ErrorContext;

    /// Add context with a simple message
    fn with_context_msg<S: Into<String>>(self, msg: S) -> Result<T>;

    /// Add session context
    fn with_session_context(self, session_id: Uuid) -> Result<T>;

    /// Add objective context
    fn with_objective_context(self, session_id: Uuid, objective_id: Uuid) -> Result<T>;
}

impl<T, E> ErrorContextExt<T> for std::result::Result<T, E>
where
    E: Into<PixlieError>,
{
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> ErrorContext,
    {
        self.map_err(|e| {
            let mut error = e.into();
            // Update the context if it's a PixlieError
            match &mut error {
                PixlieError::Tui { context, .. } |
                PixlieError::Session { context, .. } |
                PixlieError::Database { context, .. } |
                PixlieError::LlmProvider { context, .. } |
                PixlieError::ToolExecution { context, .. } |
                PixlieError::Configuration { context, .. } |
                PixlieError::Analysis { context, .. } |
                PixlieError::Validation { context, .. } => {
                    let new_context = f();
                    // Preserve the original correlation_id and timestamp, but update other fields
                    if new_context.session_id.is_some() {
                        context.session_id = new_context.session_id;
                    }
                    if new_context.objective_id.is_some() {
                        context.objective_id = new_context.objective_id;
                    }
                    if new_context.context.is_some() {
                        context.context = new_context.context;
                    }
                }
            }
            error
        })
    }

    fn with_context_msg<S: Into<String>>(self, msg: S) -> Result<T> {
        self.with_context(|| ErrorContext::new().with_context(msg))
    }

    fn with_session_context(self, session_id: Uuid) -> Result<T> {
        self.with_context(|| ErrorContext::with_session(session_id))
    }

    fn with_objective_context(self, session_id: Uuid, objective_id: Uuid) -> Result<T> {
        self.with_context(|| ErrorContext::with_objective(session_id, objective_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext::new();
        assert!(context.session_id.is_none());
        assert!(context.objective_id.is_none());
        assert!(context.context.is_none());
    }

    #[test]
    fn test_error_context_with_session() {
        let session_id = Uuid::new_v4();
        let context = ErrorContext::with_session(session_id);
        assert_eq!(context.session_id, Some(session_id));
        assert!(context.objective_id.is_none());
    }

    #[test]
    fn test_error_context_with_objective() {
        let session_id = Uuid::new_v4();
        let objective_id = Uuid::new_v4();
        let context = ErrorContext::with_objective(session_id, objective_id);
        assert_eq!(context.session_id, Some(session_id));
        assert_eq!(context.objective_id, Some(objective_id));
    }

    #[test]
    fn test_error_context_with_context() {
        let context = ErrorContext::new().with_context("test context");
        assert_eq!(context.context, Some("test context".to_string()));
    }

    #[test]
    fn test_pixlie_error_creation() {
        let context = ErrorContext::new();
        let error = PixlieError::tui("Test error", context);
        
        match error {
            PixlieError::Tui { message, .. } => {
                assert_eq!(message, "Test error");
            }
            _ => panic!("Expected TUI error"),
        }
    }

    #[test]
    fn test_error_retryable() {
        let context = ErrorContext::new();
        let retryable_error = PixlieError::llm_provider("API error", "openai", true, context.clone());
        let non_retryable_error = PixlieError::database("Connection failed", context);

        assert!(retryable_error.is_retryable());
        assert!(!non_retryable_error.is_retryable());
    }

    #[test]
    fn test_error_severity() {
        let context = ErrorContext::new();
        
        let validation_error = PixlieError::validation("field", "invalid", context.clone());
        let config_error = PixlieError::configuration("missing config", context);

        assert!(matches!(validation_error.severity(), ErrorSeverity::Low));
        assert!(matches!(config_error.severity(), ErrorSeverity::Critical));
    }

    #[test]
    fn test_user_message() {
        let context = ErrorContext::new();
        let error = PixlieError::tui("Interface rendering failed", context);
        let user_msg = error.user_message();
        assert!(user_msg.contains("Interface Error"));
        assert!(user_msg.contains("Interface rendering failed"));
    }
}