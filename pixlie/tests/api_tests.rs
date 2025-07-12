use actix_web::{test, web, App};
use pixlie::handlers::{get_config, set_data_folder, get_models, download_model, start_download, get_download_status, AppData, ConfigResponse, SetDataFolderRequest, ModelsResponse, DownloadModelRequest, StartDownloadRequest, DownloadStatusResponse};
use pixlie::config::Config;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use pixlie::database::Database;
use pixlie::hn_api::HnApiClient;
use pixlie::entity_extraction::EntityExtractor;

#[actix_rt::test]
async fn test_get_config() {
    let config = Config::default();
    let app_data = Arc::new(AppData {
        config: Mutex::new(config),
        database: None,
        hn_client: HnApiClient::new(),
        entity_extractor: Mutex::new(EntityExtractor::new()),
    });
    let app_state = web::Data::new(app_data);

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/api/config", web::get().to(get_config)),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/config").to_request();
    let resp: ConfigResponse = test::call_and_read_body_json(&app, req).await;

    assert!(resp.config_path.ends_with("config.json"));
    assert_eq!(resp.data_folder, None);
}

#[actix_rt::test]
async fn test_set_data_folder() {
    let config = Config::default();
    let app_data = Arc::new(AppData {
        config: Mutex::new(config),
        database: None,
        hn_client: HnApiClient::new(),
        entity_extractor: Mutex::new(EntityExtractor::new()),
    });
    let app_state = web::Data::new(app_data);

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/api/data-folder", web::post().to(set_data_folder)),
    )
    .await;

    let temp_dir = tempdir().unwrap();
    let data_folder_path = temp_dir.path().to_str().unwrap().to_string();

    let req = test::TestRequest::post()
        .uri("/api/data-folder")
        .set_json(&SetDataFolderRequest {
            folder_path: data_folder_path.clone(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let config = app_state.config.lock().unwrap();
    assert_eq!(config.data_folder, Some(data_folder_path.into()));
}

#[actix_rt::test]
async fn test_get_models() {
    let config = Config::default();
    let app_data = Arc::new(AppData {
        config: Mutex::new(config),
        database: None,
        hn_client: HnApiClient::new(),
        entity_extractor: Mutex::new(EntityExtractor::new()),
    });
    let app_state = web::Data::new(app_data);

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/api/models", web::get().to(get_models)),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/models").to_request();
    let resp: ModelsResponse = test::call_and_read_body_json(&app, req).await;

    assert!(!resp.models.is_empty());
}

#[actix_rt::test]
async fn test_download_model() {
    let temp_dir = tempdir().unwrap();
    let data_folder_path = temp_dir.path().to_path_buf();

    let mut config = Config::default();
    config.data_folder = Some(data_folder_path.clone());

    let app_data = Arc::new(AppData {
        config: Mutex::new(config),
        database: None,
        hn_client: HnApiClient::new(),
        entity_extractor: Mutex::new(EntityExtractor::new()),
    });
    let app_state = web::Data::new(app_data);

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/api/models/download", web::post().to(download_model)),
    )
    .await;

    let model_name = "bert-base-cased".to_string();

    let req = test::TestRequest::post()
        .uri("/api/models/download")
        .set_json(&DownloadModelRequest {
            model_name: model_name.clone(),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_get_download_status() {
    let temp_dir = tempdir().unwrap();
    let data_folder_path = temp_dir.path().to_path_buf();
    let db_path = data_folder_path.join("hackernews.db");

    let mut config = Config::default();
    config.data_folder = Some(data_folder_path.clone());

    let database = Database::new(&db_path).await.unwrap();

    let app_data = Arc::new(AppData {
        config: Mutex::new(config),
        database: Some(database),
        hn_client: HnApiClient::new(),
        entity_extractor: Mutex::new(EntityExtractor::new()),
    });
    let app_state = web::Data::new(app_data);

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/api/download/status", web::get().to(get_download_status)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/download/status")
        .to_request();

    let resp: DownloadStatusResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp.download_stats.is_downloading, false);
}

#[actix_rt::test]
async fn test_start_download() {
    let temp_dir = tempdir().unwrap();
    let data_folder_path = temp_dir.path().to_path_buf();
    let db_path = data_folder_path.join("hackernews.db");

    let mut config = Config::default();
    config.data_folder = Some(data_folder_path.clone());

    let database = Database::new(&db_path).await.unwrap();

    let app_data = Arc::new(AppData {
        config: Mutex::new(config),
        database: Some(database),
        hn_client: HnApiClient::new(),
        entity_extractor: Mutex::new(EntityExtractor::new()),
    });
    let app_state = web::Data::new(app_data);

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/api/download/start", web::post().to(start_download)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/download/start")
        .set_json(&StartDownloadRequest {
            download_type: "stories".to_string(),
            limit: Some(1),
        })
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
