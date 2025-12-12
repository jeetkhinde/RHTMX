//! PostgreSQL storage backend for Automerge documents

use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use super::DocumentStorage;
use crate::error::{MergeError, MergeResult};

/// PostgreSQL storage for Automerge documents
pub struct PostgresStorage {
    pool: PgPool,
}

impl PostgresStorage {
    /// Create a new PostgreSQL storage
    pub async fn new(database_url: &str) -> MergeResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .map_err(|e| MergeError::Database(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Get the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Save document with optional metadata
    pub async fn save_document_with_meta(
        &self,
        entity_type: &str,
        data: &[u8],
        heads: &[String],
        change_count: usize,
    ) -> MergeResult<()> {
        let heads_json = serde_json::to_value(heads)
            .map_err(|e| MergeError::Serialization(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO _merge_documents (entity_type, data, heads, change_count, updated_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (entity_type) DO UPDATE SET
                data = EXCLUDED.data,
                heads = EXCLUDED.heads,
                change_count = EXCLUDED.change_count,
                updated_at = NOW()
            "#,
        )
        .bind(entity_type)
        .bind(data)
        .bind(heads_json)
        .bind(change_count as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| MergeError::Database(e.to_string()))?;

        Ok(())
    }

    /// Load document with metadata
    pub async fn load_document_with_meta(
        &self,
        entity_type: &str,
    ) -> MergeResult<Option<(Vec<u8>, Vec<String>, i64)>> {
        let result = sqlx::query_as::<_, (Vec<u8>, serde_json::Value, i64)>(
            r#"
            SELECT data, heads, change_count
            FROM _merge_documents
            WHERE entity_type = $1
            "#,
        )
        .bind(entity_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| MergeError::Database(e.to_string()))?;

        match result {
            Some((data, heads_json, count)) => {
                let heads: Vec<String> = serde_json::from_value(heads_json)
                    .map_err(|e| MergeError::Serialization(e.to_string()))?;
                Ok(Some((data, heads, count)))
            }
            None => Ok(None),
        }
    }

    /// Store a change log entry (for audit/debugging)
    pub async fn log_change(
        &self,
        entity_type: &str,
        entity_id: &str,
        change_type: &str,
        change_hash: &str,
        actor_id: &str,
    ) -> MergeResult<()> {
        sqlx::query(
            r#"
            INSERT INTO _merge_change_log
            (entity_type, entity_id, change_type, change_hash, actor_id, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            "#,
        )
        .bind(entity_type)
        .bind(entity_id)
        .bind(change_type)
        .bind(change_hash)
        .bind(actor_id)
        .execute(&self.pool)
        .await
        .map_err(|e| MergeError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get recent changes for an entity type
    pub async fn get_recent_changes(
        &self,
        entity_type: &str,
        limit: i64,
    ) -> MergeResult<Vec<ChangeLogEntry>> {
        let entries = sqlx::query_as::<_, ChangeLogEntry>(
            r#"
            SELECT id, entity_type, entity_id, change_type, change_hash, actor_id, created_at
            FROM _merge_change_log
            WHERE entity_type = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(entity_type)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| MergeError::Database(e.to_string()))?;

        Ok(entries)
    }

    /// Cleanup old change log entries
    pub async fn cleanup_change_log(&self, days: i64) -> MergeResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM _merge_change_log
            WHERE created_at < NOW() - INTERVAL '$1 days'
            "#,
        )
        .bind(days)
        .execute(&self.pool)
        .await
        .map_err(|e| MergeError::Database(e.to_string()))?;

        Ok(result.rows_affected())
    }
}

#[async_trait]
impl DocumentStorage for PostgresStorage {
    async fn save_document(&self, entity_type: &str, data: &[u8]) -> MergeResult<()> {
        sqlx::query(
            r#"
            INSERT INTO _merge_documents (entity_type, data, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (entity_type) DO UPDATE SET
                data = EXCLUDED.data,
                updated_at = NOW()
            "#,
        )
        .bind(entity_type)
        .bind(data)
        .execute(&self.pool)
        .await
        .map_err(|e| MergeError::Database(e.to_string()))?;

        Ok(())
    }

    async fn load_document(&self, entity_type: &str) -> MergeResult<Option<Vec<u8>>> {
        let result = sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT data FROM _merge_documents WHERE entity_type = $1",
        )
        .bind(entity_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| MergeError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn delete_document(&self, entity_type: &str) -> MergeResult<()> {
        sqlx::query("DELETE FROM _merge_documents WHERE entity_type = $1")
            .bind(entity_type)
            .execute(&self.pool)
            .await
            .map_err(|e| MergeError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_documents(&self) -> MergeResult<Vec<String>> {
        let result =
            sqlx::query_scalar::<_, String>("SELECT entity_type FROM _merge_documents")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| MergeError::Database(e.to_string()))?;

        Ok(result)
    }

    async fn migrate(&self) -> MergeResult<()> {
        // Create documents table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS _merge_documents (
                entity_type VARCHAR(255) PRIMARY KEY,
                data BYTEA NOT NULL,
                heads JSONB DEFAULT '[]'::jsonb,
                change_count BIGINT DEFAULT 0,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| MergeError::Database(e.to_string()))?;

        // Create change log table for auditing
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS _merge_change_log (
                id BIGSERIAL PRIMARY KEY,
                entity_type VARCHAR(255) NOT NULL,
                entity_id VARCHAR(255) NOT NULL,
                change_type VARCHAR(50) NOT NULL,
                change_hash VARCHAR(255),
                actor_id VARCHAR(255),
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| MergeError::Database(e.to_string()))?;

        // Create index on change log
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_merge_change_log_entity
            ON _merge_change_log(entity_type, created_at DESC)
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| MergeError::Database(e.to_string()))?;

        // Create index for entity_id lookups
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_merge_change_log_entity_id
            ON _merge_change_log(entity_type, entity_id)
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| MergeError::Database(e.to_string()))?;

        tracing::info!("PostgreSQL migrations completed");

        Ok(())
    }
}

/// Change log entry from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ChangeLogEntry {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: String,
    pub change_type: String,
    pub change_hash: Option<String>,
    pub actor_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests require a running PostgreSQL instance
    // Set DATABASE_URL environment variable to run

    #[tokio::test]
    #[ignore]
    async fn test_save_and_load_document() {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
        let storage = PostgresStorage::new(&db_url).await.unwrap();
        storage.migrate().await.unwrap();

        let data = b"test document data";
        storage.save_document("test_entity", data).await.unwrap();

        let loaded = storage.load_document("test_entity").await.unwrap();
        assert_eq!(loaded, Some(data.to_vec()));

        // Cleanup
        storage.delete_document("test_entity").await.unwrap();
    }
}
