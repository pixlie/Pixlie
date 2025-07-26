//! Structured logging system for Pixlie TUI application
//! 
//! Provides logging utilities for different components with proper
//! context propagation and structured output.

use tracing::{info, warn, error, debug, trace};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use uuid::Uuid;
use std::io;
use crate::error::{PixlieError, ErrorContext, ErrorSeverity};

/// Logging configuration for the application
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Enable JSON output format
    pub json_format: bool,
    /// Log level filter
    pub level: String,
    /// Enable colored output (for non-JSON format)
    pub colored: bool,
    /// Log file path (optional)
    pub file_path: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            json_format: false,
            level: "info".to_string(),
            colored: true,
            file_path: None,
        }
    }
}

/// Initialize the logging system with the given configuration
pub fn init_logging(config: LoggingConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    let subscriber_builder = tracing_subscriber::registry().with(env_filter);

    if config.json_format {
        let json_layer = fmt::layer()
            .json()
            .with_writer(io::stderr);
        subscriber_builder.with(json_layer).try_init()?;
    } else {
        let fmt_layer = fmt::layer()
            .with_ansi(config.colored)
            .with_writer(io::stderr);
        subscriber_builder.with(fmt_layer).try_init()?;
    }

    info!("Logging system initialized with level: {}", config.level);
    Ok(())
}

/// Component-specific logger for TUI operations
pub struct TuiLogger;

impl TuiLogger {
    /// Log TUI state changes
    pub fn state_change(event: &str, details: Option<&str>) {
        match details {
            Some(details) => debug!(component = "tui", event = event, details = details, "TUI state change"),
            None => debug!(component = "tui", event = event, "TUI state change"),
        }
    }

    /// Log user interactions
    pub fn user_interaction(action: &str, context: Option<&str>) {
        match context {
            Some(context) => info!(component = "tui", action = action, context = context, "User interaction"),
            None => info!(component = "tui", action = action, "User interaction"),
        }
    }

    /// Log rendering events
    pub fn rendering(area: &str, duration_ms: Option<u64>) {
        match duration_ms {
            Some(duration) => trace!(component = "tui", area = area, duration_ms = duration, "UI rendering"),
            None => trace!(component = "tui", area = area, "UI rendering"),
        }
    }

    /// Log input events
    pub fn input_event(event_type: &str, key: Option<&str>) {
        match key {
            Some(key) => debug!(component = "tui", event_type = event_type, key = key, "Input event"),
            None => debug!(component = "tui", event_type = event_type, "Input event"),
        }
    }
}

/// Component-specific logger for session operations
pub struct SessionLogger;

impl SessionLogger {
    /// Log session lifecycle events
    pub fn lifecycle(event: &str, session_id: Option<Uuid>) {
        match session_id {
            Some(id) => info!(component = "session", event = event, session_id = %id, "Session lifecycle"),
            None => info!(component = "session", event = event, "Session lifecycle"),
        }
    }

    /// Log workspace operations
    pub fn workspace_operation(operation: &str, path: &str, result: &str) {
        info!(
            component = "session",
            operation = operation,
            path = path,
            result = result,
            "Workspace operation"
        );
    }

    /// Log history operations
    pub fn history_operation(operation: &str, session_id: Uuid, size: Option<usize>) {
        match size {
            Some(size) => debug!(
                component = "session",
                operation = operation,
                session_id = %session_id,
                size = size,
                "History operation"
            ),
            None => debug!(
                component = "session",
                operation = operation,
                session_id = %session_id,
                "History operation"
            ),
        }
    }
}

/// Component-specific logger for tool operations
pub struct ToolLogger;

impl ToolLogger {
    /// Log tool execution start
    pub fn execution_start(tool_name: &str, context: &ErrorContext) {
        info!(
            component = "tool",
            tool_name = tool_name,
            correlation_id = %context.correlation_id,
            session_id = ?context.session_id,
            objective_id = ?context.objective_id,
            "Tool execution started"
        );
    }

    /// Log tool execution completion
    pub fn execution_complete(tool_name: &str, context: &ErrorContext, duration_ms: u64) {
        info!(
            component = "tool",
            tool_name = tool_name,
            correlation_id = %context.correlation_id,
            session_id = ?context.session_id,
            objective_id = ?context.objective_id,
            duration_ms = duration_ms,
            "Tool execution completed"
        );
    }

    /// Log tool execution failure
    pub fn execution_failed(tool_name: &str, context: &ErrorContext, error: &str) {
        error!(
            component = "tool",
            tool_name = tool_name,
            correlation_id = %context.correlation_id,
            session_id = ?context.session_id,
            objective_id = ?context.objective_id,
            error = error,
            "Tool execution failed"
        );
    }

    /// Log SQL query execution
    pub fn sql_query(query: &str, context: &ErrorContext, duration_ms: Option<u64>) {
        match duration_ms {
            Some(duration) => debug!(
                component = "tool",
                tool_name = "sql",
                correlation_id = %context.correlation_id,
                query = query,
                duration_ms = duration,
                "SQL query executed"
            ),
            None => debug!(
                component = "tool",
                tool_name = "sql",
                correlation_id = %context.correlation_id,
                query = query,
                "SQL query started"
            ),
        }
    }
}

/// Component-specific logger for LLM operations
pub struct LlmLogger;

