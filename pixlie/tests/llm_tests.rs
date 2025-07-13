// pixlie/tests/llm_tests.rs

use pixlie::llm::{LLMProvider, mock::MockLLMProvider};

#[tokio::test]
async fn test_mock_llm_provider() {
    let provider = MockLLMProvider;
    let response = provider.send_query("test query", &[], None).await.unwrap();
    assert_eq!(response.content, "Mock response");
}
