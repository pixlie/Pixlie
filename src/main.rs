use clap::Parser;
use pixlie::{LoggingConfig, init_logging, log_error, ErrorContext, PixlieError, Result, ErrorSeverity, ConfigManager};
use std::process;
use tracing::{info, debug};

#[derive(Parser)]
#[command(name = "pixlie")]
#[command(about = "LLM-enabled TUI data analysis tool for SQLite databases")]
#[command(version = "0.1.0")]
pub struct Args {
    /// Optional workspace path to open on startup
    #[arg(value_name = "WORKSPACE")]
    pub workspace: Option<String>,
}


async fn run_application(args: Args) -> Result<()> {
    let _context = ErrorContext::new().with_context("Application startup");

    info!("🚀 Pixlie TUI Data Analyzer v0.1.0");

    // Initialize configuration manager
    let mut config_manager = ConfigManager::new()?;
    
    // Load workspace if specified
    if let Some(workspace_path) = &args.workspace {
        info!(workspace = workspace_path, "📁 Loading specified workspace");
        load_workspace_configuration(&mut config_manager, workspace_path).await?;
    } else {
        // Auto-detect workspace in current directory
        if let Some(detected_workspace) = detect_workspace_in_current_dir().await? {
            info!(workspace = detected_workspace, "📁 Auto-detected workspace");
            load_workspace_configuration(&mut config_manager, &detected_workspace).await?;
        } else {
            info!("📁 No workspace detected, using default configuration");
        }
    }

    // Load configuration from environment and files (no CLI args to override)
    load_basic_configuration(&mut config_manager).await?;

    // Get effective configuration for TUI startup
    let ui_config = config_manager.effective_ui_config();
    let _session_config = config_manager.effective_session_config();

    debug!("Configuration loaded for TUI startup");
    debug!("UI theme: {}", ui_config.theme);
    debug!("UI layout: {}", ui_config.layout);

    // Display startup information
    if let Some(workspace) = &args.workspace {
        info!(workspace = workspace, "📁 Workspace");
        
        if let Some(workspace_config) = &config_manager.workspace {
            if let Some(name) = &workspace_config.metadata.name {
                info!(workspace_name = name, "📁 Workspace name");
            }
            
            let pinned_count = workspace_config.workspace.pinned_objectives.len();
            if pinned_count > 0 {
                info!(pinned_objectives = pinned_count, "📌 Pinned objectives available");
            }
        }
    }

    info!("🚀 Starting TUI interface...");
    
    // TODO: Launch TUI interface here
    // For now, we'll show a placeholder message
    println!("TUI interface would launch here!");
    println!("Theme: {}", ui_config.theme);
    println!("Layout: {}", ui_config.layout);
    if let Some(workspace) = &config_manager.workspace {
        if let Some(name) = &workspace.metadata.name {
            println!("Workspace: {}", name);
        }
    }

    Ok(())
}

/// Load workspace configuration if workspace exists
async fn load_workspace_configuration(config_manager: &mut ConfigManager, workspace_path: &str) -> Result<()> {
    use pixlie::ConfigLoader;
    
    let loader = ConfigLoader::new()?;
    
    // Check if workspace configuration exists
    if loader.workspace_config_exists(workspace_path) {
        debug!("Loading workspace configuration from: {}", workspace_path);
        if let Some(workspace_config) = loader.load_workspace_config(workspace_path).await? {
            config_manager.workspace = Some(workspace_config);
            config_manager.paths.workspace_config = Some(
                std::path::PathBuf::from(workspace_path).join(".pixlie-workspace.toml")
            );
        }
    } else {
        debug!("No workspace configuration found at: {}", workspace_path);
    }
    
    Ok(())
}

/// Load basic configuration (global config + environment variables)
async fn load_basic_configuration(config_manager: &mut ConfigManager) -> Result<()> {
    use pixlie::ConfigLoader;
    
    let loader = ConfigLoader::new()?;
    
    // Load global configuration
    config_manager.global = loader.load_global_config().await?;
    
    // Apply environment variable overrides
    loader.apply_environment_overrides(&mut config_manager.global)?;
    
    debug!("Basic configuration loaded");
    Ok(())
}

/// Detect if current directory or parent directories contain a workspace
async fn detect_workspace_in_current_dir() -> Result<Option<String>> {
    let current_dir = std::env::current_dir()
        .map_err(|e| PixlieError::session(
            format!("Failed to get current directory: {}", e),
            ErrorContext::new().with_context("Workspace detection"),
        ))?;
    
    // Look for .pixlie-workspace.toml in current directory and parents
    let mut dir = current_dir.as_path();
    
    loop {
        let workspace_config = dir.join(".pixlie-workspace.toml");
        if workspace_config.exists() {
            debug!("Found workspace configuration at: {:?}", dir);
            return Ok(Some(dir.to_string_lossy().to_string()));
        }
        
        // Move to parent directory
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break, // Reached filesystem root
        }
    }
    
    debug!("No workspace configuration found in current directory or parents");
    Ok(None)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize basic logging system with defaults
    // The configuration system will be loaded inside run_application
    let logging_config = LoggingConfig {
        json_format: false,
        level: "info".to_string(),
        colored: true,
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