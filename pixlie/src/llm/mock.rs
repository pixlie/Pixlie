// pixlie/src/llm/mock.rs

use super::{LLMError, LLMProvider, LLMResponse, ModelInfo, Tool, ToolCall};
use async_trait::async_trait;

pub struct MockLLMProvider;

impl Default for MockLLMProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MockLLMProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LLMProvider for MockLLMProvider {
    async fn send_query(
        &self,
        _query: &str,
        _tools: &[Tool],
        _context: Option<&str>,
    ) -> Result<LLMResponse, LLMError> {
        Ok(LLMResponse {
            content: "Mock response".to_string(),
        })
    }

    async fn parse_response(&self, _response: &str) -> Result<Vec<ToolCall>, LLMError> {
        Ok(vec![])
    }

    async fn generate_response(
        &self,
        _prompt: &str,
        _tools: &[super::super::conversation::ToolDescriptor],
    ) -> Result<String, LLMError> {
        Ok("Mock LLM response".to_string())
    }

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "mock".to_string(),
            model_name: "mock-model".to_string(),
        }
    }
}
