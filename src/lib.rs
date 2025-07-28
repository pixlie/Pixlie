//! Pixlie - LLM-enabled CLI data analysis tool
//!
//! A Rust-based CLI application that leverages Large Language Models (LLMs)
//! to perform intelligent data analysis on SQLite databases.

pub mod config;
pub mod error;
pub mod logging;
pub mod tui;

pub use config::{CliArgs, ConfigLoader, ConfigManager, GlobalConfig, WorkspaceConfig};
pub use error::{ErrorContext, ErrorContextExt, ErrorSeverity, PixlieError, Result};
pub use logging::{init_logging, log_error, log_performance_metric, LoggingConfig};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
