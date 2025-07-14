use super::manager::ConversationStore;
use super::*;
use crate::database::Database;
use async_trait::async_trait;
use sqlx::Row;

pub struct SqliteConversationStore {
    db: Database,
}

#[allow(dead_code)]
impl SqliteConversationStore {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn init_tables(&self) -> Result<(), ConversationError> {
        let conn = self
            .db
            .get_connection()
            .await
            .map_err(|e| ConversationError::StorageError(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                user_query TEXT NOT NULL,
                state TEXT NOT NULL,
                context TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL
            )
            "#,
        )
        .execute(&conn)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS conversation_steps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id TEXT NOT NULL,
                step_id INTEGER NOT NULL,
                step_type TEXT NOT NULL,
                llm_request TEXT,
                llm_response TEXT,
                tool_calls TEXT NOT NULL,
                results TEXT,
                status TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES conversations (id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&conn)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl ConversationStore for SqliteConversationStore {
    async fn save_conversation(
        &self,
        conversation: &Conversation,
    ) -> Result<(), ConversationError> {
        let conn = self
            .db
            .get_connection()
            .await
            .map_err(|e| ConversationError::StorageError(e.to_string()))?;

        let context_json = serde_json::to_string(&conversation.context)?;

        sqlx::query(
            r#"
            INSERT INTO conversations (id, user_query, state, context, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(conversation.id.to_string())
        .bind(&conversation.user_query)
        .bind(serde_json::to_string(&conversation.state)?)
        .bind(context_json)
        .bind(conversation.created_at)
        .bind(conversation.updated_at)
        .execute(&conn)
        .await?;

        // Save steps
        for step in &conversation.steps {
            let tool_calls_json = serde_json::to_string(&step.tool_calls)?;
            let results_json = step
                .results
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?;

            sqlx::query(
                r#"
                INSERT INTO conversation_steps 
                (conversation_id, step_id, step_type, llm_request, llm_response, tool_calls, results, status, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(conversation.id.to_string())
            .bind(step.step_id as i64)
            .bind(serde_json::to_string(&step.step_type)?)
            .bind(&step.llm_request)
            .bind(&step.llm_response)
            .bind(tool_calls_json)
            .bind(results_json)
            .bind(serde_json::to_string(&step.status)?)
            .bind(step.created_at)
            .execute(&conn)
            .await?;
        }

        Ok(())
    }

    async fn load_conversation(&self, id: Uuid) -> Result<Option<Conversation>, ConversationError> {
        let conn = self
            .db
            .get_connection()
            .await
            .map_err(|e| ConversationError::StorageError(e.to_string()))?;

        let row = sqlx::query(
            "SELECT id, user_query, state, context, created_at, updated_at FROM conversations WHERE id = ?"
        )
        .bind(id.to_string())
        .fetch_optional(&conn)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        let conversation_id: String = row.get("id");
        let user_query: String = row.get("user_query");
        let state_json: String = row.get("state");
        let context_json: String = row.get("context");
        let created_at: DateTime<Utc> = row.get("created_at");
        let updated_at: DateTime<Utc> = row.get("updated_at");

        let state: ConversationState = serde_json::from_str(&state_json)?;
        let context: ConversationContext = serde_json::from_str(&context_json)?;

        // Load steps
        let step_rows = sqlx::query(
            r#"
            SELECT step_id, step_type, llm_request, llm_response, tool_calls, results, status, created_at
            FROM conversation_steps 
            WHERE conversation_id = ? 
            ORDER BY step_id
            "#
        )
        .bind(&conversation_id)
        .fetch_all(&conn)
        .await?;

        let mut steps = Vec::new();
        for step_row in step_rows {
            let step_id: i64 = step_row.get("step_id");
            let step_type_json: String = step_row.get("step_type");
            let llm_request: Option<String> = step_row.get("llm_request");
            let llm_response: Option<String> = step_row.get("llm_response");
            let tool_calls_json: String = step_row.get("tool_calls");
            let results_json: Option<String> = step_row.get("results");
            let status_json: String = step_row.get("status");
            let step_created_at: DateTime<Utc> = step_row.get("created_at");

            let step_type: StepType = serde_json::from_str(&step_type_json)?;
            let tool_calls: Vec<ToolExecution> = serde_json::from_str(&tool_calls_json)?;
            let results: Option<StepResult> = results_json
                .map(|json| serde_json::from_str(&json))
                .transpose()?;
            let status: StepStatus = serde_json::from_str(&status_json)?;

            steps.push(ConversationStep {
                step_id: step_id as u32,
                step_type,
                llm_request,
                llm_response,
                tool_calls,
                results,
                status,
                created_at: step_created_at,
            });
        }

        Ok(Some(Conversation {
            id: Uuid::parse_str(&conversation_id)
                .map_err(|e| ConversationError::SerializationError(e.to_string()))?,
            user_query,
            state,
            steps,
            context,
            created_at,
            updated_at,
        }))
    }

    async fn update_conversation(
        &self,
        conversation: &Conversation,
    ) -> Result<(), ConversationError> {
        let conn = self
            .db
            .get_connection()
            .await
            .map_err(|e| ConversationError::StorageError(e.to_string()))?;

        let context_json = serde_json::to_string(&conversation.context)?;

        sqlx::query("UPDATE conversations SET state = ?, context = ?, updated_at = ? WHERE id = ?")
            .bind(serde_json::to_string(&conversation.state)?)
            .bind(context_json)
            .bind(conversation.updated_at)
            .bind(conversation.id.to_string())
            .execute(&conn)
            .await?;

        // Delete existing steps and re-insert (simple approach)
        sqlx::query("DELETE FROM conversation_steps WHERE conversation_id = ?")
            .bind(conversation.id.to_string())
            .execute(&conn)
            .await?;

        // Re-insert all steps
        for step in &conversation.steps {
            let tool_calls_json = serde_json::to_string(&step.tool_calls)?;
            let results_json = step
                .results
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?;

            sqlx::query(
                r#"
                INSERT INTO conversation_steps 
                (conversation_id, step_id, step_type, llm_request, llm_response, tool_calls, results, status, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(conversation.id.to_string())
            .bind(step.step_id as i64)
            .bind(serde_json::to_string(&step.step_type)?)
            .bind(&step.llm_request)
            .bind(&step.llm_response)
            .bind(tool_calls_json)
            .bind(results_json)
            .bind(serde_json::to_string(&step.status)?)
            .bind(step.created_at)
            .execute(&conn)
            .await?;
        }

        Ok(())
    }

    async fn delete_conversation(&self, id: Uuid) -> Result<(), ConversationError> {
        let conn = self
            .db
            .get_connection()
            .await
            .map_err(|e| ConversationError::StorageError(e.to_string()))?;

        sqlx::query("DELETE FROM conversations WHERE id = ?")
            .bind(id.to_string())
            .execute(&conn)
            .await?;

        Ok(())
    }

    async fn list_conversations(
        &self,
        limit: Option<u32>,
    ) -> Result<Vec<Conversation>, ConversationError> {
        let conn = self
            .db
            .get_connection()
            .await
            .map_err(|e| ConversationError::StorageError(e.to_string()))?;

        let limit_clause = match limit {
            Some(l) => format!("LIMIT {l}"),
            None => "".to_string(),
        };

        let query = format!(
            "SELECT id, user_query, state, context, created_at, updated_at FROM conversations ORDER BY updated_at DESC {limit_clause}"
        );

        let rows = sqlx::query(&query).fetch_all(&conn).await?;

        let mut conversations = Vec::new();
        for row in rows {
            let conversation_id: String = row.get("id");
            let user_query: String = row.get("user_query");
            let state_json: String = row.get("state");
            let context_json: String = row.get("context");
            let created_at: DateTime<Utc> = row.get("created_at");
            let updated_at: DateTime<Utc> = row.get("updated_at");

            let state: ConversationState = serde_json::from_str(&state_json)?;
            let context: ConversationContext = serde_json::from_str(&context_json)?;

            // For list view, we don't load steps to keep it lightweight
            conversations.push(Conversation {
                id: Uuid::parse_str(&conversation_id)
                    .map_err(|e| ConversationError::SerializationError(e.to_string()))?,
                user_query,
                state,
                steps: Vec::new(), // Empty for list view
                context,
                created_at,
                updated_at,
            });
        }

        Ok(conversations)
    }
}
