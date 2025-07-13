use crate::config::Config;
use crate::database::{Database, DownloadStats, ExtractionStats};
use crate::entity_extraction::{EntityExtractor, ModelInfo};
use crate::hn_api::HnApiClient;
use crate::llm::LLMProvider;
use crate::tools::{
    ToolArguments, ToolCategory, ToolDescriptor, ToolRegistry, ToolValidator, ValidationError,
    types,
};
use actix_web::{HttpResponse, Result, web};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::task;
use ts_rs::TS;

pub type AppState = web::Data<Arc<AppData>>;

pub struct AppData {
    pub config: Mutex<Config>,
    pub database: Option<Database>,
    pub hn_client: HnApiClient,
    pub entity_extractor: Mutex<EntityExtractor>,
    pub tool_registry: Mutex<ToolRegistry>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConfigResponse {
    pub config_path: String,
    #[ts(type = "string | null")]
    pub data_folder: Option<PathBuf>,
    pub download_stats: DownloadStats,
}

#[derive(Deserialize, Serialize, TS)]
#[ts(export)]
pub struct SetDataFolderRequest {
    pub folder_path: String,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DownloadStatusResponse {
    pub download_stats: DownloadStats,
}

#[derive(Deserialize, Serialize, TS)]
#[ts(export)]
pub struct StartDownloadRequest {
    pub download_type: String, // "stories", "recent", "all"
    pub limit: Option<u64>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ModelsResponse {
    pub models: Vec<ModelInfo>,
}

#[derive(Deserialize, Serialize, TS)]
#[ts(export)]
pub struct DownloadModelRequest {
    pub model_name: String,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ExtractionStatusResponse {
    pub extraction_stats: ExtractionStats,
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct StartExtractionRequest {
    pub batch_size: Option<u64>,
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct GetItemsRequest {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct GetItemsResponse {
    pub items: Vec<crate::database::HnItem>,
    pub total_count: u32,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct GetEntitiesRequest {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct GetEntitiesResponse {
    pub entities: Vec<crate::database::Entity>,
    pub total_count: u32,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct GetRelationsRequest {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub entity_id: Option<i64>,
    pub relation_type: Option<String>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct GetRelationsResponse {
    pub relations: Vec<crate::database::EntityRelation>,
    pub total_count: u32,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct SearchEntitiesRequest {
    pub q: Option<String>,
    pub entity_type: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct SearchEntitiesResponse {
    pub entities: Vec<crate::database::Entity>,
    pub total_count: u32,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct EntityDetailResponse {
    pub entity: crate::database::Entity,
    pub references_count: u32,
    pub items_count: u32,
    pub relations_count: u32,
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct GetEntityReferencesRequest {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct GetEntityReferencesResponse {
    pub references: Vec<crate::database::EntityReference>,
    pub total_count: u32,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct GetEntityItemsRequest {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct EntityItemWithHighlights {
    pub item: crate::database::HnItem,
    pub highlights: Vec<crate::database::EntityReference>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct GetEntityItemsResponse {
    pub items: Vec<EntityItemWithHighlights>,
    pub total_count: u32,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

pub async fn get_config(data: AppState) -> Result<HttpResponse> {
    let config_path =
        Config::get_config_path().map_err(actix_web::error::ErrorInternalServerError)?;

    let data_folder = {
        let config = data.config.lock().unwrap();
        config.data_folder.clone()
    };

    let download_stats = if let Some(ref db) = data.database {
        db.get_stats().await.unwrap_or(DownloadStats {
            total_items: 0,
            total_users: 0,
            last_download_time: None,
            items_downloaded_today: 0,
            download_errors: 0,
            is_downloading: false,
        })
    } else {
        DownloadStats {
            total_items: 0,
            total_users: 0,
            last_download_time: None,
            items_downloaded_today: 0,
            download_errors: 0,
            is_downloading: false,
        }
    };

    let response = ConfigResponse {
        config_path: config_path.to_string_lossy().to_string(),
        data_folder,
        download_stats,
    };

    Ok(HttpResponse::Ok().json(response))
}

// LLM Handlers

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct LLMQueryRequest {
    pub query: String,
    #[ts(type = "string | null")]
    pub context: Option<String>,
}

pub async fn llm_query(_data: AppState, req: web::Json<LLMQueryRequest>) -> Result<HttpResponse> {
    // For now, we are using a mock provider.
    // In the future, we will use the provider from the config.
    let provider = crate::llm::mock::MockLLMProvider;

    let tools = vec![]; // TODO: Get tools from the tool registry

    let response = provider
        .send_query(&req.query, &tools, req.context.as_deref())
        .await;

    match response {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

pub async fn list_llm_tools(data: AppState) -> Result<HttpResponse> {
    let tool_registry = data.tool_registry.lock().unwrap();
    let tool_descriptors = tool_registry.get_all_descriptors();
    Ok(HttpResponse::Ok().json(tool_descriptors))
}

// Tool-specific endpoint handlers

#[derive(Deserialize)]
pub struct ExecuteToolRequest {
    pub parameters: serde_json::Value,
    pub context: Option<serde_json::Value>,
}

pub async fn execute_tool(
    data: AppState,
    path: web::Path<String>,
    req: web::Json<ExecuteToolRequest>,
) -> Result<HttpResponse> {
    let tool_name = path.into_inner();

    let context = req
        .context
        .as_ref()
        .and_then(|c| serde_json::from_value(c.clone()).ok());

    let args = ToolArguments {
        parameters: req.parameters.clone(),
        context,
    };

    // Clone the tool to avoid holding the lock across await
    let tool = {
        let tool_registry = data.tool_registry.lock().unwrap();
        tool_registry.get_tool_cloned(&tool_name)
    };

    let result = if let Some(tool) = tool {
        Some(tool.execute(args).await)
    } else {
        None
    };

    // Update metrics after execution would happen here
    // For now we skip this since it requires more complex state management

    if let Some(tool_result) = result {
        Ok(HttpResponse::Ok().json(tool_result))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Tool not found",
            "tool_name": tool_name
        })))
    }
}

pub async fn get_tool_descriptor(data: AppState, path: web::Path<String>) -> Result<HttpResponse> {
    let tool_name = path.into_inner();
    let tool_registry = data.tool_registry.lock().unwrap();

    if let Some(tool) = tool_registry.get_tool(&tool_name) {
        let descriptor = tool.describe();
        Ok(HttpResponse::Ok().json(descriptor))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "error": format!("Tool '{}' not found", tool_name)
        })))
    }
}

pub async fn get_tools_by_category(
    data: AppState,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let category_str = path.into_inner();
    let tool_registry = data.tool_registry.lock().unwrap();

    let category = match category_str.as_str() {
        "data_query" => ToolCategory::DataQuery,
        "entity_analysis" => ToolCategory::EntityAnalysis,
        "relation_exploration" => ToolCategory::RelationExploration,
        "analytics" => ToolCategory::Analytics,
        _ => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": format!("Invalid category: {}", category_str)
            })));
        }
    };

    let tools = tool_registry.get_tools_by_category(category);
    Ok(HttpResponse::Ok().json(tools))
}

pub async fn get_tool_schema(data: AppState) -> Result<HttpResponse> {
    let tool_registry = data.tool_registry.lock().unwrap();
    let descriptors = tool_registry.get_all_descriptors();

    let schema = serde_json::json!({
        "tools": descriptors,
        "categories": [
            "data_query",
            "entity_analysis",
            "relation_exploration",
            "analytics"
        ],
        "version": "1.0.0"
    });

    Ok(HttpResponse::Ok().json(schema))
}

pub async fn get_tool_metrics(data: AppState, path: web::Path<String>) -> Result<HttpResponse> {
    let tool_name = path.into_inner();
    let tool_registry = data.tool_registry.lock().unwrap();

    if let Some(metrics) = tool_registry.get_tool_metrics(&tool_name) {
        Ok(HttpResponse::Ok().json(metrics))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "error": format!("Tool '{}' not found", tool_name)
        })))
    }
}

pub async fn get_llm_conversation() -> Result<HttpResponse> {
    let conversation: Vec<serde_json::Value> = vec![]; // TODO: Implement conversation history
    Ok(HttpResponse::Ok().json(conversation))
}

pub async fn set_data_folder(
    data: AppState,
    req: web::Json<SetDataFolderRequest>,
) -> Result<HttpResponse> {
    let folder_path = PathBuf::from(&req.folder_path);

    let config_result = {
        let mut config = data.config.lock().unwrap();
        config.set_data_folder(folder_path.clone())
    };

    match config_result {
        Ok(_) => {
            // Initialize database in the new data folder
            let db_path = folder_path.join("hackernews.db");
            match Database::new(&db_path).await {
                Ok(_database) => {
                    // Update the database in AppData (this is a simplified approach)
                    // In a real implementation, you'd want better state management
                    Ok(HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "message": "Data folder set successfully and database initialized"
                    })))
                }
                Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to initialize database: {e}")
                }))),
            }
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

pub async fn start_download(
    data: AppState,
    req: web::Json<StartDownloadRequest>,
) -> Result<HttpResponse> {
    // Check if data folder is set
    let _data_folder = {
        let config = data.config.lock().unwrap();
        match &config.data_folder {
            Some(folder) => folder.clone(),
            None => {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "success": false,
                    "error": "Data folder not set. Please set a data folder first."
                })));
            }
        }
    };

    // Check if database exists
    let database = match &data.database {
        Some(db) => db,
        None => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": "Database not initialized. Please set a data folder first."
            })));
        }
    };

    // Check if already downloading
    let stats = database
        .get_stats()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    if stats.is_downloading {
        return Ok(HttpResponse::Conflict().json(serde_json::json!({
            "success": false,
            "error": "Download is already in progress"
        })));
    }

    // Start download session
    let session_id = database
        .start_download_session(&req.download_type)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Clone necessary data for the async task
    let app_data = data.clone();
    let download_type = req.download_type.clone();
    let limit = req.limit;

    // Spawn download task
    task::spawn(async move {
        if let Err(e) = perform_download(app_data, session_id, download_type, limit).await {
            eprintln!("Download failed: {e}");
        }
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Download started in background"
    })))
}

