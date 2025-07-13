// pixlie/src/tools/mod.rs

#![allow(dead_code)]

pub mod data_query;
pub mod entity_analysis;
pub mod relation_exploration;
pub mod schemas;

#[cfg(test)]
mod tests;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;

/// Tool categories for organizing tools by functionality
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolCategory {
    DataQuery,
    EntityAnalysis,
    RelationExploration,
    Analytics,
}

/// Parameter types for tool arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
    Date,
}

/// Validation rules for tool parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationRule {
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub allowed_values: Option<Vec<String>>,
}

/// Tool parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: ParameterType,
    pub description: String,
    pub required: bool,
    pub default_value: Option<Value>,
    pub validation: Option<ValidationRule>,
}

/// Tool parameter schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameters {
    pub parameters: Vec<Parameter>,
    pub json_schema: Value,
}

/// Tool usage example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExample {
    pub description: String,
    pub input: Value,
    pub expected_output: Option<Value>,
    pub use_case: String,
}

/// Tool constraints and limitations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConstraints {
    pub max_execution_time_ms: Option<u64>,
    pub max_result_size: Option<usize>,
    pub rate_limit_per_minute: Option<u32>,
    pub requires_authentication: bool,
}

/// Comprehensive tool descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    pub category: ToolCategory,
    pub version: String,
    pub parameters: ToolParameters,
    pub examples: Vec<ToolExample>,
    pub constraints: ToolConstraints,
    pub tags: Vec<String>,
}

/// Tool execution arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolArguments {
    pub parameters: Value,
    pub context: Option<QueryContext>,
}

/// Query context for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub metadata: HashMap<String, Value>,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub data: Value,
    pub message: Option<String>,
    pub execution_time_ms: u64,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Validation error for tool parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub error_type: String,
    pub message: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

use jsonschema::JSONSchema;
/// Tool handler trait for executing tools
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Get the tool descriptor
    fn describe(&self) -> ToolDescriptor;

    /// Validate tool arguments before execution
    fn validate_args(&self, args: &ToolArguments) -> Result<(), Vec<ValidationError>> {
        let schema = self.describe().parameters.json_schema;
        let compiled_schema = JSONSchema::compile(&schema).expect("Invalid JSON schema");
        let result = compiled_schema.validate(&args.parameters);
        if let Err(errors) = result {
            let validation_errors = errors
                .map(|e| ValidationError {
                    field: e.instance_path.to_string(),
                    error_type: e.kind.to_string(),
                    message: e.to_string(),
                    expected: None,
                    actual: Some(e.instance.to_string()),
                })
                .collect();
            Err(validation_errors)
        } else {
            Ok(())
        }
    }

    /// Get tool performance metrics
    fn get_metrics(&self) -> ToolMetrics {
        ToolMetrics::default()
    }
}

/// Concrete tool implementations
#[derive(Debug, Clone)]
pub enum Tool {
    SearchItems(data_query::SearchItemsTool),
    FilterItems(data_query::FilterItemsTool),
    SearchEntities(entity_analysis::SearchEntitiesTool),
    ExploreRelations(relation_exploration::ExploreRelationsTool),
}

impl Tool {
    pub async fn execute(&self, args: ToolArguments) -> ToolResult {
        match self {
            Tool::SearchItems(tool) => tool.execute(args).await,
            Tool::FilterItems(tool) => tool.execute(args).await,
            Tool::SearchEntities(tool) => tool.execute(args).await,
            Tool::ExploreRelations(tool) => tool.execute(args).await,
        }
    }

    pub fn describe(&self) -> ToolDescriptor {
        match self {
            Tool::SearchItems(tool) => tool.describe(),
            Tool::FilterItems(tool) => tool.describe(),
            Tool::SearchEntities(tool) => tool.describe(),
            Tool::ExploreRelations(tool) => tool.describe(),
        }
    }

    pub fn validate_args(&self, args: &ToolArguments) -> Result<(), Vec<ValidationError>> {
        match self {
            Tool::SearchItems(tool) => tool.validate_args(args),
            Tool::FilterItems(tool) => tool.validate_args(args),
            Tool::SearchEntities(tool) => tool.validate_args(args),
            Tool::ExploreRelations(tool) => tool.validate_args(args),
        }
    }

    pub fn name(&self) -> String {
        self.describe().name
    }
}

/// Tool performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolMetrics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time_ms: f64,
    pub last_execution: Option<String>,
}

