// pixlie/src/tools/types.rs

// Re-declare simplified types for TypeScript generation
// These avoid complex dependencies on database-specific types
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Simplified HN Item for API responses
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct HnItem {
    pub id: i64,
    pub title: Option<String>,
    pub by: Option<String>,
    pub score: Option<i32>,
    pub item_type: String,
    #[ts(type = "string")]
    pub time: DateTime<Utc>,
    pub text: Option<String>,
    pub url: Option<String>,
    pub descendants: Option<i32>,
}

/// Simplified Entity for API responses
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct Entity {
    pub id: i64,
    pub entity_value: String,
    pub entity_type: String,
    pub confidence: f64,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
}

/// Simplified Entity Relation for API responses
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct EntityRelation {
    pub id: i64,
    pub subject_entity_id: i64,
    pub object_entity_id: i64,
    pub relation_type: String,
    pub confidence: f64,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
}

/// Time range filter for queries
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct TimeRange {
    #[ts(type = "string")]
    pub start: DateTime<Utc>,
    #[ts(type = "string")]
    pub end: DateTime<Utc>,
}

/// Score range filter for items
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct ScoreRange {
    pub min: i32,
    pub max: i32,
}

// === Data Query Tool Types ===

/// Parameters for searching HN items
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct SearchItemsParams {
    /// Search keywords or phrases to find in item titles and text
    pub query: String,
    /// Filter by specific author username
    pub author: Option<String>,
    /// Filter by item type (story, comment, job, poll)
    pub item_type: Option<String>,
    /// Minimum score threshold for items
    pub min_score: Option<i32>,
    /// Date range for filtering items
    pub time_range: Option<TimeRange>,
    /// Maximum number of results to return (1-1000)
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// Response from searching HN items
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct SearchItemsResponse {
    pub items: Vec<HnItem>,
    pub total_count: u64,
    pub query_time_ms: u64,
    pub filters_applied: SearchItemsFilters,
}

/// Filters that were applied to the search
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct SearchItemsFilters {
    pub query: String,
    pub author: Option<String>,
    pub item_type: Option<String>,
    pub min_score: Option<i32>,
    pub time_range: Option<TimeRange>,
    pub limit: u32,
}

/// Parameters for filtering HN items
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct FilterItemsParams {
    /// Filter by score range (min and max)
    pub score_range: Option<ScoreRange>,
    /// Filter by time range
    pub time_range: Option<TimeRange>,
    /// Filter by comment count range
    pub comment_count_range: Option<ScoreRange>,
    /// Filter by specific authors
    pub authors: Option<Vec<String>>,
    /// Maximum number of results to return
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// Response from filtering HN items
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct FilterItemsResponse {
    pub items: Vec<HnItem>,
    pub total_count: u64,
    pub filters_applied: FilterItemsParams,
    pub query_time_ms: u64,
}

// === Entity Analysis Tool Types ===

/// Parameters for searching entities
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct SearchEntitiesParams {
    /// Entity name or partial name to search for
    pub query: Option<String>,
    /// Filter by entity type (person, company, technology, etc.)
    pub entity_type: Option<String>,
    /// Minimum confidence threshold (0.0-1.0)
    pub min_confidence: Option<f64>,
    /// Minimum number of mentions
    pub min_mentions: Option<u32>,
    /// Maximum number of results to return
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// Response from searching entities
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct SearchEntitiesResponse {
    pub entities: Vec<EntityWithStats>,
    pub total_count: u64,
    pub query_applied: SearchEntitiesParams,
    pub query_time_ms: u64,
}

/// Entity with additional statistics
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct EntityWithStats {
    pub entity: Entity,
    pub mentions_count: u32,
    #[ts(type = "string | null")]
    pub first_mentioned: Option<DateTime<Utc>>,
    #[ts(type = "string | null")]
    pub last_mentioned: Option<DateTime<Utc>>,
    pub related_entities_count: u32,
}

// === Relation Exploration Tool Types ===

/// Parameters for exploring entity relationships
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct ExploreRelationsParams {
    /// Filter by specific relation type (founded, acquired, works_at, etc.)
    pub relation_type: Option<String>,
    /// Focus on relations for specific entity ID
    pub entity_id: Option<i64>,
    /// Focus on relations involving specific entity name
    pub entity_name: Option<String>,
    /// Minimum confidence threshold (0.0-1.0)
    pub min_confidence: Option<f64>,
    /// Maximum number of results to return
    #[serde(default = "default_limit")]
    pub limit: u32,
}

/// Response from exploring entity relationships
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct ExploreRelationsResponse {
    pub relations: Vec<RelationWithEntities>,
    pub total_count: u64,
    pub query_applied: ExploreRelationsParams,
    pub query_time_ms: u64,
}

/// Entity relation with full entity details
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct RelationWithEntities {
    pub relation: EntityRelation,
    pub subject_entity: Entity,
    pub object_entity: Entity,
    pub source_item_count: u32,
}

/// Validation result for tool parameters
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

/// Validation warning for tool parameters
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct ValidationWarning {
    pub field: String,
    pub warning_type: String,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Tool schema information
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct ToolSchema {
    pub name: String,
    #[ts(type = "Record<string, unknown>")]
    pub parameter_schema: serde_json::Value,
    #[ts(type = "Record<string, unknown>")]
    pub response_schema: serde_json::Value,
    pub examples: Vec<ToolSchemaExample>,
}

/// Tool schema example
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
pub struct ToolSchemaExample {
    pub name: String,
    pub description: String,
    #[ts(type = "Record<string, unknown>")]
    pub parameters: serde_json::Value,
    #[ts(type = "Record<string, unknown>")]
    pub expected_response: serde_json::Value,
}

// Re-export ValidationError from parent module
use super::ValidationError;

/// Default limit for search results
fn default_limit() -> u32 {
    100
}