pub async fn stop_download(data: AppState) -> Result<HttpResponse> {
    // Check if database exists
    let database = match &data.database {
        Some(db) => db,
        None => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": "Database not initialized"
            })));
        }
    };

    // Check if downloading
    let stats = database
        .get_stats()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    if !stats.is_downloading {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": "No download in progress"
        })));
    }

    // For now, we'll just mark running sessions as stopped
    // In a more sophisticated implementation, you'd have proper cancellation
    match database.stop_all_downloads().await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Download stopped"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("Failed to stop download: {}", e)
        }))),
    }
}

pub async fn get_download_status(data: AppState) -> Result<HttpResponse> {
    let download_stats = if let Some(ref db) = data.database {
        db.get_stats()
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        DownloadStats {
            total_items: 0,
            total_users: 0,
            last_download_time: None,
            items_downloaded_today: 0,
            download_errors: 0,
            is_downloading: false,
        }
    };

    let response = DownloadStatusResponse { download_stats };

    Ok(HttpResponse::Ok().json(response))
}

async fn perform_download(
    app_data: web::Data<Arc<AppData>>,
    session_id: i64,
    download_type: String,
    limit: Option<u64>,
) -> Result<(), String> {
    let database = app_data.database.as_ref().ok_or("Database not available")?;

    // Simple synchronous approach - download items and save them directly
    let result = match download_type.as_str() {
        "stories" => {
            let story_ids = app_data
                .hn_client
                .get_top_stories()
                .await
                .map_err(|e| e.to_string())?;
            let limited_ids = if let Some(limit) = limit {
                story_ids.into_iter().take(limit as usize).collect()
            } else {
                story_ids
            };

            let mut downloaded = 0u64;
            let mut errors = 0u64;

            for (index, story_id) in limited_ids.iter().enumerate() {
                match app_data.hn_client.get_item(*story_id).await {
                    Ok(Some(item)) => {
                        if let Err(e) = database.insert_item(&item).await {
                            eprintln!("Failed to save item {}: {}", item.id, e);
                            errors += 1;
                        } else {
                            downloaded += 1;
                        }
                    }
                    Ok(None) => {
                        errors += 1;
                    }
                    Err(e) => {
                        eprintln!("Error fetching story {story_id}: {e}");
                        errors += 1;
                    }
                }

                // Update progress every 10 items
                if index % 10 == 0 {
                    let _ = database
                        .update_download_session(session_id, downloaded, errors)
                        .await;
                    println!("Downloaded {}/{} stories", index + 1, limited_ids.len());
                }
            }

            Ok((downloaded, errors))
        }
        "recent" => {
            let max_id = app_data
                .hn_client
                .get_max_item_id()
                .await
                .map_err(|e| e.to_string())?;
            let start_id = max_id.saturating_sub(limit.unwrap_or(1000) as i64);

            let mut downloaded = 0u64;
            let mut errors = 0u64;

            for id in start_id..=max_id {
                match app_data.hn_client.get_item(id).await {
                    Ok(Some(item)) => {
                        if let Err(e) = database.insert_item(&item).await {
                            eprintln!("Failed to save item {}: {}", item.id, e);
                            errors += 1;
                        } else {
                            downloaded += 1;
                        }
                    }
                    Ok(None) => {
                        // Item doesn't exist, not an error
                    }
                    Err(e) => {
                        eprintln!("Error fetching item {id}: {e}");
                        errors += 1;
                    }
                }

                // Update progress every 50 items
                if (id - start_id) % 50 == 0 {
                    let _ = database
                        .update_download_session(session_id, downloaded, errors)
                        .await;
                    let progress =
                        ((id - start_id) as f64 / (max_id - start_id) as f64 * 100.0) as u32;
                    println!(
                        "Progress: {}% ({}/{})",
                        progress,
                        id - start_id + 1,
                        max_id - start_id + 1
                    );
                }
            }

            Ok((downloaded, errors))
        }
        _ => {
            return Err("Invalid download type".to_string());
        }
    };

    match result {
        Ok((downloaded, errors)) => {
            database
                .update_download_session(session_id, downloaded, errors)
                .await
                .map_err(|e| e.to_string())?;
            database
                .complete_download_session(session_id, "completed")
                .await
                .map_err(|e| e.to_string())?;
            println!("Download completed: {downloaded} items downloaded, {errors} errors");
        }
        Err(e) => {
            let _ = database
                .complete_download_session(session_id, "failed")
                .await;
            eprintln!("Download failed: {e}");
            return Err(e);
        }
    }

    Ok(())
}

