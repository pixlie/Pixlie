use clap::Parser;
use pixlie::{LoggingConfig, init_logging, log_error, ErrorContext, PixlieError, Result, ErrorSeverity, ConfigManager, CliArgs};
use std::process;
use tracing::{info, debug};

#[derive(Parser)]
#[command(name = "data-analyzer")]
#[command(about = "LLM-enabled CLI data analysis tool for SQLite databases")]
#[command(version = "0.1.0")]
pub struct Args {
    /// Path to SQLite database file
    #[arg(short, long)]
    pub database: Option<String>,

    /// Analysis objective or question
    #[arg(short, long)]
    pub objective: Option<String>,

    /// LLM model to use
    #[arg(short, long, default_value = "gpt-3.5-turbo")]
    pub model: String,

    /// Maximum number of iterations
    #[arg(long, default_value = "10")]
    pub max_iterations: u32,

    /// Workspace path for project-specific settings
    #[arg(short, long)]
    pub workspace: Option<String>,

    /// Enable JSON logging format
    #[arg(long)]
    pub json_logs: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Create default configuration files
    #[arg(long)]
    pub create_config: bool,

    /// Show effective configuration and exit
    #[arg(long)]
    pub show_config: bool,
}

impl CliArgs for Args {
    fn workspace(&self) -> Option<&str> {
        self.workspace.as_deref()
    }
    
    fn log_level(&self) -> &str {
        &self.log_level
    }
    
    fn json_logs(&self) -> bool {
        self.json_logs
    }
    
    fn model(&self) -> &str {
        &self.model
    }
    
    fn max_iterations(&self) -> u32 {
        self.max_iterations
    }
}

async fn run_application(args: Args) -> Result<()> {
    let context = ErrorContext::new().with_context("Application startup");

    info!("🚀 Pixlie Data Analyzer v0.1.0");

    // Initialize configuration manager
    let mut config_manager = ConfigManager::new()?;
    
    // Handle create-config flag
    if args.create_config {
        return handle_create_config(&args).await;
    }

    // Load configuration from all sources
    config_manager.load(&args).await?;

    // Handle show-config flag
    if args.show_config {
        return handle_show_config(&config_manager).await;
    }

    // Get effective configuration
    let ui_config = config_manager.effective_ui_config();
    let llm_config = config_manager.effective_llm_config();
    let database_config = config_manager.effective_database_config();
    let _session_config = config_manager.effective_session_config();

    debug!("Effective configuration loaded");
    debug!("UI theme: {}", ui_config.theme);
    debug!("LLM model: {}", llm_config.default_model);
    debug!("Database read-only: {}", database_config.read_only);

    // Validate database path if provided
    if let Some(database) = &args.database {
        info!(database = database, "📊 Database specified");
        
        // Check if file exists
        if !std::path::Path::new(database).exists() {
            return Err(PixlieError::validation(
                "database",
                "Database file does not exist",
                context,
            ));
        }
    } else if let Some(workspace) = &args.workspace {
        // Check for default database in workspace metadata
        if let Some(workspace_config) = &config_manager.workspace {
            if let Some(default_db) = &workspace_config.metadata.default_database {
                let db_path = std::path::Path::new(workspace).join(default_db);
                if db_path.exists() {
                    info!(database = %db_path.display(), "📊 Using workspace default database");
                } else {
                    info!("📊 Workspace default database not found: {:?}", db_path);
                }
            }
        }
        info!("📊 No database specified");
    } else {
        info!("📊 No database specified");
    }

    // Validate objective
    if let Some(objective) = &args.objective {
        info!(objective = objective, "🎯 Objective specified");
        
        if objective.trim().is_empty() {
            return Err(PixlieError::validation(
                "objective",
                "Objective cannot be empty",
                context,
            ));
        }
    } else {
        info!("🎯 No objective specified");
    }

    // Display effective configuration values
    info!(model = llm_config.default_model, "🤖 Model selected");
    info!(max_iterations = llm_config.max_iterations, "🔄 Max iterations set");
    info!(theme = ui_config.theme, "🎨 UI theme");
    info!(layout = ui_config.layout, "📐 UI layout");
    
    if let Some(workspace) = &args.workspace {
        info!(workspace = workspace, "📁 Workspace");
        
        if let Some(workspace_config) = &config_manager.workspace {
            if let Some(name) = &workspace_config.metadata.name {
                info!(workspace_name = name, "📁 Workspace name");
            }
            
            let pinned_count = workspace_config.workspace.pinned_objectives.len();
            if pinned_count > 0 {
                info!(pinned_objectives = pinned_count, "📌 Pinned objectives");
            }
        }
    }

    info!("Hello, World! 🌍");
    info!("Ready to analyze your data with AI! 🔍✨");

    Ok(())
}

