mod cli;
mod config;
mod database;
mod entity_extraction;
mod handlers;
mod hn_api;
mod llm;
mod tools;

use actix_web::{App, HttpServer, middleware::Logger, web};
use clap::Parser;

use cli::{Cli, Commands};
use config::Config;
use database::Database;
use entity_extraction::EntityExtractor;
use handlers::{
    AppData, download_model, execute_tool, get_config, get_download_status, get_entities,
    get_entity_detail, get_entity_items, get_entity_references, get_extraction_status, get_items,
    get_llm_conversation, get_models, get_relations, get_tool_descriptor, get_tool_metrics,
    get_tool_schema, get_tools_by_category, list_llm_tools, llm_query, search_entities,
    set_data_folder, start_download, start_extraction, stop_download, stop_extraction,
};
use hn_api::HnApiClient;
use std::sync::{Arc, Mutex};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Config) => {
            match Config::get_config_path() {
                Ok(path) => println!("Config file path: {}", path.display()),
                Err(e) => eprintln!("Error getting config path: {e}"),
            }
            return Ok(());
        }
        Some(Commands::Server { port }) => start_server(port).await,
        None => {
            // Default to starting server on port 8080
            start_server(8080).await
        }
    }
}

async fn start_server(port: u16) -> std::io::Result<()> {
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("Failed to load config: {e}, using default");
        Config::default()
    });

    let hn_client = HnApiClient::new();

    // Initialize database if data folder is set
    let database = if let Some(ref data_folder) = config.data_folder {
        let db_path = data_folder.join("hackernews.db");
        match Database::new(&db_path).await {
            Ok(db) => {
                println!("Database initialized at: {}", db_path.display());
                Some(db)
            }
            Err(e) => {
                eprintln!("Failed to initialize database: {e}");
                None
            }
        }
    } else {
        None
    };

    // Initialize entity extractor and check for existing models
    let mut entity_extractor = EntityExtractor::new();
    if let Some(ref data_folder) = config.data_folder {
        let models_dir = data_folder.join("models");
        let models = EntityExtractor::get_available_models_with_status(&models_dir);

        // Try to load the first downloaded model
        if let Some(downloaded_model) = models.iter().find(|m| m.is_downloaded) {
            if let Some(ref model_path) = downloaded_model.local_path {
                if let Err(e) = entity_extractor.load_model(model_path) {
                    eprintln!(
                        "Failed to load existing model {}: {e}",
                        downloaded_model.name
                    );
                } else {
                    println!("Loaded existing model: {}", downloaded_model.name);
                }
            }
        }
    }

    // Initialize tool registry with available tools
    let mut tool_registry = tools::ToolRegistry::new();
    tool_registry.register(tools::Tool::SearchItems(
        tools::data_query::SearchItemsTool::new(),
    ));
    tool_registry.register(tools::Tool::FilterItems(
        tools::data_query::FilterItemsTool::new(),
    ));
    tool_registry.register(tools::Tool::SearchEntities(
        tools::entity_analysis::SearchEntitiesTool::new(),
    ));
    tool_registry.register(tools::Tool::ExploreRelations(
        tools::relation_exploration::ExploreRelationsTool::new(),
    ));

    let app_data = Arc::new(AppData {
        config: Mutex::new(config),
        database,
        hn_client,
        entity_extractor: Mutex::new(entity_extractor),
        tool_registry: Mutex::new(tool_registry),
    });
    let app_state = web::Data::new(app_data);

    println!("Starting server on http://localhost:{port}");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .service(
                web::scope("/api")
                    .route("/config", web::get().to(get_config))
                    .route("/data-folder", web::post().to(set_data_folder))
                    .route("/download/start", web::post().to(start_download))
                    .route("/download/stop", web::post().to(stop_download))
                    .route("/download/status", web::get().to(get_download_status))
                    .route("/models", web::get().to(get_models))
                    .route("/models/download", web::post().to(download_model))
                    .route("/extraction/start", web::post().to(start_extraction))
                    .route("/extraction/stop", web::post().to(stop_extraction))
                    .route("/extraction/status", web::get().to(get_extraction_status))
                    .route("/items", web::get().to(get_items))
                    .route("/entities", web::get().to(get_entities))
                    .route("/entities/search", web::get().to(search_entities))
                    .route("/entities/{id}", web::get().to(get_entity_detail))
                    .route(
                        "/entities/{id}/references",
                        web::get().to(get_entity_references),
                    )
                    .route("/entities/{id}/items", web::get().to(get_entity_items))
                    .route("/relations", web::get().to(get_relations))
                    .route("/llm/query", web::post().to(llm_query))
                    .route("/llm/tools", web::get().to(list_llm_tools))
                    .route("/llm/conversation", web::get().to(get_llm_conversation))
                    // Tool management endpoints
                    .route("/tools", web::get().to(list_llm_tools))
                    .route("/tools/schema", web::get().to(get_tool_schema))
                    .route("/tools/{category}", web::get().to(get_tools_by_category))
                    .route("/tools/{name}/describe", web::get().to(get_tool_descriptor))
                    .route("/tools/{name}/execute", web::post().to(execute_tool))
                    .route("/tools/{name}/metrics", web::get().to(get_tool_metrics)),
            )
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