// Entity extraction handlers
pub async fn get_models(data: AppState) -> Result<HttpResponse> {
    let config = data.config.lock().unwrap();
    let models = if let Some(ref data_folder) = config.data_folder {
        let models_dir = data_folder.join("models");
        EntityExtractor::get_available_models_with_status(&models_dir)
    } else {
        EntityExtractor::get_available_models()
    };
    drop(config);

    let response = ModelsResponse { models };
    Ok(HttpResponse::Ok().json(response))
}

pub async fn download_model(
    data: AppState,
    req: web::Json<DownloadModelRequest>,
) -> Result<HttpResponse> {
    let config = data.config.lock().unwrap();

    // Check if data folder is set
    let data_folder = match &config.data_folder {
        Some(folder) => folder.clone(),
        None => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Data folder not set. Please set a data folder first."
            })));
        }
    };
    drop(config);

    let models_dir = data_folder.join("models");
    let model_name = req.model_name.clone();

    // Clone data for async task
    let app_data = data.clone();

    // Spawn download task
    task::spawn(async move {
        match EntityExtractor::download_model(&model_name, &models_dir).await {
            Ok(model_path) => {
                println!("Model {model_name} downloaded successfully to {model_path}");
                // Load the model into the shared extractor
                {
                    let mut extractor = app_data.entity_extractor.lock().unwrap();
                    if let Err(e) = extractor.load_model(&model_path) {
                        eprintln!("Failed to load model: {e}");
                    } else {
                        println!("Model {model_name} loaded successfully");
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to download model {model_name}: {e}");
            }
        }
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Model download started in background"
    })))
}

