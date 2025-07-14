use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;
use uuid::Uuid;

pub mod context;
pub mod executor;
pub mod manager;
pub mod planner;
pub mod storage;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Conversation {
    #[ts(type = "string")]
    pub id: Uuid,
    pub user_query: String,
    pub state: ConversationState,
    pub steps: Vec<ConversationStep>,
    pub context: ConversationContext,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
    #[ts(type = "string")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum ConversationState {
    Planning,
    Executing,
    Synthesizing,
    Completed,
    Failed,
    RequiresUserInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConversationStep {
    pub step_id: u32,
    pub step_type: StepType,
    pub llm_request: Option<String>,
    pub llm_response: Option<String>,
    pub tool_calls: Vec<ToolExecution>,
    pub results: Option<StepResult>,
    pub status: StepStatus,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum StepType {
    Planning,
    ToolExecution,
    ResultSynthesis,
    UserClarification,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ToolExecution {
    pub tool_name: String,
    #[ts(type = "unknown")]
    pub parameters: serde_json::Value,
    #[ts(type = "unknown | null")]
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StepResult {
    #[ts(type = "unknown")]
    pub data: serde_json::Value,
    pub summary: Option<String>,
    pub next_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConversationContext {
    pub available_tools: Vec<ToolDescriptor>,
    pub data_summary: DataSummary,
    pub user_preferences: UserPreferences,
    pub execution_history: Vec<ToolExecution>,
    #[ts(type = "Record<string, unknown>")]
    pub intermediate_results: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    #[ts(type = "unknown")]
    pub parameters: serde_json::Value,
    pub return_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DataSummary {
    pub entity_count_by_type: HashMap<String, u64>,
    pub relation_count_by_type: HashMap<String, u64>,
    pub item_count_by_timeframe: HashMap<String, u64>,
    #[ts(type = "string")]
    pub data_freshness: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UserPreferences {
    pub max_conversation_steps: Option<u32>,
    pub preferred_response_format: Option<String>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConversationResult {
    pub summary: String,
    #[ts(type = "unknown")]
    pub data: serde_json::Value,
    pub confidence: f64,
    pub sources: Vec<String>,
}

#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum ConversationError {
    #[error("LLM provider error: {0}")]
    LLMProviderError(String),

    #[error("Tool execution error: {0}")]
    ToolExecutionError(String),

    #[error("Invalid tool call: {0}")]
    InvalidToolCall(String),

    #[error("Conversation timeout")]
    ConversationTimeout,

    #[error("Context too large")]
    ContextTooLarge,

    #[error("Planning failed: {0}")]
    PlanningFailed(String),

    #[error("User intervention required: {0}")]
    UserInterventionRequired(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<serde_json::Error> for ConversationError {
    fn from(err: serde_json::Error) -> Self {
        ConversationError::SerializationError(err.to_string())
    }
}

impl From<sqlx::Error> for ConversationError {
    fn from(err: sqlx::Error) -> Self {
        ConversationError::StorageError(err.to_string())
    }
}
