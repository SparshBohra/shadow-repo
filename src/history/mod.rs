use crate::db::Database;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::fs;
use chrono::Utc;
use uuid::Uuid;
use similar::{TextDiff, ChangeTag};
use std::sync::Arc;
use crate::search::SearchEngine;

pub struct HistoryManager {
    db: Arc<Database>,
    search: Arc<SearchEngine>,
    base_path: PathBuf,
    objects_path: PathBuf,
    current_session_id: Uuid,
}

impl HistoryManager {
    pub async fn new(db: Arc<Database>, base_path: PathBuf) -> Result<Self> {
        let objects_path = base_path.join(".stasher").join("objects");
        
        // Start a new session for the daemon run
        let session_id = Uuid::new_v4();
        let now = Utc::now().timestamp_millis();
        
        sqlx::query("INSERT INTO sessions (id, start_time) VALUES (?, ?)")
            .bind(session_id.to_string())
            .bind(now)
            .execute(&db.sqlite)
            .await?;

        // Initialize search engine
        let search = Arc::new(SearchEngine::new(db.lancedb.clone()).await?);

        Ok(Self {
            db,
            search,
            base_path,
            objects_path,
            current_session_id: session_id,
        })
    }

    pub async fn record_change(&self, file_path: PathBuf) -> Result<()> {
        let relative_path = file_path.strip_prefix(&self.base_path)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .to_string();

        let content = fs::read_to_string(&file_path)
            .context("Failed to read file for record_change")?;
        
        let new_hash = blake3::hash(content.as_bytes()).to_hex().to_string();

        // Check latest snapshot for this file
        let latest: Option<(String, String)> = sqlx::query_as(
            "SELECT content_hash, diff_patch FROM snapshots WHERE file_path = ? ORDER BY timestamp DESC LIMIT 1"
        )
        .bind(&relative_path)
        .fetch_optional(&self.db.sqlite)
        .await?;

        let (diff_patch, added, removed) = if let Some((old_hash, _)) = latest {
            if old_hash == new_hash {
                // No actual content change
                return Ok(());
            }

            // Generate diff
            let old_content_path = self.objects_path.join(&old_hash);
            let old_content = if old_content_path.exists() {
                fs::read_to_string(old_content_path).unwrap_or_default()
            } else {
                String::new()
            };

            let diff = TextDiff::from_lines(&old_content, &content);
            let mut patch = String::new();
            let mut added = 0;
            let mut removed = 0;

            for hunk in diff.unified_diff().header(&relative_path, &relative_path).iter_hunks() {
                patch.push_str(&format!("{}", hunk));
                for change in hunk.iter_changes() {
                    match change.tag() {
                        ChangeTag::Delete => removed += 1,
                        ChangeTag::Insert => added += 1,
                        _ => {}
                    }
                }
            }
            (patch, added, removed)
        } else {
            // First time seeing this file, diff is the whole file
            let patch = format!("--- /dev/null\n+++ {}\n@@ -0,0 +1,{} @@\n{}", 
                relative_path, 
                content.lines().count(),
                content.lines().map(|l| format!("+{}", l)).collect::<Vec<_>>().join("\n")
            );
            (patch, content.lines().count() as i32, 0)
        };

        let snapshot_id = self.save_snapshot(&relative_path, &new_hash, &diff_patch, added, removed).await?;

        // 3. Index for semantic search
        if let Err(e) = self.search.index_snapshot(snapshot_id, relative_path, content.clone()).await {
            eprintln!("⚠️ Failed to index snapshot: {}", e);
        }

        // Save to CAS
        let object_path = self.objects_path.join(&new_hash);
        fs::write(object_path, content)?;

        Ok(())
    }

    async fn save_snapshot(&self, file_path: &str, hash: &str, patch: &str, added: i32, removed: i32) -> Result<String> {
        let snapshot_id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp_millis();

        sqlx::query(
            "INSERT INTO snapshots (id, session_id, file_path, timestamp, diff_patch, content_hash, lines_added, lines_removed) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&snapshot_id)
        .bind(self.current_session_id.to_string())
        .bind(file_path)
        .bind(now)
        .bind(patch)
        .bind(hash)
        .bind(added)
        .bind(removed)
        .execute(&self.db.sqlite)
        .await?;

        Ok(snapshot_id)
    }
}
