use super::*;
use crate::llm::LLMProvider;
use crate::tools::ToolRegistry;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::time::{Duration, timeout};

#[async_trait]
#[allow(dead_code)]
pub trait ConversationStore: Send + Sync {
    async fn save_conversation(&self, conversation: &Conversation)
    -> Result<(), ConversationError>;
    async fn load_conversation(&self, id: Uuid) -> Result<Option<Conversation>, ConversationError>;
    async fn update_conversation(
        &self,
        conversation: &Conversation,
    ) -> Result<(), ConversationError>;
    async fn delete_conversation(&self, id: Uuid) -> Result<(), ConversationError>;
    async fn list_conversations(
        &self,
        limit: Option<u32>,
    ) -> Result<Vec<Conversation>, ConversationError>;
}

#[allow(dead_code)]
pub struct ConversationManager {
    pub llm_provider: Box<dyn LLMProvider>,
    pub tool_registry: Arc<ToolRegistry>,
    pub conversation_store: Box<dyn ConversationStore>,
    pub max_steps: u32,
    pub step_timeout: Duration,
}

#[allow(dead_code)]
impl ConversationManager {
    pub fn new(
        llm_provider: Box<dyn LLMProvider>,
        tool_registry: Arc<ToolRegistry>,
        conversation_store: Box<dyn ConversationStore>,
    ) -> Self {
        Self {
            llm_provider,
            tool_registry,
            conversation_store,
            max_steps: 20,
            step_timeout: Duration::from_secs(60),
        }
    }

    pub async fn start_conversation(&self, query: &str) -> Result<Conversation, ConversationError> {
        let conversation_id = Uuid::new_v4();
        let now = Utc::now();

        let context = self.build_initial_context().await?;

        let mut conversation = Conversation {
            id: conversation_id,
            user_query: query.to_string(),
            state: ConversationState::Planning,
            steps: Vec::new(),
            context,
            created_at: now,
            updated_at: now,
        };

        // Start with planning step
        let planning_step = self.create_planning_step(&conversation).await?;
        conversation.steps.push(planning_step);
        conversation.updated_at = Utc::now();

        self.conversation_store
            .save_conversation(&conversation)
            .await?;
        Ok(conversation)
    }

    pub async fn continue_conversation(
        &self,
        conversation_id: Uuid,
        user_input: Option<&str>,
    ) -> Result<ConversationStep, ConversationError> {
        let mut conversation = self
            .conversation_store
            .load_conversation(conversation_id)
            .await?
            .ok_or_else(|| ConversationError::StorageError("Conversation not found".to_string()))?;

        // Check if conversation can continue
        if conversation.state == ConversationState::Completed
            || conversation.state == ConversationState::Failed
        {
            return Err(ConversationError::PlanningFailed(
                "Conversation already finished".to_string(),
            ));
        }

        if conversation.steps.len() >= self.max_steps as usize {
            conversation.state = ConversationState::Failed;
            self.conversation_store
                .update_conversation(&conversation)
                .await?;
            return Err(ConversationError::ConversationTimeout);
        }

        // Handle user input if provided
        if let Some(input) = user_input {
            if conversation.state == ConversationState::RequiresUserInput {
                conversation.state = ConversationState::Executing;
                // Add user input to context
                conversation.context.intermediate_results.insert(
                    "user_clarification".to_string(),
                    serde_json::Value::String(input.to_string()),
                );
            }
        }

        let next_step = match conversation.state {
            ConversationState::Planning => self.execute_planning_step(&mut conversation).await?,
            ConversationState::Executing => self.execute_tool_step(&mut conversation).await?,
            ConversationState::Synthesizing => {
                self.execute_synthesis_step(&mut conversation).await?
            }
            _ => {
                return Err(ConversationError::PlanningFailed(
                    "Invalid conversation state".to_string(),
                ));
            }
        };

        conversation.steps.push(next_step.clone());
        conversation.updated_at = Utc::now();

        self.conversation_store
            .update_conversation(&conversation)
            .await?;
        Ok(next_step)
    }

    async fn create_planning_step(
        &self,
        conversation: &Conversation,
    ) -> Result<ConversationStep, ConversationError> {
        let planning_prompt = self.build_planning_prompt(conversation).await?;

        let llm_response = timeout(
            self.step_timeout,
            self.llm_provider
                .generate_response(&planning_prompt, &conversation.context.available_tools),
        )
        .await
        .map_err(|_| ConversationError::ConversationTimeout)?
        .map_err(|e| ConversationError::LLMProviderError(e.to_string()))?;

        Ok(ConversationStep {
            step_id: 1,
            step_type: StepType::Planning,
            llm_request: Some(planning_prompt),
            llm_response: Some(llm_response.clone()),
            tool_calls: Vec::new(),
            results: Some(StepResult {
                data: serde_json::json!({ "plan": llm_response }),
                summary: Some("Query analysis and execution plan created".to_string()),
                next_action: Some("Execute planned tools".to_string()),
            }),
            status: StepStatus::Completed,
            created_at: Utc::now(),
        })
    }

