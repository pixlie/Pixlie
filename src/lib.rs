//! Pixlie - LLM-enabled CLI data analysis tool
//! 
//! A Rust-based CLI application that leverages Large Language Models (LLMs) 
//! to perform intelligent data analysis on SQLite databases.

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
