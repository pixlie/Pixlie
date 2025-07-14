use super::*;
use crate::tools::ToolRegistry;
use futures::future::join_all;
use std::sync::Arc;
use tokio::time::{Duration, Instant, timeout};

#[allow(dead_code)]
pub struct ToolExecutor {
    tool_registry: Arc<ToolRegistry>,
    max_parallel_executions: usize,
    execution_timeout: Duration,
}

#[allow(dead_code)]
impl ToolExecutor {
    pub fn new(tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            tool_registry,
            max_parallel_executions: 4,
            execution_timeout: Duration::from_secs(30),
        }
    }

    pub fn with_config(
        tool_registry: Arc<ToolRegistry>,
        max_parallel_executions: usize,
        execution_timeout: Duration,
    ) -> Self {
        Self {
            tool_registry,
            max_parallel_executions,
            execution_timeout,
        }
    }

    pub async fn execute_step(
        &self,
        step: &ConversationStep,
    ) -> Result<Vec<ToolExecution>, ConversationError> {
        match step.step_type {
            StepType::ToolExecution => self.execute_tool_calls(&step.tool_calls).await,
            _ => Err(ConversationError::ToolExecutionError(
                "Step is not a tool execution step".to_string(),
            )),
        }
    }

    pub async fn execute_plan_step(
        &self,
        plan_step: &super::planner::PlanStep,
        _context: &ConversationContext,
    ) -> Result<ToolExecution, ConversationError> {
        let tool_call = ToolExecution {
            tool_name: plan_step.tool_name.clone(),
            parameters: plan_step.parameters.clone(),
            result: None,
            error: None,
            execution_time_ms: None,
        };

        let executed_calls = self.execute_tool_calls(&[tool_call]).await?;

        executed_calls.into_iter().next().ok_or_else(|| {
            ConversationError::ToolExecutionError("No tool execution result".to_string())
        })
    }

    async fn execute_tool_calls(
        &self,
        tool_calls: &[ToolExecution],
    ) -> Result<Vec<ToolExecution>, ConversationError> {
        let mut results = Vec::new();

        // Group tool calls by parallelization capability
        let (parallel_calls, sequential_calls) = self.group_tool_calls(tool_calls);

        // Execute parallel calls first
        if !parallel_calls.is_empty() {
            let parallel_results = self.execute_parallel_tools(&parallel_calls).await?;
            results.extend(parallel_results);
        }

        // Execute sequential calls
        for tool_call in sequential_calls {
            let result = self.execute_single_tool(&tool_call).await?;
            results.push(result);
        }

        Ok(results)
    }

    async fn execute_parallel_tools(
        &self,
        tool_calls: &[ToolExecution],
    ) -> Result<Vec<ToolExecution>, ConversationError> {
        // Limit parallel executions
        let chunks: Vec<_> = tool_calls.chunks(self.max_parallel_executions).collect();
        let mut all_results = Vec::new();

        for chunk in chunks {
            let futures: Vec<_> = chunk
                .iter()
                .map(|tool_call| self.execute_single_tool(tool_call))
                .collect();

            let chunk_results = join_all(futures).await;

            for result in chunk_results {
                match result {
                    Ok(execution) => all_results.push(execution),
                    Err(e) => {
                        // Continue with other executions even if one fails
                        log::warn!("Tool execution failed: {e}");
                        // Create a failed execution record
                        all_results.push(ToolExecution {
                            tool_name: "unknown".to_string(),
                            parameters: serde_json::json!({}),
                            result: None,
                            error: Some(e.to_string()),
                            execution_time_ms: None,
                        });
                    }
                }
            }
        }

        Ok(all_results)
    }

    async fn execute_single_tool(
        &self,
        tool_call: &ToolExecution,
    ) -> Result<ToolExecution, ConversationError> {
        let start_time = Instant::now();

        let result = timeout(
            self.execution_timeout,
            self.tool_registry
                .execute_tool(&tool_call.tool_name, &tool_call.parameters),
        )
        .await;

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        let mut executed_call = tool_call.clone();
        executed_call.execution_time_ms = Some(execution_time_ms);

        match result {
            Ok(Ok(tool_result)) => {
                executed_call.result = Some(tool_result);
                Ok(executed_call)
            }
            Ok(Err(tool_error)) => {
                executed_call.error = Some(tool_error.to_string());
                Ok(executed_call)
            }
            Err(_) => {
                executed_call.error = Some("Tool execution timeout".to_string());
                Ok(executed_call)
            }
        }
    }

    fn group_tool_calls(
        &self,
        tool_calls: &[ToolExecution],
    ) -> (Vec<ToolExecution>, Vec<ToolExecution>) {
        let mut parallel_calls = Vec::new();
        let mut sequential_calls = Vec::new();

        for tool_call in tool_calls {
            if self.can_run_parallel(&tool_call.tool_name) {
                parallel_calls.push(tool_call.clone());
            } else {
                sequential_calls.push(tool_call.clone());
            }
        }

        (parallel_calls, sequential_calls)
    }

    fn can_run_parallel(&self, tool_name: &str) -> bool {
        // Determine if a tool can safely run in parallel
        match tool_name {
            // Read-only operations can usually run in parallel
            name if name.contains("search") => true,
            name if name.contains("get") => true,
            name if name.contains("list") => true,
            name if name.contains("query") => true,
            name if name.contains("analyze") => true,

            // Write operations should typically be sequential
            name if name.contains("create") => false,
            name if name.contains("update") => false,
            name if name.contains("delete") => false,
            name if name.contains("modify") => false,

            // Default to sequential for safety
            _ => false,
        }
    }

    pub async fn execute_with_retry(
        &self,
        tool_call: &ToolExecution,
        max_retries: u32,
        retry_delay: Duration,
    ) -> Result<ToolExecution, ConversationError> {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match self.execute_single_tool(tool_call).await {
                Ok(result) => {
                    if result.error.is_none() {
                        return Ok(result);
                    }
                    last_error = result.error.clone();
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                }
            }

            if attempt < max_retries {
                tokio::time::sleep(retry_delay * (attempt + 1)).await;
            }
        }

        Err(ConversationError::ToolExecutionError(
            last_error.unwrap_or_else(|| "Tool execution failed after retries".to_string()),
        ))
    }

    pub async fn execute_with_fallback(
        &self,
        primary_tool: &ToolExecution,
        fallback_tools: &[ToolExecution],
    ) -> Result<ToolExecution, ConversationError> {
        // Try primary tool first
        let primary_result = self.execute_single_tool(primary_tool).await?;

        if primary_result.error.is_none() {
            return Ok(primary_result);
        }

        // Try fallback tools
        for fallback_tool in fallback_tools {
            let fallback_result = self.execute_single_tool(fallback_tool).await;
            if let Ok(result) = fallback_result {
                if result.error.is_none() {
                    return Ok(result);
                }
            }
        }

        // If all fallbacks fail, return the primary result with error
        Ok(primary_result)
    }

    pub fn aggregate_results(
        &self,
        executions: &[ToolExecution],
    ) -> Result<serde_json::Value, ConversationError> {
        let mut aggregated = serde_json::Map::new();
        let mut errors = Vec::new();

        for execution in executions {
            if let Some(error) = &execution.error {
                errors.push(format!("{}: {}", execution.tool_name, error));
            } else if let Some(result) = &execution.result {
                aggregated.insert(execution.tool_name.clone(), result.clone());
            }
        }

        let mut final_result = serde_json::Map::new();
        final_result.insert("results".to_string(), serde_json::Value::Object(aggregated));

        if !errors.is_empty() {
            final_result.insert(
                "errors".to_string(),
                serde_json::Value::Array(
                    errors.into_iter().map(serde_json::Value::String).collect(),
                ),
            );
        }

        let total_execution_time: u64 = executions.iter().filter_map(|e| e.execution_time_ms).sum();

        final_result.insert(
            "total_execution_time_ms".to_string(),
            serde_json::Value::Number(serde_json::Number::from(total_execution_time)),
        );

        Ok(serde_json::Value::Object(final_result))
    }

    pub fn get_execution_metrics(&self, executions: &[ToolExecution]) -> ExecutionMetrics {
        let total_executions = executions.len();
        let successful_executions = executions.iter().filter(|e| e.error.is_none()).count();
        let failed_executions = total_executions - successful_executions;

        let total_time_ms = executions.iter().filter_map(|e| e.execution_time_ms).sum();

        let average_time_ms = if total_executions > 0 {
            total_time_ms / total_executions as u64
        } else {
            0
        };

        let max_time_ms = executions
            .iter()
            .filter_map(|e| e.execution_time_ms)
            .max()
            .unwrap_or(0);

        ExecutionMetrics {
            total_executions,
            successful_executions,
            failed_executions,
            total_time_ms,
            average_time_ms,
            max_time_ms,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub total_time_ms: u64,
    pub average_time_ms: u64,
    pub max_time_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::ToolRegistry;

    fn create_mock_tool_registry() -> Arc<ToolRegistry> {
        Arc::new(ToolRegistry::new())
    }

    #[tokio::test]
    async fn test_tool_executor_single_execution() {
        let tool_registry = create_mock_tool_registry();
        let executor = ToolExecutor::new(tool_registry);

        let tool_call = ToolExecution {
            tool_name: "test_tool".to_string(),
            parameters: serde_json::json!({"param": "value"}),
            result: None,
            error: None,
            execution_time_ms: None,
        };

        let result = executor.execute_single_tool(&tool_call).await;
        assert!(result.is_ok());

        let executed = result.unwrap();
        assert!(executed.execution_time_ms.is_some());
    }

    #[tokio::test]
    async fn test_tool_grouping() {
        let tool_registry = create_mock_tool_registry();
        let executor = ToolExecutor::new(tool_registry);

        let tool_calls = vec![
            ToolExecution {
                tool_name: "search_entities".to_string(),
                parameters: serde_json::json!({}),
                result: None,
                error: None,
                execution_time_ms: None,
            },
            ToolExecution {
                tool_name: "create_entity".to_string(),
                parameters: serde_json::json!({}),
                result: None,
                error: None,
                execution_time_ms: None,
            },
        ];

        let (parallel, sequential) = executor.group_tool_calls(&tool_calls);

        assert_eq!(parallel.len(), 1); // search_entities should be parallel
        assert_eq!(sequential.len(), 1); // create_entity should be sequential
    }
}