pub async fn start_extraction(
    data: AppState,
    req: web::Json<StartExtractionRequest>,
) -> Result<HttpResponse> {
    // Check if database exists
    let database = match &data.database {
        Some(db) => db,
        None => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": "Database not initialized. Please set a data folder first."
            })));
        }
    };

    // Check if model is loaded and not already extracting
    let (is_model_loaded, is_extracting) = {
        let extractor = data.entity_extractor.lock().unwrap();
        (extractor.is_model_loaded(), extractor.is_extracting())
    };

    if !is_model_loaded {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": "No model loaded. Please download a model first."
        })));
    }

    if is_extracting {
        return Ok(HttpResponse::Conflict().json(serde_json::json!({
            "success": false,
            "error": "Extraction is already in progress"
        })));
    }

    // Start extraction session
    let session_id = database
        .start_extraction_session()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Clone data for async task
    let app_data = data.clone();
    let batch_size = req.batch_size.unwrap_or(100);

    // Spawn extraction task
    task::spawn(async move {
        if let Err(e) = perform_extraction(app_data, session_id, batch_size).await {
            eprintln!("Extraction failed: {e}");
        }
    });

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Entity extraction started in background"
    })))
}

pub async fn stop_extraction(data: AppState) -> Result<HttpResponse> {
    // Check if extraction is in progress and stop it
    let is_extracting = {
        let extractor = data.entity_extractor.lock().unwrap();
        let extracting = extractor.is_extracting();
        if extracting {
            extractor.stop_extraction();
        }
        extracting
    };

    if !is_extracting {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": "No extraction in progress"
        })));
    }

    // Stop extraction sessions in database
    if let Some(database) = &data.database {
        let _ = database.stop_all_extractions().await;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Extraction stopped"
    })))
}

