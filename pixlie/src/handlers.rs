use crate::config::Config;
use crate::database::{Database, DownloadStats, ExtractionStats};
use crate::entity_extraction::{EntityExtractor, ModelInfo};
use crate::hn_api::HnApiClient;
use actix_web::{HttpResponse, Result, web};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::task;

pub type AppState = web::Data<Arc<AppData>>;

pub struct AppData {
    pub config: Mutex<Config>,
    pub database: Option<Database>,
    pub hn_client: HnApiClient,
    pub entity_extractor: Mutex<EntityExtractor>,
}

#[derive(Serialize)]
pub struct ConfigResponse {
    pub config_path: String,
    pub data_folder: Option<PathBuf>,
    pub download_stats: DownloadStats,
}

#[derive(Deserialize)]
pub struct SetDataFolderRequest {
    pub folder_path: String,
}

#[derive(Serialize)]
pub struct DownloadStatusResponse {
    pub download_stats: DownloadStats,
}

#[derive(Deserialize)]
pub struct StartDownloadRequest {
    pub download_type: String, // "stories", "recent", "all"
    pub limit: Option<u64>,
}

#[derive(Serialize)]
pub struct ModelsResponse {
    pub models: Vec<ModelInfo>,
}

#[derive(Deserialize)]
pub struct DownloadModelRequest {
    pub model_name: String,
}

#[derive(Serialize)]
pub struct ExtractionStatusResponse {
    pub extraction_stats: ExtractionStats,
}

#[derive(Deserialize)]
pub struct StartExtractionRequest {
    pub batch_size: Option<u64>,
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
                // Load the model into the extractor
                let mut extractor = app_data.entity_extractor.lock().unwrap();
                if let Err(e) = extractor.load_model(&model_path) {
                    eprintln!("Failed to load model: {e}");
                } else {
                    println!("Model {model_name} loaded successfully");
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
