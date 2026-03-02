use crate::db::Database;
use crate::history::HistoryManager;
use anyhow::Result;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct StasherDaemon {
    history: Arc<HistoryManager>,
    base_path: PathBuf,
}

impl StasherDaemon {
    pub async fn new(db: Database, base_path: PathBuf) -> Result<Self> {
        let history = HistoryManager::new(Arc::new(db), base_path.clone()).await?;
        Ok(Self {
            history: Arc::new(history),
            base_path,
        })
    }

    pub async fn run(&self) -> Result<()> {
        let (tx, mut rx) = mpsc::channel(100);

        // Notify requires a synchronous callback, so we use a channel to bridge to async
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            Config::default(),
        )?;

        watcher.watch(&self.base_path, RecursiveMode::Recursive)?;

        println!("👀 Monitoring changes in: {:?}", self.base_path);

        while let Some(event) = rx.recv().await {
            self.handle_event(event).await?;
        }

        Ok(())
    }

    async fn handle_event(&self, event: notify::Event) -> Result<()> {
        // We only care about data modifications (file saves)
        if event.kind.is_modify() {
            for path in event.paths {
                if self.should_watch(&path) {
                    self.process_file_change(path).await?;
                }
            }
        }
        Ok(())
    }

    fn should_watch(&self, path: &Path) -> bool {
        path.is_file() && self.history.should_index(path)
    }

    async fn process_file_change(&self, path: PathBuf) -> Result<()> {
        let relative_path = path.strip_prefix(&self.base_path)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        println!("📝 Snapshotting: {}", relative_path);
        
        if let Err(e) = self.history.record_change(path).await {
            eprintln!("❌ Failed to record change for {}: {}", relative_path, e);
        }
        
        Ok(())
    }
}
