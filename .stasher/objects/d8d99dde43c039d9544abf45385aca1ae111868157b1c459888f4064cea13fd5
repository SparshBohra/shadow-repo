use anyhow::{Context, Result};
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{Table, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use arrow_array::{RecordBatch, StringArray, RecordBatchIterator};
use arrow_schema::{DataType, Field, Schema};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRecord {
    pub snapshot_id: String,
    pub file_path: String,
    pub content: String,
    pub vector: Vec<f32>,
}

pub struct SearchEngine {
    model: tokio::sync::Mutex<TextEmbedding>,
    lancedb: Connection,
}

impl SearchEngine {
    pub async fn new(lancedb: Connection) -> Result<Self> {
        let mut options = InitOptions::default();
        options.model_name = EmbeddingModel::NomicEmbedTextV15;
        options.show_download_progress = true;
        
        let model = TextEmbedding::try_new(options)?;

        Ok(Self {
            model: tokio::sync::Mutex::new(model),
            lancedb,
        })
    }

    pub async fn index_snapshot(&self, snapshot_id: String, file_path: String, content: String) -> Result<()> {
        let embeddings = {
            let mut model = self.model.lock().await;
            model.embed(vec![content.clone()], None)?
        };
        let vector = embeddings[0].clone();

        let schema = Arc::new(Schema::new(vec![
            Field::new("snapshot_id", DataType::Utf8, false),
            Field::new("file_path", DataType::Utf8, false),
            Field::new("content", DataType::Utf8, false),
            Field::new("vector", DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 768), false),
        ]));

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec![snapshot_id])),
                Arc::new(StringArray::from(vec![file_path])),
                Arc::new(StringArray::from(vec![content])),
                Arc::new(arrow_array::FixedSizeListArray::from_iter_primitive::<arrow_array::types::Float32Type, _, _>(
                    vec![Some(vector.into_iter().map(Some).collect::<Vec<_>>())],
                    768
                )),
            ],
        )?;

        let table_names: Vec<String> = self.lancedb.table_names().execute().await?;
        let batches = RecordBatchIterator::new(vec![Ok(batch)], schema.clone());

        if table_names.contains(&"snapshots_v1".to_string()) {
            let table = self.lancedb.open_table("snapshots_v1").execute().await?;
            table.add(batches).execute().await?;
        } else {
            self.lancedb
                .create_table("snapshots_v1", batches)
                .execute()
                .await?;
        }

        Ok(())
    }

    pub async fn search(&self, query: String, limit: usize) -> Result<Vec<SearchRecord>> {
        use futures_util::StreamExt;
        let query_vec = {
            let mut model = self.model.lock().await;
            model.embed(vec![query], None)?[0].clone()
        };
        
        let table = self.lancedb.open_table("snapshots_v1").execute().await?;
        let mut results = table
            .vector_search(query_vec)?
            .limit(limit)
            .execute()
                .await?;
            
        let mut records = Vec::new();
        while let Some(batch) = results.next().await {
            let batch = batch?;
            let snapshot_ids = batch.column_by_name("snapshot_id")
                .context("Missing snapshot_id column")?
                .as_any().downcast_ref::<StringArray>()
                .context("Failed to downcast snapshot_id")?;
            
            let file_paths = batch.column_by_name("file_path")
                .context("Missing file_path column")?
                .as_any().downcast_ref::<StringArray>()
                .context("Failed to downcast file_path")?;
            
            let contents = batch.column_by_name("content")
                .context("Missing content column")?
                .as_any().downcast_ref::<StringArray>()
                .context("Failed to downcast content")?;

            for i in 0..batch.num_rows() {
                records.push(SearchRecord {
                    snapshot_id: snapshot_ids.value(i).to_string(),
                    file_path: file_paths.value(i).to_string(),
                    content: contents.value(i).to_string(),
                    vector: vec![],
                });
            }
        }
            
        Ok(records)
    }
}