pub async fn get_extraction_status(data: AppState) -> Result<HttpResponse> {
    let extraction_stats = if let Some(ref db) = data.database {
        db.get_extraction_stats()
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?
    } else {
        ExtractionStats {
            total_entities: 0,
            entities_by_type: std::collections::HashMap::new(),
            total_items_processed: 0,
            items_remaining: 0,
            is_extracting: false,
            last_extraction_time: None,
        }
    };

    let response = ExtractionStatusResponse { extraction_stats };
    Ok(HttpResponse::Ok().json(response))
}

async fn perform_extraction(
    app_data: web::Data<Arc<AppData>>,
    session_id: i64,
    batch_size: u64,
) -> Result<(), String> {
    let database = app_data.database.as_ref().ok_or("Database not available")?;

    // Get items that haven't been processed for entity extraction
    let items = database
        .get_items_for_extraction(batch_size as i64)
        .await
        .map_err(|e| e.to_string())?;

    if items.is_empty() {
        database
            .complete_extraction_session(session_id, "completed")
            .await
            .map_err(|e| e.to_string())?;
        println!("No items to process for entity extraction");
        return Ok(());
    }

    // Extract entities
    let (is_extracting, model_path) = {
        let extractor = app_data.entity_extractor.lock().unwrap();
        (
            extractor.is_extracting.clone(),
            extractor.model_path.clone(),
        )
    };

    let result = EntityExtractor::extract_entities_from_items(
        items,
        database,
        session_id,
        is_extracting,
        model_path,
    )
    .await;

    match result {
        Ok((entities_extracted, items_processed)) => {
            database
                .complete_extraction_session(session_id, "completed")
                .await
                .map_err(|e| e.to_string())?;
            println!(
                "Extraction completed: {entities_extracted} entities extracted from {items_processed} items"
            );
        }
        Err(e) => {
            let _ = database
                .complete_extraction_session(session_id, "failed")
                .await;
            eprintln!("Extraction failed: {e}");
            return Err(e.to_string());
        }
    }

    Ok(())
}

