use clap::Parser;
use pixlie::{LoggingConfig, init_logging, log_error, ErrorContext, PixlieError, Result, ErrorSeverity};
use std::process;
use tracing::info;

#[derive(Parser)]
#[command(name = "data-analyzer")]
#[command(about = "LLM-enabled CLI data analysis tool for SQLite databases")]
#[command(version = "0.1.0")]
struct Args {
    /// Path to SQLite database file
    #[arg(short, long)]
    database: Option<String>,

    /// Analysis objective or question
    #[arg(short, long)]
    objective: Option<String>,

    /// LLM model to use
    #[arg(short, long, default_value = "gpt-3.5-turbo")]
    model: String,

    /// Maximum number of iterations
    #[arg(long, default_value = "10")]
    max_iterations: u32,

    /// Enable JSON logging format
    #[arg(long)]
    json_logs: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

async fn run_application(args: Args) -> Result<()> {
    let context = ErrorContext::new().with_context("Application startup");

    info!("🚀 Pixlie Data Analyzer v0.1.0");

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

    // Validate model name
    if args.model.trim().is_empty() {
        return Err(PixlieError::validation(
            "model",
            "Model name cannot be empty",
            context,
        ));
    }

    // Validate max iterations
    if args.max_iterations == 0 {
        return Err(PixlieError::validation(
            "max_iterations",
            "Max iterations must be greater than 0",
            context,
        ));
    }

    info!(model = args.model, "🤖 Model selected");
    info!(max_iterations = args.max_iterations, "🔄 Max iterations set");
    
    info!("Hello, World! 🌍");
    info!("Ready to analyze your data with AI! 🔍✨");

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