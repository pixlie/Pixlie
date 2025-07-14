use super::*;
use crate::database::Database;
use sqlx::Row;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct ContextManager {
    db: Database,
    max_context_size: usize,
    max_history_items: usize,
}

#[allow(dead_code)]
impl ContextManager {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            max_context_size: 1024 * 1024, // 1MB
            max_history_items: 100,
        }
    }

    pub async fn build_conversation_context(
        &self,
        available_tools: Vec<ToolDescriptor>,
    ) -> Result<ConversationContext, ConversationError> {
        let data_summary = self.build_data_summary().await?;

        Ok(ConversationContext {
            available_tools,
            data_summary,
            user_preferences: UserPreferences {
                max_conversation_steps: Some(20),
                preferred_response_format: None,
                timeout_seconds: Some(60),
            },
            execution_history: Vec::new(),
            intermediate_results: HashMap::new(),
        })
    }

    pub async fn update_context_with_execution(
        &self,
        context: &mut ConversationContext,
        execution: &ToolExecution,
    ) -> Result<(), ConversationError> {
        // Add to execution history
        context.execution_history.push(execution.clone());

        // Limit history size
        if context.execution_history.len() > self.max_history_items {
            context.execution_history.remove(0);
        }

        // Store intermediate results
        if let Some(result) = &execution.result {
            let result_key = format!(
                "{}_{}",
                execution.tool_name,
                context.execution_history.len()
            );
            context
                .intermediate_results
                .insert(result_key, result.clone());
        }

        // Compress context if it's getting too large
        self.compress_context_if_needed(context).await?;

        Ok(())
    }

    pub async fn compress_context_if_needed(
        &self,
        context: &mut ConversationContext,
    ) -> Result<(), ConversationError> {
        let context_size = self.estimate_context_size(context)?;

        if context_size > self.max_context_size {
            self.compress_context(context).await?;
        }

        Ok(())
    }

    async fn compress_context(
        &self,
        context: &mut ConversationContext,
    ) -> Result<(), ConversationError> {
        // Remove older execution history items
        let keep_count = self.max_history_items / 2;
        if context.execution_history.len() > keep_count {
            context
                .execution_history
                .drain(0..context.execution_history.len() - keep_count);
        }

        // Compress intermediate results by keeping only the most recent ones
        let sorted_keys: Vec<_> = context.intermediate_results.keys().cloned().collect();
        let keep_count = 20; // Keep only the most recent 20 intermediate results

        if sorted_keys.len() > keep_count {
            let remove_count = sorted_keys.len() - keep_count;
            for key in sorted_keys.into_iter().take(remove_count) {
                context.intermediate_results.remove(&key);
            }
        }

        // Summarize large result objects
        for (_, value) in context.intermediate_results.iter_mut() {
            if let Some(obj) = value.as_object_mut() {
                if obj.len() > 10 {
                    // Summarize large objects
                    let summary = self.summarize_large_object(obj)?;
                    *value = summary;
                }
            }
        }

        Ok(())
    }

    fn summarize_large_object(
        &self,
        obj: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<serde_json::Value, ConversationError> {
        let mut summary = serde_json::Map::new();

        // Keep basic metadata
        for key in ["count", "total", "length", "size", "type"] {
            if let Some(value) = obj.get(key) {
                summary.insert(key.to_string(), value.clone());
            }
        }

        // Add summary information
        summary.insert(
            "_summary".to_string(),
            serde_json::json!({
                "original_keys": obj.keys().take(5).collect::<Vec<_>>(),
                "total_keys": obj.len(),
                "compressed": true
            }),
        );

        Ok(serde_json::Value::Object(summary))
    }

    fn estimate_context_size(
        &self,
        context: &ConversationContext,
    ) -> Result<usize, ConversationError> {
        let serialized = serde_json::to_string(context)?;
        Ok(serialized.len())
    }

    async fn build_data_summary(&self) -> Result<DataSummary, ConversationError> {
        let conn = self
            .db
            .get_connection()
            .await
            .map_err(|e| ConversationError::StorageError(e.to_string()))?;

        // Get entity counts by type
        let entity_rows =
            sqlx::query("SELECT entity_type, COUNT(*) as count FROM entities GROUP BY entity_type")
                .fetch_all(&conn)
                .await
                .map_err(|e| ConversationError::StorageError(e.to_string()))?;

        let mut entity_count_by_type = HashMap::new();
        for row in entity_rows {
            let entity_type: String = row.get("entity_type");
            let count: i64 = row.get("count");
            entity_count_by_type.insert(entity_type, count as u64);
        }

        // Get relation counts by type
        let relation_rows = sqlx::query(
            "SELECT relation_type, COUNT(*) as count FROM entity_relations GROUP BY relation_type",
        )
        .fetch_all(&conn)
        .await
        .map_err(|e| ConversationError::StorageError(e.to_string()))?;

        let mut relation_count_by_type = HashMap::new();
        for row in relation_rows {
            let relation_type: String = row.get("relation_type");
            let count: i64 = row.get("count");
            relation_count_by_type.insert(relation_type, count as u64);
        }

        // Get item counts by timeframe (simplified - just total for now)
        let item_count_row = sqlx::query("SELECT COUNT(*) as count FROM hn_items")
            .fetch_one(&conn)
            .await
            .map_err(|e| ConversationError::StorageError(e.to_string()))?;

        let total_items: i64 = item_count_row.get("count");
        let mut item_count_by_timeframe = HashMap::new();
        item_count_by_timeframe.insert("total".to_string(), total_items as u64);

        // Get data freshness from the most recent item
        let freshness_row = sqlx::query("SELECT MAX(created_at) as latest FROM hn_items")
            .fetch_optional(&conn)
            .await
            .map_err(|e| ConversationError::StorageError(e.to_string()))?;

        let data_freshness = if let Some(row) = freshness_row {
            row.get::<Option<DateTime<Utc>>, _>("latest")
                .unwrap_or_else(Utc::now)
        } else {
            Utc::now()
        };

        Ok(DataSummary {
            entity_count_by_type,
            relation_count_by_type,
            item_count_by_timeframe,
            data_freshness,
        })
    }

    pub fn add_user_preference(
        &self,
        context: &mut ConversationContext,
        preference_key: &str,
        preference_value: &str,
    ) {
        match preference_key {
            "response_format" => {
                context.user_preferences.preferred_response_format =
                    Some(preference_value.to_string());
            }
            "timeout" => {
                if let Ok(timeout) = preference_value.parse::<u64>() {
                    context.user_preferences.timeout_seconds = Some(timeout);
                }
            }
            "max_steps" => {
                if let Ok(max_steps) = preference_value.parse::<u32>() {
                    context.user_preferences.max_conversation_steps = Some(max_steps);
                }
            }
            _ => {
                // Store as intermediate result for custom preferences
                context.intermediate_results.insert(
                    format!("preference_{preference_key}"),
                    serde_json::Value::String(preference_value.to_string()),
                );
            }
        }
    }

    pub fn get_relevant_history<'a>(
        &self,
        context: &'a ConversationContext,
        tool_name: &str,
        limit: usize,
    ) -> Vec<&'a ToolExecution> {
        context
            .execution_history
            .iter()
            .rev()
            .filter(|execution| execution.tool_name == tool_name)
            .take(limit)
            .collect()
    }

    pub fn calculate_relevance_score(
        &self,
        context: &ConversationContext,
        query: &str,
    ) -> HashMap<String, f64> {
        let mut scores = HashMap::new();
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        // Score execution history based on query relevance
        for execution in &context.execution_history {
            let mut score = 0.0;

            // Tool name relevance
            for word in &query_words {
                if execution.tool_name.to_lowercase().contains(word) {
                    score += 1.0;
                }
            }

            // Parameter relevance
            if let Ok(params_str) = serde_json::to_string(&execution.parameters) {
                let params_lower = params_str.to_lowercase();
                for word in &query_words {
                    if params_lower.contains(word) {
                        score += 0.5;
                    }
                }
            }

            // Result relevance
            if let Some(result) = &execution.result {
                if let Ok(result_str) = serde_json::to_string(result) {
                    let result_lower = result_str.to_lowercase();
                    for word in &query_words {
                        if result_lower.contains(word) {
                            score += 0.3;
                        }
                    }
                }
            }

            let key = format!(
                "{}_{}",
                execution.tool_name,
                context
                    .execution_history
                    .iter()
                    .position(|e| std::ptr::eq(e, execution))
                    .unwrap_or(0)
            );
            scores.insert(key, score);
        }

        scores
    }

    pub fn get_context_statistics(&self, context: &ConversationContext) -> ContextStatistics {
        let execution_count = context.execution_history.len();
        let successful_executions = context
            .execution_history
            .iter()
            .filter(|e| e.error.is_none())
            .count();

        let total_execution_time: u64 = context
            .execution_history
            .iter()
            .filter_map(|e| e.execution_time_ms)
            .sum();

        let intermediate_results_count = context.intermediate_results.len();
        let context_size = self.estimate_context_size(context).unwrap_or(0);

        ContextStatistics {
            execution_count,
            successful_executions,
            failed_executions: execution_count - successful_executions,
            total_execution_time_ms: total_execution_time,
            intermediate_results_count,
            context_size_bytes: context_size,
            available_tools_count: context.available_tools.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStatistics {
    pub execution_count: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub total_execution_time_ms: u64,
    pub intermediate_results_count: usize,
    pub context_size_bytes: usize,
    pub available_tools_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    #[tokio::test]
    #[allow(clippy::uninlined_format_args)]
    async fn test_context_compression() {
        // Use a temporary file for the database in tests
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(&db_path).await.unwrap();
        let context_manager = ContextManager::new(db);

        let mut context = ConversationContext {
            available_tools: Vec::new(),
            data_summary: DataSummary {
                entity_count_by_type: HashMap::new(),
                relation_count_by_type: HashMap::new(),
                item_count_by_timeframe: HashMap::new(),
                data_freshness: Utc::now(),
            },
            user_preferences: UserPreferences {
                max_conversation_steps: Some(10),
                preferred_response_format: None,
                timeout_seconds: Some(60),
            },
            execution_history: Vec::new(),
            intermediate_results: HashMap::new(),
        };

        // Add many execution history items
        for i in 0..150 {
            context.execution_history.push(ToolExecution {
                tool_name: format!("tool_{}", i),
                parameters: serde_json::json!({"param": i}),
                result: Some(serde_json::json!({"result": i})),
                error: None,
                execution_time_ms: Some(100),
            });
        }

        let original_count = context.execution_history.len();
        context_manager
            .compress_context(&mut context)
            .await
            .unwrap();

        assert!(context.execution_history.len() < original_count);
        assert!(context.execution_history.len() <= context_manager.max_history_items / 2);
    }
}
