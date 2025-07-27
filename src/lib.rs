//! Pixlie - LLM-enabled CLI data analysis tool
//! 
//! A Rust-based CLI application that leverages Large Language Models (LLMs) 
//! to perform intelligent data analysis on SQLite databases.

pub mod error;
pub mod logging;
pub mod config;

pub use error::{PixlieError, ErrorContext, ErrorContextExt, ErrorSeverity, Result};
pub use logging::{LoggingConfig, init_logging, log_error, log_performance_metric};
pub use config::{ConfigManager, GlobalConfig, WorkspaceConfig, ConfigLoader, CliArgs};

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