    async fn execute_planning_step(
        &self,
        conversation: &mut Conversation,
    ) -> Result<ConversationStep, ConversationError> {
        let step_id = conversation.steps.len() as u32 + 1;

        // Parse the planning step to extract tool calls
        let planning_result = conversation
            .steps
            .iter()
            .find(|s| s.step_type == StepType::Planning)
            .and_then(|s| s.llm_response.as_ref())
            .ok_or_else(|| {
                ConversationError::PlanningFailed("No planning step found".to_string())
            })?;

        let tool_calls = self.parse_tool_calls_from_plan(planning_result)?;

        if tool_calls.is_empty() {
            conversation.state = ConversationState::Synthesizing;
            return self.execute_synthesis_step(conversation).await;
        }

        conversation.state = ConversationState::Executing;

        Ok(ConversationStep {
            step_id,
            step_type: StepType::ToolExecution,
            llm_request: None,
            llm_response: None,
            tool_calls: tool_calls.clone(),
            results: None,
            status: StepStatus::Pending,
            created_at: Utc::now(),
        })
    }

    async fn execute_tool_step(
        &self,
        conversation: &mut Conversation,
    ) -> Result<ConversationStep, ConversationError> {
        let step_id = conversation.steps.len() as u32 + 1;

        // Get the last pending tool execution step
        let current_step = conversation
            .steps
            .iter()
            .rev()
            .find(|s| s.step_type == StepType::ToolExecution && s.status == StepStatus::Pending)
            .cloned()
            .ok_or_else(|| {
                ConversationError::ToolExecutionError("No pending tool execution found".to_string())
            })?;

        // Execute all tool calls
        let mut executed_tools = Vec::new();
        for mut tool_call in current_step.tool_calls {
            let result = self.execute_single_tool(&tool_call).await;
            match result {
                Ok(tool_result) => {
                    tool_call.result = Some(tool_result);
                    executed_tools.push(tool_call);
                }
                Err(e) => {
                    tool_call.error = Some(e.to_string());
                    executed_tools.push(tool_call);
                }
            }
        }

        // Store results in context
        for tool in &executed_tools {
            if let Some(result) = &tool.result {
                conversation.context.execution_history.push(tool.clone());
                conversation
                    .context
                    .intermediate_results
                    .insert(format!("tool_result_{}", tool.tool_name), result.clone());
            }
        }

        // Check if we need more tool executions or can move to synthesis
        let has_errors = executed_tools.iter().any(|t| t.error.is_some());
        if has_errors || self.should_continue_execution(&executed_tools) {
            // Continue with more tool executions or handle errors
            conversation.state = ConversationState::Executing;
        } else {
            conversation.state = ConversationState::Synthesizing;
        }

        Ok(ConversationStep {
            step_id,
            step_type: StepType::ToolExecution,
            llm_request: None,
            llm_response: None,
            tool_calls: executed_tools.clone(),
            results: Some(StepResult {
                data: serde_json::json!({ "executed_tools": executed_tools.len() }),
                summary: Some(format!("Executed {} tools", executed_tools.len())),
                next_action: Some(if conversation.state == ConversationState::Synthesizing {
                    "Synthesize results".to_string()
                } else {
                    "Continue tool execution".to_string()
                }),
            }),
            status: StepStatus::Completed,
            created_at: Utc::now(),
        })
    }

    async fn execute_synthesis_step(
        &self,
        conversation: &mut Conversation,
    ) -> Result<ConversationStep, ConversationError> {
        let step_id = conversation.steps.len() as u32 + 1;

        let synthesis_prompt = self.build_synthesis_prompt(conversation).await?;

        let llm_response = timeout(
            self.step_timeout,
            self.llm_provider
                .generate_response(&synthesis_prompt, &conversation.context.available_tools),
        )
        .await
        .map_err(|_| ConversationError::ConversationTimeout)?
        .map_err(|e| ConversationError::LLMProviderError(e.to_string()))?;

        conversation.state = ConversationState::Completed;

        Ok(ConversationStep {
            step_id,
            step_type: StepType::ResultSynthesis,
            llm_request: Some(synthesis_prompt),
            llm_response: Some(llm_response.clone()),
            tool_calls: Vec::new(),
            results: Some(StepResult {
                data: serde_json::json!({ "final_answer": llm_response }),
                summary: Some("Final answer synthesized from tool results".to_string()),
                next_action: None,
            }),
            status: StepStatus::Completed,
            created_at: Utc::now(),
        })
    }

