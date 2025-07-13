// pixlie/src/llm/mock.rs

use super::{LLMError, LLMProvider, LLMResponse, ModelInfo, Tool, ToolCall};
use async_trait::async_trait;

pub struct MockLLMProvider;

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

    fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            provider: "mock".to_string(),
            model_name: "mock-model".to_string(),
        }
    }
}