pub async fn get_items(data: AppState, req: web::Query<GetItemsRequest>) -> Result<HttpResponse> {
    // Check if database exists
    let database = match &data.database {
        Some(db) => db,
        None => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": "Database not initialized. Please set a data folder first."
            })));
        }
    };

    let page = req.page.unwrap_or(1);
    let limit = req.limit.unwrap_or(100);

    // Ensure page is at least 1
    let page = if page < 1 { 1 } else { page };

    // Calculate offset
    let offset = (page - 1) * limit;

    // Get total count for pagination info
    let total_count = match database.get_total_items_count().await {
        Ok(count) => count as u32,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to get total count: {}", e)
            })));
        }
    };

    // Get paginated items
    let items = match database
        .get_items_paginated(limit as i64, offset as i64)
        .await
    {
        Ok(items) => items,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch items: {}", e)
            })));
        }
    };

    // Calculate total pages
    let total_pages = if total_count > 0 {
        total_count.div_ceil(limit)
    } else {
        0
    };

    let response = GetItemsResponse {
        items,
        total_count,
        page,
        limit,
        total_pages,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_entities(
    data: AppState,
    req: web::Query<GetEntitiesRequest>,
) -> Result<HttpResponse> {
    // Check if database exists
    let database = match &data.database {
        Some(db) => db,
        None => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": "Database not initialized. Please set a data folder first."
            })));
        }
    };

    let page = req.page.unwrap_or(1);
    let limit = req.limit.unwrap_or(100);

    // Ensure page is at least 1
    let page = if page < 1 { 1 } else { page };

    // Calculate offset
    let offset = (page - 1) * limit;

    // Get total count for pagination info
    let total_count = match database.get_total_entities_count().await {
        Ok(count) => count as u32,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to get total count: {}", e)
            })));
        }
    };

    // Get paginated entities
    let entities = match database
        .get_entities_paginated(limit as i64, offset as i64)
        .await
    {
        Ok(entities) => entities,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch entities: {}", e)
            })));
        }
    };

    // Calculate total pages
    let total_pages = if total_count > 0 {
        total_count.div_ceil(limit)
    } else {
        0
    };

    let response = GetEntitiesResponse {
        entities,
        total_count,
        page,
        limit,
        total_pages,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_relations(
    data: AppState,
    req: web::Query<GetRelationsRequest>,
) -> Result<HttpResponse> {
    // Check if database exists
    let database = match &data.database {
        Some(db) => db,
        None => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": "Database not initialized. Please set a data folder first."
            })));
        }
    };

    let page = req.page.unwrap_or(1);
    let limit = req.limit.unwrap_or(100);

    // Ensure page is at least 1
    let page = if page < 1 { 1 } else { page };

    // Calculate offset
    let offset = (page - 1) * limit;

    let (relations, total_count) = match (req.entity_id, req.relation_type.as_deref()) {
        (Some(entity_id), None) => {
            // Get relations for a specific entity
            let relations = match database.get_relations_for_entity(entity_id).await {
                Ok(relations) => relations,
                Err(e) => {
                    return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to fetch relations for entity: {}", e)
                    })));
                }
            };
            let total_count = relations.len() as u32;

            // Apply pagination manually for entity-specific relations
            let paginated_relations = relations
                .into_iter()
                .skip(offset as usize)
                .take(limit as usize)
                .collect();

            (paginated_relations, total_count)
        }
        (None, Some(relation_type)) => {
            // Get relations filtered by type
            let total_count = match database.get_relations_by_type_count(relation_type).await {
                Ok(count) => count as u32,
                Err(e) => {
                    return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to get relations count by type: {}", e)
                    })));
                }
            };

            let relations = match database
                .get_relations_by_type(relation_type, limit as i64, offset as i64)
                .await
            {
                Ok(relations) => relations,
                Err(e) => {
                    return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to fetch relations by type: {}", e)
                    })));
                }
            };

            (relations, total_count)
        }
        (Some(_entity_id), Some(_relation_type)) => {
            // TODO: Implement combined entity_id + relation_type filtering
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "success": false,
                "error": "Filtering by both entity_id and relation_type is not yet supported"
            })));
        }
        (None, None) => {
            // Get all relations with pagination
            let total_count = match database.get_total_relations_count().await {
                Ok(count) => count as u32,
                Err(e) => {
                    return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to get total count: {}", e)
                    })));
                }
            };

            let relations = match database
                .get_relations_paginated(limit as i64, offset as i64)
                .await
            {
                Ok(relations) => relations,
                Err(e) => {
                    return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "error": format!("Failed to fetch relations: {}", e)
                    })));
                }
            };

            (relations, total_count)
        }
    };

    // Calculate total pages
    let total_pages = if total_count > 0 {
        total_count.div_ceil(limit)
    } else {
        0
    };

    let response = GetRelationsResponse {
        relations,
        total_count,
        page,
        limit,
        total_pages,
    };

    Ok(HttpResponse::Ok().json(response))
}

// New entity browser handlers

