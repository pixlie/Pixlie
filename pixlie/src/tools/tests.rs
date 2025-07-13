// pixlie/src/tools/tests.rs

#[cfg(test)]
mod tests {
    use crate::tools::{
        Parameter, ParameterType, ToolArguments, ToolCategory, ToolRegistry, ValidationRule,
        create_json_schema, ToolHandler, data_query::SearchItemsTool,
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
        registry.register(Box::new(SearchItemsTool::new()));

        // Check that tool is registered
        let tool = registry.get_tool("search_items");
        assert!(tool.is_some());

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
}