    async fn build_initial_context(&self) -> Result<ConversationContext, ConversationError> {
        let available_tools = self
            .tool_registry
            .list_tools()
            .await
            .map_err(|e| ConversationError::ToolExecutionError(e.to_string()))?
            .into_iter()
            .map(|tool| ToolDescriptor {
                name: tool.name,
                description: tool.description,
                parameters: serde_json::json!({}), // TODO: Get actual parameters
                return_type: "object".to_string(),
            })
            .collect();

        // TODO: Build actual data summary from database
        let data_summary = DataSummary {
            entity_count_by_type: HashMap::new(),
            relation_count_by_type: HashMap::new(),
            item_count_by_timeframe: HashMap::new(),
            data_freshness: Utc::now(),
        };

        Ok(ConversationContext {
            available_tools,
            data_summary,
            user_preferences: UserPreferences {
                max_conversation_steps: Some(self.max_steps),
                preferred_response_format: None,
                timeout_seconds: Some(self.step_timeout.as_secs()),
            },
            execution_history: Vec::new(),
            intermediate_results: HashMap::new(),
        })
    }

    async fn build_planning_prompt(
        &self,
        conversation: &Conversation,
    ) -> Result<String, ConversationError> {
        let tools_description = conversation
            .context
            .available_tools
            .iter()
            .map(|tool| format!("- {}: {}", tool.name, tool.description))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!(
            r#"You are an AI assistant that helps analyze Hacker News data. The user has asked: "{}"

Available tools:
{}

Please create a step-by-step plan to answer this query. Break down the task into specific tool calls that can be executed sequentially. Consider what data needs to be retrieved, filtered, and analyzed.

Return your plan as a structured response that includes specific tool calls with parameters."#,
            conversation.user_query, tools_description
        ))
    }

    #[allow(clippy::uninlined_format_args)]
    async fn build_synthesis_prompt(
        &self,
        conversation: &Conversation,
    ) -> Result<String, ConversationError> {
        let tool_results = conversation
            .context
            .intermediate_results
            .iter()
            .filter(|(k, _)| k.starts_with("tool_result_"))
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!(
            r#"Based on the following tool execution results, provide a comprehensive answer to the user's query: "{}"

Tool Results:
{}

Please synthesize these results into a clear, helpful response that directly answers the user's question. Include relevant data, insights, and any important findings."#,
            conversation.user_query, tool_results
        ))
    }

    fn parse_tool_calls_from_plan(
        &self,
        plan: &str,
    ) -> Result<Vec<ToolExecution>, ConversationError> {
        // Simple parsing - in a real implementation, this would be more sophisticated
        // For now, we'll create mock tool calls based on the plan content

        let mut tool_calls = Vec::new();

        if plan.to_lowercase().contains("search") || plan.to_lowercase().contains("find") {
            tool_calls.push(ToolExecution {
                tool_name: "search_entities".to_string(),
                parameters: serde_json::json!({
                    "query": "general search",
                    "limit": 10
                }),
                result: None,
                error: None,
                execution_time_ms: None,
            });
        }

        if plan.to_lowercase().contains("relation") {
            tool_calls.push(ToolExecution {
                tool_name: "analyze_relations".to_string(),
                parameters: serde_json::json!({
                    "entity_type": "company"
                }),
                result: None,
                error: None,
                execution_time_ms: None,
            });
        }

        if tool_calls.is_empty() {
            // Default to a general query tool
            tool_calls.push(ToolExecution {
                tool_name: "query_data".to_string(),
                parameters: serde_json::json!({
                    "query": plan
                }),
                result: None,
                error: None,
                execution_time_ms: None,
            });
        }

        Ok(tool_calls)
    }

    async fn execute_single_tool(
        &self,
        tool_call: &ToolExecution,
    ) -> Result<serde_json::Value, ConversationError> {
        let start_time = std::time::Instant::now();

        let result = timeout(
            self.step_timeout,
            self.tool_registry
                .execute_tool(&tool_call.tool_name, &tool_call.parameters),
        )
        .await
        .map_err(|_| ConversationError::ConversationTimeout)?
        .map_err(|e| ConversationError::ToolExecutionError(e.to_string()))?;

        let execution_time = start_time.elapsed().as_millis() as u64;

        // The result should include execution time, but we'll add it here for consistency
        Ok(serde_json::json!({
            "result": result,
            "execution_time_ms": execution_time
        }))
    }

    fn should_continue_execution(&self, executed_tools: &[ToolExecution]) -> bool {
        // Simple heuristic - in a real implementation, this would be more sophisticated
        // Continue if any tool suggests more data is needed
        executed_tools.iter().any(|tool| {
            tool.result
                .as_ref()
                .and_then(|r| r.get("continue"))
                .and_then(|c| c.as_bool())
                .unwrap_or(false)
        })
    }

    pub async fn get_conversation(
        &self,
        id: Uuid,
    ) -> Result<Option<Conversation>, ConversationError> {
        self.conversation_store.load_conversation(id).await
    }

    pub async fn list_conversations(
        &self,
        limit: Option<u32>,
    ) -> Result<Vec<Conversation>, ConversationError> {
        self.conversation_store.list_conversations(limit).await
    }

    pub async fn delete_conversation(&self, id: Uuid) -> Result<(), ConversationError> {
        self.conversation_store.delete_conversation(id).await
    }
}