pub async fn search_entities(
    data: AppState,
    req: web::Query<SearchEntitiesRequest>,
) -> Result<HttpResponse> {
    // Check if database exists
    let database = match &data.database {
        Some(db) => db,
        None => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": "Database not initialized. Please set a data folder first."
            })));
        }
    };

    let query = req.q.as_deref().unwrap_or("");
    let entity_type = req.entity_type.as_deref();
    let page = req.page.unwrap_or(1);
    let limit = req.limit.unwrap_or(50);

    // Ensure page is at least 1
    let page = if page < 1 { 1 } else { page };

    // Calculate offset
    let offset = (page - 1) * limit;

    if query.is_empty() {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "success": false,
            "error": "Search query 'q' parameter is required"
        })));
    }

    // Get total count for pagination
    let total_count = match database.search_entities_count(query, entity_type).await {
        Ok(count) => count as u32,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to get search count: {}", e)
            })));
        }
    };

    // Get paginated search results
    let entities = match database
        .search_entities(query, entity_type, limit as i64, offset as i64)
        .await
    {
        Ok(entities) => entities,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to search entities: {}", e)
            })));
        }
    };

    // Calculate total pages
    let total_pages = if total_count > 0 {
        total_count.div_ceil(limit)
    } else {
        0
    };

    let response = SearchEntitiesResponse {
        entities,
        total_count,
        page,
        limit,
        total_pages,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_entity_detail(data: AppState, path: web::Path<i64>) -> Result<HttpResponse> {
    // Check if database exists
    let database = match &data.database {
        Some(db) => db,
        None => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": "Database not initialized. Please set a data folder first."
            })));
        }
    };

    let entity_id = path.into_inner();

    // Get the entity
    let entity = match database.get_entity_by_id(entity_id).await {
        Ok(Some(entity)) => entity,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "error": "Entity not found"
            })));
        }
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch entity: {}", e)
            })));
        }
    };

    // Get counts for additional information
    let references_count = database
        .get_entity_references_count(entity_id)
        .await
        .unwrap_or(0) as u32;
    let items_count = database
        .get_entity_items_count(entity_id)
        .await
        .unwrap_or(0) as u32;
    let relations = database
        .get_relations_for_entity(entity_id)
        .await
        .unwrap_or_default();
    let relations_count = relations.len() as u32;

    let response = EntityDetailResponse {
        entity,
        references_count,
        items_count,
        relations_count,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_entity_references(
    data: AppState,
    path: web::Path<i64>,
    req: web::Query<GetEntityReferencesRequest>,
) -> Result<HttpResponse> {
    // Check if database exists
    let database = match &data.database {
        Some(db) => db,
        None => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": "Database not initialized. Please set a data folder first."
            })));
        }
    };

    let entity_id = path.into_inner();
    let page = req.page.unwrap_or(1);
    let limit = req.limit.unwrap_or(50);

    // Ensure page is at least 1
    let page = if page < 1 { 1 } else { page };

    // Calculate offset
    let offset = (page - 1) * limit;

    // Get total count for pagination
    let total_count = match database.get_entity_references_count(entity_id).await {
        Ok(count) => count as u32,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to get references count: {}", e)
            })));
        }
    };

    // Get paginated references
    let references = match database
        .get_entity_references(entity_id, limit as i64, offset as i64)
        .await
    {
        Ok(references) => references,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch entity references: {}", e)
            })));
        }
    };

    // Calculate total pages
    let total_pages = if total_count > 0 {
        total_count.div_ceil(limit)
    } else {
        0
    };

    let response = GetEntityReferencesResponse {
        references,
        total_count,
        page,
        limit,
        total_pages,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_entity_items(
    data: AppState,
    path: web::Path<i64>,
    req: web::Query<GetEntityItemsRequest>,
) -> Result<HttpResponse> {
    // Check if database exists
    let database = match &data.database {
        Some(db) => db,
        None => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": "Database not initialized. Please set a data folder first."
            })));
        }
    };

    let entity_id = path.into_inner();
    let page = req.page.unwrap_or(1);
    let limit = req.limit.unwrap_or(20);

    // Ensure page is at least 1
    let page = if page < 1 { 1 } else { page };

    // Calculate offset
    let offset = (page - 1) * limit;

    // Get total count for pagination
    let total_count = match database.get_entity_items_count(entity_id).await {
        Ok(count) => count as u32,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to get items count: {}", e)
            })));
        }
    };

    // Get paginated items with highlights
    let items_with_highlights = match database
        .get_entity_items_with_highlights(entity_id, limit as i64, offset as i64)
        .await
    {
        Ok(items) => items,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch entity items: {}", e)
            })));
        }
    };

    // Convert to response format
    let items: Vec<EntityItemWithHighlights> = items_with_highlights
        .into_iter()
        .map(|(item, highlights)| EntityItemWithHighlights { item, highlights })
        .collect();

    // Calculate total pages
    let total_pages = if total_count > 0 {
        total_count.div_ceil(limit)
    } else {
        0
    };

    let response = GetEntityItemsResponse {
        items,
        total_count,
        page,
        limit,
        total_pages,
    };

    Ok(HttpResponse::Ok().json(response))
}

// New tool validation and schema endpoints

/// Get all available tools
#[allow(dead_code)]
pub async fn get_tools(data: AppState) -> Result<HttpResponse> {
    let tool_registry = data.tool_registry.lock().unwrap();
    let tools = tool_registry.get_all_descriptors();

    let response = GetToolsResponse { tools };
    Ok(HttpResponse::Ok().json(response))
}

