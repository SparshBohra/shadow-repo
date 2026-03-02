mod db;
mod daemon;
mod history;
mod search;

use anyhow::Result;
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
    /// Restore a file to a previous version
    Restore { 
        file: String, 
        #[arg(short, long)]
        snapshot: Option<String> 
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let base_path = std::env::current_dir()?;

    match &cli.command {
        Commands::Daemon => {
            println!("🚀 Initializing Stasher database in .stasher/ ...");
            let db = db::Database::init(&base_path).await?;
            println!("💾 Database ready. Starting daemon...");
            
            let daemon = daemon::StasherDaemon::new(db, base_path.to_path_buf()).await?;
            daemon.run().await?;
            
            Ok(())
        }
        Commands::Ask { query } => {
            println!("🔍 Searching for: \"{}\"...", query);
            let db = db::Database::init(&base_path).await?;
            let search = search::SearchEngine::new(db.lancedb.clone()).await?;
            
            let results = search.search(query.clone(), 5).await?;
            
            if results.is_empty() {
                println!("🤷 No relevant history found.");
            } else {
                println!("✨ Found {} relevant snapshots:", results.len());
                for (i, res) in results.iter().enumerate() {
                    println!("\n[{}] File: {}", i + 1, res.file_path);
                    println!("--- Snippet ---");
                    let snippet: String = res.content.lines().take(5).collect::<Vec<_>>().join("\n");
                    println!("{}...", snippet);
                }
            }
            Ok(())
        }
        Commands::Restore { file, snapshot } => {
            println!("⏪ Restoring {}...", file);
            let db = db::Database::init(&base_path).await?;
            let history = history::HistoryManager::new(std::sync::Arc::new(db), base_path.to_path_buf()).await?;
            
            history.restore_file(&file, snapshot.clone()).await?;
            println!("✅ Restore complete.");
            Ok(())
        }
        _ => {
            println!("Command not implemented yet!");
            Ok(())
        }
    }
}
