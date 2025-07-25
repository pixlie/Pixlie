use clap::Parser;

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
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("🚀 Pixlie Data Analyzer v0.1.0");
    println!();

    if let Some(database) = &args.database {
        println!("📊 Database: {}", database);
    } else {
        println!("📊 No database specified");
    }

    if let Some(objective) = &args.objective {
        println!("🎯 Objective: {}", objective);
    } else {
        println!("🎯 No objective specified");
    }

    println!("🤖 Model: {}", args.model);
    println!("🔄 Max iterations: {}", args.max_iterations);
    println!();
    println!("Hello, World! 🌍");
    println!();
    println!("Ready to analyze your data with AI! 🔍✨");
}