/// Get JSON schema for a specific tool
#[allow(dead_code)]
pub async fn get_tool_schema_specific(
    data: AppState,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let tool_name = path.into_inner();
    let tool_registry = data.tool_registry.lock().unwrap();

    if let Some(tool) = tool_registry.get_tool(&tool_name) {
        let descriptor = tool.describe();

        let schema = types::ToolSchema {
            name: descriptor.name.clone(),
            parameter_schema: descriptor.parameters.json_schema,
            response_schema: serde_json::json!({}), // TODO: Add response schema
            examples: descriptor
                .examples
                .iter()
                .map(|ex| types::ToolSchemaExample {
                    name: ex.description.clone(),
                    description: ex.use_case.clone(),
                    parameters: ex.input.clone(),
                    expected_response: ex.expected_output.clone().unwrap_or(serde_json::json!({})),
                })
                .collect(),
        };

        let response = GetToolSchemaResponse { schema };
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "error": format!("Tool '{}' not found", tool_name)
        })))
    }
}

/// Validate tool parameters without executing
#[allow(dead_code)]
pub async fn validate_tool_params(
    data: AppState,
    req: web::Json<ValidateToolParamsRequest>,
) -> Result<HttpResponse> {
    let tool_registry = data.tool_registry.lock().unwrap();

    if let Some(tool) = tool_registry.get_tool(&req.tool_name) {
        // Use the tool's validation if it implements ToolValidator
        let validation_result = match tool {
            crate::tools::Tool::SearchItems(search_tool) => {
                search_tool.validate_parameters(&req.parameters)
            }
            crate::tools::Tool::FilterItems(filter_tool) => {
                filter_tool.validate_parameters(&req.parameters)
            }
            crate::tools::Tool::SearchEntities(search_entities_tool) => {
                search_entities_tool.validate_parameters(&req.parameters)
            }
            crate::tools::Tool::ExploreRelations(explore_relations_tool) => {
                explore_relations_tool.validate_parameters(&req.parameters)
            }
        };

        let response = ValidateToolParamsResponse { validation_result };
        Ok(HttpResponse::Ok().json(response))
    } else {
        let validation_result = types::ValidationResult {
            is_valid: false,
            errors: vec![ValidationError {
                field: "tool_name".to_string(),
                error_type: "not_found".to_string(),
                message: format!("Tool '{}' not found", req.tool_name),
                expected: Some("Valid tool name".to_string()),
                actual: Some(req.tool_name.clone()),
            }],
            warnings: vec![],
        };

        let response = ValidateToolParamsResponse { validation_result };
        Ok(HttpResponse::BadRequest().json(response))
    }
}

/// Enhanced tool execution with better validation
#[allow(dead_code)]
pub async fn execute_tool_enhanced(
    data: AppState,
    req: web::Json<ExecuteToolRequestNew>,
) -> Result<HttpResponse> {
    // Clone the tool to avoid holding the lock across the await
    let tool = {
        let tool_registry = data.tool_registry.lock().unwrap();
        tool_registry.get_tool_cloned(&req.tool_name)
    };

    let result = if let Some(tool) = tool {
        Some(tool.execute(req.arguments.clone()).await)
    } else {
        None
    };

    if let Some(tool_result) = result {
        let response = ExecuteToolResponse {
            result: tool_result,
        };
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "success": false,
            "error": format!("Tool '{}' not found", req.tool_name)
        })))
    }
}

// Tool request/response types (avoiding duplicates with existing ones)
#[allow(dead_code)]
#[derive(Serialize, TS)]
#[ts(export)]
pub struct GetToolsResponse {
    pub tools: Vec<ToolDescriptor>,
}

#[allow(dead_code)]
#[derive(Serialize, TS)]
#[ts(export)]
pub struct GetToolSchemaResponse {
    pub schema: types::ToolSchema,
}

#[allow(dead_code)]
#[derive(Deserialize, TS)]
#[ts(export)]
pub struct ValidateToolParamsRequest {
    pub tool_name: String,
    #[ts(type = "Record<string, unknown>")]
    pub parameters: serde_json::Value,
}

#[allow(dead_code)]
#[derive(Serialize, TS)]
#[ts(export)]
pub struct ValidateToolParamsResponse {
    pub validation_result: types::ValidationResult,
}

#[allow(dead_code)]
#[derive(Deserialize, TS)]
#[ts(export)]
pub struct ExecuteToolRequestNew {
    pub tool_name: String,
    pub arguments: ToolArguments,
}

#[allow(dead_code)]
#[derive(Serialize, TS)]
#[ts(export)]
pub struct ExecuteToolResponse {
    pub result: crate::tools::ToolResult,
}