async fn handle_create_config(args: &Args) -> Result<()> {
    use pixlie::ConfigLoader;
    
    let loader = ConfigLoader::new()?;
    
    info!("Creating default configuration files...");
    
    // Create global config if it doesn't exist
    if !loader.global_config_exists() {
        loader.create_default_global_config().await?;
        info!("✅ Created global configuration at: {:?}", loader.paths().global_config);
    } else {
        info!("ℹ️  Global configuration already exists at: {:?}", loader.paths().global_config);
    }
    
    // Create workspace config if workspace is specified
    if let Some(workspace_path) = &args.workspace {
        if !loader.workspace_config_exists(workspace_path) {
            // Extract workspace name from path
            let workspace_name = std::path::Path::new(workspace_path)
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string());
                
            loader.create_default_workspace_config(workspace_path, workspace_name).await?;
            info!("✅ Created workspace configuration at: {}", 
                std::path::Path::new(workspace_path).join(".pixlie-workspace.toml").display());
        } else {
            info!("ℹ️  Workspace configuration already exists at: {}",
                std::path::Path::new(workspace_path).join(".pixlie-workspace.toml").display());
        }
    }
    
    info!("Configuration creation complete!");
    Ok(())
}

async fn handle_show_config(config_manager: &ConfigManager) -> Result<()> {
    info!("Current effective configuration:");
    
    let global_toml = toml::to_string_pretty(&config_manager.global)
        .map_err(|e| PixlieError::configuration(
            format!("Failed to serialize configuration: {}", e),
            ErrorContext::new().with_context("Configuration serialization"),
        ))?;
    
    println!("\n=== Global Configuration ===");
    println!("{}", global_toml);
    
    if let Some(workspace) = &config_manager.workspace {
        let workspace_toml = toml::to_string_pretty(workspace)
            .map_err(|e| PixlieError::configuration(
                format!("Failed to serialize workspace configuration: {}", e),
                ErrorContext::new().with_context("Workspace configuration serialization"),
            ))?;
        
        println!("\n=== Workspace Configuration ===");
        println!("{}", workspace_toml);
    }
    
    println!("\n=== Configuration File Paths ===");
    println!("Global config: {:?}", config_manager.paths.global_config);
    if let Some(workspace_config) = &config_manager.paths.workspace_config {
        println!("Workspace config: {:?}", workspace_config);
    }
    
    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize logging system
    let logging_config = LoggingConfig {
        json_format: args.json_logs,
        level: args.log_level.clone(),
        colored: !args.json_logs, // Disable colors for JSON format
        file_path: None,
    };

    if let Err(e) = init_logging(logging_config) {
        eprintln!("Failed to initialize logging: {}", e);
        process::exit(1);
    }

    // Run the application and handle errors
    if let Err(e) = run_application(args).await {
        log_error(&e);
        
        // Also print user-friendly error to stderr
        eprintln!("Error: {}", e.user_message());
        
        // Exit with error code based on severity
        let exit_code = match e.severity() {
            ErrorSeverity::Low => 1,
            ErrorSeverity::Medium => 2,
            ErrorSeverity::High => 3,
            ErrorSeverity::Critical => 4,
        };
        
        process::exit(exit_code);
    }
}