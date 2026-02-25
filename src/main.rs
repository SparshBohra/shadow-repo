mod db;
mod daemon;
mod history;
mod search;

use anyhow::Result;
use std::path::Path;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "stasher")]
#[command(about = "Local-first development history tracker", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the background daemon
    Daemon,
    /// Search history using natural language
    Ask { query: String },
    /// Show history for a file
    Show { file: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let base_path = Path::new(".");

    match &cli.command {
        Commands::Daemon => {
            println!("🚀 Initializing Stasher database in .stasher/ ...");
            let db = db::Database::init(base_path).await?;
            println!("💾 Database ready. Starting daemon...");
            
            let daemon = daemon::StasherDaemon::new(db, base_path.to_path_buf());
            daemon.run().await?;
            
            Ok(())
        }
        _ => {
            println!("Command not implemented yet!");
            Ok(())
        }
    }
}