/// Tool registry for managing available tools
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
    metrics: HashMap<String, ToolMetrics>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            metrics: HashMap::new(),
        }
    }

    /// Register a new tool
    pub fn register(&mut self, tool: Tool) {
        let descriptor = tool.describe();
        self.tools.insert(descriptor.name.clone(), tool);
        self.metrics.insert(descriptor.name, ToolMetrics::default());
    }

    /// Get a tool by name
    pub fn get_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    /// Get a cloned tool by name to avoid lifetime issues
    pub fn get_tool_cloned(&self, name: &str) -> Option<Tool> {
        self.tools.get(name).cloned()
    }

    /// Get all tool descriptors
    pub fn get_all_descriptors(&self) -> Vec<ToolDescriptor> {
        self.tools.values().map(|tool| tool.describe()).collect()
    }

    /// Get tools by category
    pub fn get_tools_by_category(&self, category: ToolCategory) -> Vec<ToolDescriptor> {
        self.tools
            .values()
            .map(|tool| tool.describe())
            .filter(|desc| desc.category == category)
            .collect()
    }

    /// Execute a tool by name
    pub async fn execute_tool(&mut self, name: &str, args: ToolArguments) -> Option<ToolResult> {
        if let Some(tool) = self.tools.get(name).cloned() {
            let start_time = Instant::now();
            let result = tool.execute(args).await;
            let execution_time = start_time.elapsed().as_millis() as u64;

            // Update metrics
            if let Some(metrics) = self.metrics.get_mut(name) {
                metrics.total_executions += 1;
                if result.success {
                    metrics.successful_executions += 1;
                } else {
                    metrics.failed_executions += 1;
                }

                // Update average execution time
                let total_time =
                    metrics.average_execution_time_ms * (metrics.total_executions - 1) as f64;
                metrics.average_execution_time_ms =
                    (total_time + execution_time as f64) / metrics.total_executions as f64;
                metrics.last_execution = Some(chrono::Utc::now().to_rfc3339());
            }

            Some(result)
        } else {
            None
        }
    }

    /// Get tool metrics
    pub fn get_tool_metrics(&self, name: &str) -> Option<&ToolMetrics> {
        self.metrics.get(name)
    }

    /// Get all tool metrics
    pub fn get_all_metrics(&self) -> &HashMap<String, ToolMetrics> {
        &self.metrics
    }
}

/// Helper function to create JSON schema from parameters
pub fn create_json_schema(parameters: &[Parameter]) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for param in parameters {
        if param.required {
            required.push(param.name.clone());
        }

        let mut param_schema = serde_json::Map::new();
        param_schema.insert(
            "type".to_string(),
            Value::String(
                match param.param_type {
                    ParameterType::String => "string",
                    ParameterType::Integer => "integer",
                    ParameterType::Float => "number",
                    ParameterType::Boolean => "boolean",
                    ParameterType::Array => "array",
                    ParameterType::Object => "object",
                    ParameterType::Date => "string",
                }
                .to_string(),
            ),
        );

        param_schema.insert(
            "description".to_string(),
            Value::String(param.description.clone()),
        );

        if let Some(default) = &param.default_value {
            param_schema.insert("default".to_string(), default.clone());
        }

        if let Some(validation) = &param.validation {
            if let Some(min) = validation.min_value {
                param_schema.insert(
                    "minimum".to_string(),
                    Value::Number(serde_json::Number::from_f64(min).unwrap()),
                );
            }
            if let Some(max) = validation.max_value {
                param_schema.insert(
                    "maximum".to_string(),
                    Value::Number(serde_json::Number::from_f64(max).unwrap()),
                );
            }
            if let Some(min_len) = validation.min_length {
                param_schema.insert(
                    "minLength".to_string(),
                    Value::Number(serde_json::Number::from(min_len)),
                );
            }
            if let Some(max_len) = validation.max_length {
                param_schema.insert(
                    "maxLength".to_string(),
                    Value::Number(serde_json::Number::from(max_len)),
                );
            }
            if let Some(pattern) = &validation.pattern {
                param_schema.insert("pattern".to_string(), Value::String(pattern.clone()));
            }
            if let Some(allowed) = &validation.allowed_values {
                param_schema.insert(
                    "enum".to_string(),
                    Value::Array(allowed.iter().map(|v| Value::String(v.clone())).collect()),
                );
            }
        }

        properties.insert(param.name.clone(), Value::Object(param_schema));
    }

    serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}
