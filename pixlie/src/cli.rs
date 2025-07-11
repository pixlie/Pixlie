use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "pixlie")]
#[command(about = "Smart Entity Analysis for Hacker News Discussions")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show configuration file path
    Config,
    /// Start the web server
    Server {
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}
