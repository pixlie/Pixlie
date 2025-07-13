// pixlie/src/tools/tests.rs

#[cfg(test)]
mod tool_tests {
    use crate::tools::{
        Parameter, ParameterType, Tool, ToolArguments, ToolCategory, ToolHandler, ToolRegistry,
        ValidationRule, create_json_schema, data_query::SearchItemsTool,
    };
    use serde_json::json;

    #[tokio::test]
    async fn test_search_items_tool_descriptor() {
        let tool = SearchItemsTool::new();
        let descriptor = tool.describe();

        assert_eq!(descriptor.name, "search_items");
        assert_eq!(descriptor.category, ToolCategory::DataQuery);
        assert!(!descriptor.parameters.parameters.is_empty());
    }

    #[tokio::test]
    async fn test_search_items_tool_validation() {
        let tool = SearchItemsTool::new();

        // Test valid arguments
        let valid_args = ToolArguments {
            parameters: json!({
                "query": "test search",
                "limit": 50
            }),
            context: None,
        };

        assert!(tool.validate_args(&valid_args).is_ok());

        // Test invalid arguments (missing query)
        let invalid_args = ToolArguments {
            parameters: json!({
                "limit": 50
            }),
            context: None,
        };

        assert!(tool.validate_args(&invalid_args).is_err());
    }

    #[tokio::test]
    async fn test_search_items_tool_execution() {
        let tool = SearchItemsTool::new();

        let args = ToolArguments {
            parameters: json!({
                "query": "test search",
                "limit": 10
            }),
            context: None,
        };

        let result = tool.execute(args).await;
        assert!(result.success);
        assert!(result.data.is_object());
    }

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();

        // Register a tool
        registry.register(Tool::SearchItems(SearchItemsTool::new()));

        // Check that tool is registered
        let tool = registry.get_tool("search_items");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "search_items".to_string());

        // Check descriptors
        let descriptors = registry.get_all_descriptors();
        assert_eq!(descriptors.len(), 1);
        assert_eq!(descriptors[0].name, "search_items");
    }

    #[test]
    fn test_json_schema_generation() {
        let parameters = vec![
            Parameter {
                name: "test_string".to_string(),
                param_type: ParameterType::String,
                description: "Test string parameter".to_string(),
                required: true,
                default_value: None,
                validation: None,
            },
            Parameter {
                name: "test_number".to_string(),
                param_type: ParameterType::Integer,
                description: "Test number parameter".to_string(),
                required: false,
                default_value: Some(json!(42)),
                validation: Some(ValidationRule {
                    min_value: Some(0.0),
                    max_value: Some(100.0),
                    ..Default::default()
                }),
            },
        ];

        let schema = create_json_schema(&parameters);

        assert!(schema.is_object());
        assert!(schema["properties"].is_object());
        assert!(schema["required"].is_array());
        assert_eq!(schema["required"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_typescript_type_generation() {
        use crate::tools::types::*;

        // Test SearchItemsParams type generation
        let params = SearchItemsParams {
            query: "AI startup".to_string(),
            author: Some("pg".to_string()),
            item_type: Some("story".to_string()),
            min_score: Some(10),
            time_range: None,
            limit: 50,
        };

        let serialized = serde_json::to_value(params).unwrap();
        assert_eq!(serialized["query"], "AI startup");
        assert_eq!(serialized["author"], "pg");
        assert_eq!(serialized["limit"], 50);
    }

    #[test]
    fn test_validation_result_creation() {
        use crate::tools::types::*;

        let validation_result = ValidationResult {
            is_valid: true,
            errors: vec![],
            warnings: vec![ValidationWarning {
                field: "limit".to_string(),
                warning_type: "high_value".to_string(),
                message: "Limit is very high".to_string(),
                suggestion: Some("Consider reducing limit for better performance".to_string()),
            }],
        };

        assert!(validation_result.is_valid);
        assert_eq!(validation_result.errors.len(), 0);
        assert_eq!(validation_result.warnings.len(), 1);
    }

    #[test]
    fn test_tool_validator_interface() {
        use crate::tools::ToolValidator;

        let tool = SearchItemsTool::new();

        // Valid parameters
        let valid_params = json!({
            "query": "artificial intelligence",
            "limit": 100
        });

        let result = tool.validate_parameters(&valid_params);
        assert!(result.is_valid);
        assert_eq!(result.errors.len(), 0);

        // Invalid parameters - missing required query
        let invalid_params = json!({
            "limit": 100
        });

        let result = tool.validate_parameters(&invalid_params);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_json_schema_generation_for_types() {
        use crate::tools::{generate_json_schema, types::*};

        let schema = generate_json_schema::<SearchItemsParams>();

        // Verify schema structure
        assert!(schema.is_object());
        let obj = schema.as_object().unwrap();
        assert!(obj.contains_key("type"));
        assert!(obj.contains_key("properties"));

        let properties = obj["properties"].as_object().unwrap();
        assert!(properties.contains_key("query"));
        assert!(properties.contains_key("limit"));
        assert!(properties.contains_key("author"));
    }

    #[test]
    fn test_time_range_serialization() {
        use crate::tools::types::*;
        use chrono::{DateTime, Utc};

        let time_range = TimeRange {
            start: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            end: DateTime::parse_from_rfc3339("2024-01-31T23:59:59Z")
                .unwrap()
                .with_timezone(&Utc),
        };

        let serialized = serde_json::to_value(time_range).unwrap();
        assert!(serialized["start"].is_string());
        assert!(serialized["end"].is_string());
    }

    #[test]
    fn test_entity_with_stats_creation() {
        use crate::tools::types::*;
        use chrono::Utc;

        let entity = Entity {
            id: 1,
            entity_value: "OpenAI".to_string(),
            entity_type: "company".to_string(),
            confidence: 0.95,
            created_at: Utc::now(),
        };

        let entity_with_stats = EntityWithStats {
            entity,
            mentions_count: 42,
            first_mentioned: Some(Utc::now()),
            last_mentioned: Some(Utc::now()),
            related_entities_count: 15,
        };

        let serialized = serde_json::to_value(entity_with_stats).unwrap();
        assert_eq!(serialized["mentions_count"], 42);
        assert_eq!(serialized["related_entities_count"], 15);
    }

    #[test]
    fn test_tool_schema_creation() {
        use crate::tools::types::*;

        let schema = ToolSchema {
            name: "test_tool".to_string(),
            parameter_schema: json!({"type": "object"}),
            response_schema: json!({"type": "object"}),
            examples: vec![ToolSchemaExample {
                name: "Basic example".to_string(),
                description: "A simple test".to_string(),
                parameters: json!({"query": "test"}),
                expected_response: json!({"success": true}),
            }],
        };

        let serialized = serde_json::to_value(schema).unwrap();
        assert_eq!(serialized["name"], "test_tool");
        assert_eq!(serialized["examples"].as_array().unwrap().len(), 1);
    }
}