impl LlmLogger {
    /// Log LLM request start
    pub fn request_start(provider: &str, model: &str, context: &ErrorContext) {
        info!(
            component = "llm",
            provider = provider,
            model = model,
            correlation_id = %context.correlation_id,
            session_id = ?context.session_id,
            objective_id = ?context.objective_id,
            "LLM request started"
        );
    }

    /// Log LLM request completion
    pub fn request_complete(
        provider: &str,
        model: &str,
        context: &ErrorContext,
        tokens_used: Option<u32>,
        duration_ms: u64,
    ) {
        match tokens_used {
            Some(tokens) => info!(
                component = "llm",
                provider = provider,
                model = model,
                correlation_id = %context.correlation_id,
                session_id = ?context.session_id,
                objective_id = ?context.objective_id,
                tokens_used = tokens,
                duration_ms = duration_ms,
                "LLM request completed"
            ),
            None => info!(
                component = "llm",
                provider = provider,
                model = model,
                correlation_id = %context.correlation_id,
                session_id = ?context.session_id,
                objective_id = ?context.objective_id,
                duration_ms = duration_ms,
                "LLM request completed"
            ),
        }
    }

    /// Log LLM request failure
    pub fn request_failed(provider: &str, model: &str, context: &ErrorContext, error: &str, retryable: bool) {
        warn!(
            component = "llm",
            provider = provider,
            model = model,
            correlation_id = %context.correlation_id,
            session_id = ?context.session_id,
            objective_id = ?context.objective_id,
            error = error,
            retryable = retryable,
            "LLM request failed"
        );
    }

    /// Log streaming events
    pub fn streaming_event(provider: &str, context: &ErrorContext, event_type: &str, chunk_size: Option<usize>) {
        match chunk_size {
            Some(size) => trace!(
                component = "llm",
                provider = provider,
                correlation_id = %context.correlation_id,
                event_type = event_type,
                chunk_size = size,
                "LLM streaming event"
            ),
            None => trace!(
                component = "llm",
                provider = provider,
                correlation_id = %context.correlation_id,
                event_type = event_type,
                "LLM streaming event"
            ),
        }
    }
}

/// Component-specific logger for analysis operations
pub struct AnalysisLogger;

impl AnalysisLogger {
    /// Log objective lifecycle events
    pub fn objective_lifecycle(event: &str, objective_id: Uuid, session_id: Uuid) {
        info!(
            component = "analysis",
            event = event,
            objective_id = %objective_id,
            session_id = %session_id,
            "Objective lifecycle"
        );
    }

    /// Log coordination events
    pub fn coordination_event(event: &str, session_id: Uuid, active_objectives: usize) {
        debug!(
            component = "analysis",
            event = event,
            session_id = %session_id,
            active_objectives = active_objectives,
            "Analysis coordination"
        );
    }

    /// Log analysis progress
    pub fn progress(objective_id: Uuid, phase: &str, progress_percent: Option<f32>) {
        match progress_percent {
            Some(percent) => info!(
                component = "analysis",
                objective_id = %objective_id,
                phase = phase,
                progress_percent = percent,
                "Analysis progress"
            ),
            None => info!(
                component = "analysis",
                objective_id = %objective_id,
                phase = phase,
                "Analysis progress"
            ),
        }
    }
}

/// Log error events with proper context and severity
pub fn log_error(error: &PixlieError) {
    let severity = error.severity();
    let context = error.context();
    let user_message = error.user_message();

    match severity {
        ErrorSeverity::Low => {
            debug!(
                severity = %severity,
                correlation_id = %context.correlation_id,
                session_id = ?context.session_id,
                objective_id = ?context.objective_id,
                error = %error,
                user_message = user_message,
                "Low severity error"
            );
        }
        ErrorSeverity::Medium => {
            warn!(
                severity = %severity,
                correlation_id = %context.correlation_id,
                session_id = ?context.session_id,
                objective_id = ?context.objective_id,
                error = %error,
                user_message = user_message,
                "Medium severity error"
            );
        }
        ErrorSeverity::High => {
            error!(
                severity = %severity,
                correlation_id = %context.correlation_id,
                session_id = ?context.session_id,
                objective_id = ?context.objective_id,
                error = %error,
                user_message = user_message,
                "High severity error"
            );
        }
        ErrorSeverity::Critical => {
            error!(
                severity = %severity,
                correlation_id = %context.correlation_id,
                session_id = ?context.session_id,
                objective_id = ?context.objective_id,
                error = %error,
                user_message = user_message,
                "Critical severity error"
            );
        }
    }
}

/// Log performance metrics
pub fn log_performance_metric(component: &str, operation: &str, duration_ms: u64, additional_metrics: Option<&str>) {
    match additional_metrics {
        Some(metrics) => info!(
            component = component,
            operation = operation,
            duration_ms = duration_ms,
            additional_metrics = metrics,
            "Performance metric"
        ),
        None => info!(
            component = component,
            operation = operation,
            duration_ms = duration_ms,
            "Performance metric"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert!(!config.json_format);
        assert_eq!(config.level, "info");
        assert!(config.colored);
        assert!(config.file_path.is_none());
    }

    #[test]
    fn test_error_logging() {
        let context = ErrorContext::new();
        let error = PixlieError::tui("Test error", context);
        
        // This would normally log, but in tests we just verify it doesn't panic
        log_error(&error);
    }

    #[test]
    fn test_performance_logging() {
        log_performance_metric("test", "operation", 100, Some("extra data"));
        log_performance_metric("test", "operation", 100, None);
    }
}