use pixlie::conversation::{
    Conversation, ConversationState,
    manager::{ConversationManager, ConversationStore},
    storage::SqliteConversationStore,
};
use pixlie::database::Database;
use pixlie::llm::mock::MockLLMProvider;
use pixlie::tools::ToolRegistry;
use std::sync::Arc;
use tempfile::tempdir;
use uuid::Uuid;

#[tokio::test]
async fn test_conversation_lifecycle() {
    // Create a temporary database
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await.unwrap();

    // Initialize conversation storage
    let conversation_store = SqliteConversationStore::new(db.clone());
    conversation_store.init_tables().await.unwrap();

    // Initialize LLM provider and tool registry
    let llm_provider = Box::new(MockLLMProvider::new());
    let tool_registry = Arc::new(ToolRegistry::new());

    // Create conversation manager
    let manager =
        ConversationManager::new(llm_provider, tool_registry, Box::new(conversation_store));

    // Test starting a conversation
    let conversation = manager
        .start_conversation("Test query")
        .await
        .expect("Failed to start conversation");

    assert_eq!(conversation.user_query, "Test query");
    assert_eq!(conversation.state, ConversationState::Planning);
    assert!(!conversation.steps.is_empty());

    // Test getting the conversation
    let retrieved = manager
        .get_conversation(conversation.id)
        .await
        .expect("Failed to get conversation")
        .expect("Conversation not found");

    assert_eq!(retrieved.id, conversation.id);
    assert_eq!(retrieved.user_query, conversation.user_query);

    // Test listing conversations
    let conversations = manager
        .list_conversations(Some(10))
        .await
        .expect("Failed to list conversations");

    assert_eq!(conversations.len(), 1);
    assert_eq!(conversations[0].id, conversation.id);

    // Test deleting the conversation
    manager
        .delete_conversation(conversation.id)
        .await
        .expect("Failed to delete conversation");

    // Verify deletion
    let deleted = manager
        .get_conversation(conversation.id)
        .await
        .expect("Failed to check deleted conversation");

    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_conversation_storage() {
    // Create a temporary database
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await.unwrap();

    // Initialize conversation storage
    let store = SqliteConversationStore::new(db);
    store.init_tables().await.unwrap();

    // Create a test conversation
    let conversation_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let conversation = Conversation {
        id: conversation_id,
        user_query: "Test storage query".to_string(),
        state: ConversationState::Planning,
        steps: Vec::new(),
        context: pixlie::conversation::ConversationContext {
            available_tools: Vec::new(),
            data_summary: pixlie::conversation::DataSummary {
                entity_count_by_type: std::collections::HashMap::new(),
                relation_count_by_type: std::collections::HashMap::new(),
                item_count_by_timeframe: std::collections::HashMap::new(),
                data_freshness: now,
            },
            user_preferences: pixlie::conversation::UserPreferences {
                max_conversation_steps: Some(10),
                preferred_response_format: None,
                timeout_seconds: Some(60),
            },
            execution_history: Vec::new(),
            intermediate_results: std::collections::HashMap::new(),
        },
        created_at: now,
        updated_at: now,
    };

    // Test saving
    store
        .save_conversation(&conversation)
        .await
        .expect("Failed to save conversation");

    // Test loading
    let loaded = store
        .load_conversation(conversation_id)
        .await
        .expect("Failed to load conversation")
        .expect("Conversation not found");

    assert_eq!(loaded.id, conversation.id);
    assert_eq!(loaded.user_query, conversation.user_query);
    assert_eq!(loaded.state, conversation.state);

    // Test updating
    let mut updated_conversation = loaded;
    updated_conversation.state = ConversationState::Executing;
    updated_conversation.updated_at = chrono::Utc::now();

    store
        .update_conversation(&updated_conversation)
        .await
        .expect("Failed to update conversation");

    // Verify update
    let reloaded = store
        .load_conversation(conversation_id)
        .await
        .expect("Failed to reload conversation")
        .expect("Conversation not found");

    assert_eq!(reloaded.state, ConversationState::Executing);

    // Test listing
    let conversations = store
        .list_conversations(Some(10))
        .await
        .expect("Failed to list conversations");

    assert_eq!(conversations.len(), 1);

    // Test deletion
    store
        .delete_conversation(conversation_id)
        .await
        .expect("Failed to delete conversation");

    let deleted = store
        .load_conversation(conversation_id)
        .await
        .expect("Failed to check deleted conversation");

    assert!(deleted.is_none());
}
