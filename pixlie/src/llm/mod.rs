// pixlie/src/llm/mod.rs

#![allow(dead_code)]

pub mod mock;
pub mod tool_registry;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Define the error type for LLM operations
#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("Failed to send query: {0}")]
    QueryError(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),
}

// Define the response from the LLM
#[derive(Debug, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
}

// Define the information about the LLM model
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub provider: String,
    pub model_name: String,
}

// Define the parameters for a tool
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolParameters {
    pub r#type: String,
    pub properties: Value,
    pub required: Vec<String>,
}

// Define the handler for a tool
#[async_trait]
pub trait ToolHandler: Send + Sync {
    async fn handle(&self, args: Value) -> Result<Value, String>;
}

// Define a tool that can be used by the LLM
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: ToolParameters,
    pub handler: Box<dyn ToolHandler>,
}

// Define a tool call from the LLM
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub arguments: Value,
    pub call_id: String,
}

// Define the trait for an LLM provider
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn send_query(
        &self,
        query: &str,
        tools: &[Tool],
        context: Option<&str>,
    ) -> Result<LLMResponse, LLMError>;

    async fn parse_response(&self, response: &str) -> Result<Vec<ToolCall>, LLMError>;

    fn get_model_info(&self) -> ModelInfo;
}
