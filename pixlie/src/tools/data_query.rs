use crate::database::HnItem;
use crate::tools::schemas::{JsonSchema, ToolParameterSchema, ToolResponseSchema};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ts_rs::TS;

use super::{
    Tool, ToolArguments, ToolCategory, ToolConstraints, ToolDescriptor, ToolExample, ToolHandler,
    ToolParameters, ToolResult, ValidationError,
};

// Define the parameters for the SearchItemsTool
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "../webapp/src/types/tools/SearchItemsParams.ts")]
pub struct SearchItemsParams {
    #[schemars(description = "Search keywords or phrases to find in item titles and text")]
    pub query: String,
    #[schemars(description = "Filter by specific author username")]
    pub author: Option<String>,
    #[schemars(description = "Filter by item type (e.g., 'story', 'comment')")]
    pub item_type: Option<String>,
    #[schemars(description = "Minimum score threshold for items")]
    pub min_score: Option<i32>,
    #[schemars(description = "Maximum number of results to return (1-1000)", range(min = 1, max = 1000))]
    pub limit: Option<u32>,
}

impl ToolParameterSchema for SearchItemsParams {}

// Define the response for the SearchItemsTool
#[derive(Debug, Clone, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "../webapp/src/types/tools/SearchItemsResponse.ts")]
pub struct SearchItemsResponse {
    pub items: Vec<HnItem>,
    pub total_count: u64,
    pub query_time_ms: u64,
}

impl ToolResponseSchema for SearchItemsResponse {}

/// Search HN items by keywords, author, and time range
#[derive(Debug, Clone)]
pub struct SearchItemsTool;

impl Default for SearchItemsTool {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchItemsTool {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn execute(&self, args: ToolArguments) -> ToolResult {
        let start_time = std::time::Instant::now();

        let params: SearchItemsParams = match serde_json::from_value(args.parameters.clone()) {
            Ok(params) => params,
            Err(e) => {
                return ToolResult {
                    success: false,
                    data: json!(null),
                    message: Some(format!("Invalid parameters: {e}")),
                    execution_time_ms: 0,
                    errors: vec![e.to_string()],
                    warnings: vec![],
                };
            }
        };

        // TODO: Implement actual database search
        let mock_items = vec![
            HnItem {
                id: 1,
                by: Some(params.author.clone().unwrap_or_else(|| "testuser".to_string())),
                title: Some(format!("Test story for '{}'", params.query)),
                text: None,
                item_type: params.item_type.clone().unwrap_or_else(|| "story".to_string()),
                ..Default::default()
            },
        ];

        let response = SearchItemsResponse {
            items: mock_items,
            total_count: 1,
            query_time_ms: start_time.elapsed().as_millis() as u64,
        };

        ToolResult {
            success: true,
            data: serde_json::to_value(response).unwrap(),
            message: Some("Search completed successfully".to_string()),
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            errors: vec![],
            warnings: vec![],
        }
    }
}

#[async_trait]
impl ToolHandler for SearchItemsTool {
    fn describe(&self) -> ToolDescriptor {
        ToolDescriptor {
            name: "search_items".to_string(),
            description: "Search Hacker News items by keywords, author, item type, and other filters"
                .to_string(),
            category: ToolCategory::DataQuery,
            version: "1.0.0".to_string(),
            parameters: ToolParameters {
                parameters: vec![],
                json_schema: serde_json::to_value(schemars::schema_for!(SearchItemsParams)).unwrap(),
            },
            examples: vec![
                ToolExample {
                    description: "Search for AI-related discussions".to_string(),
                    input: json!({
                        "query": "artificial intelligence",
                        "min_score": 10,
                        "limit": 50
                    }),
                    expected_output: None,
                    use_case: "Finding trending AI discussions with good engagement".to_string(),
                },
            ],
            constraints: ToolConstraints {
                max_execution_time_ms: Some(5000),
                max_result_size: Some(1000),
                rate_limit_per_minute: Some(60),
                requires_authentication: false,
            },
            tags: vec!["search".to_string(), "items".to_string(), "hacker-news".to_string()],
        }
    }

}

// ... (keep FilterItemsTool as is for now)
/// Filter HN items by various criteria
#[derive(Debug, Clone)]
pub struct FilterItemsTool;

impl Default for FilterItemsTool {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterItemsTool {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, args: ToolArguments) -> ToolResult {
        let start_time = std::time::Instant::now();

        // TODO: Implement actual filtering logic
        let mock_results = json!({
            "items": [],
            "total_count": 0,
            "filters_applied": args.parameters,
            "message": "Filtering functionality not yet implemented"
        });

        ToolResult {
            success: true,
            data: mock_results,
            message: Some("Filter operation completed".to_string()),
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            errors: vec![],
            warnings: vec!["Filtering functionality is not yet fully implemented".to_string()],
        }
    }
}

#[async_trait]
impl ToolHandler for FilterItemsTool {
    fn describe(&self) -> ToolDescriptor {
        ToolDescriptor {
            name: "filter_items".to_string(),
            description: "Filter HN items by score, time range, comment count, and other criteria"
                .to_string(),
            category: ToolCategory::DataQuery,
            version: "1.0.0".to_string(),
            parameters: ToolParameters {
                parameters: vec![], // Will be replaced by JSON schema
                json_schema: json!({
                    "type": "object",
                    "properties": {
                        "score_range": {
                            "type": "object",
                            "properties": {
                                "min": {"type": "integer"},
                                "max": {"type": "integer"}
                            }
                        },
                        "time_range": {
                            "type": "object",
                            "properties": {
                                "start": {"type": "string", "format": "date-time"},
                                "end": {"type": "string", "format": "date-time"}
                            }
                        }
                    }
                }),
            },
            examples: vec![],
            constraints: ToolConstraints {
                max_execution_time_ms: Some(3000),
                max_result_size: Some(1000),
                rate_limit_per_minute: Some(100),
                requires_authentication: false,
            },
            tags: vec!["filter".to_string(), "items".to_string()],
        }
    }

    fn validate_args(&self, _args: &ToolArguments) -> Result<(), Vec<ValidationError>> {
        // TODO: Implement validation
        Ok(())
    }
}
