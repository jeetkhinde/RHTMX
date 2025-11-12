// File: rhtmx-sync/src/change_tracker.rs
// Purpose: Track database changes for synchronization

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Action performed on an entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeAction {
    Create,
    Update,
    Delete,
}

impl std::fmt::Display for ChangeAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeAction::Create => write!(f, "create"),
            ChangeAction::Update => write!(f, "update"),
            ChangeAction::Delete => write!(f, "delete"),
        }
    }
}

/// A single change entry in the sync log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeLog {
    pub id: i64,
    pub entity: String,
    pub entity_id: String,
    pub action: ChangeAction,
    pub data: Option<serde_json::Value>,
    pub version: i64,
    pub client_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Manages change tracking and broadcasts
pub struct ChangeTracker {
    db_pool: Arc<SqlitePool>,
    broadcast_tx: broadcast::Sender<ChangeLog>,
}

impl ChangeTracker {
    /// Create a new change tracker
    pub async fn new(db_pool: Arc<SqlitePool>) -> anyhow::Result<Self> {
        // Create sync log table if it doesn't exist
        Self::init_sync_table(&db_pool).await?;

        let (broadcast_tx, _) = broadcast::channel(1000);

        Ok(Self {
            db_pool,
            broadcast_tx,
        })
    }

    /// Initialize the sync log table
    async fn init_sync_table(pool: &SqlitePool) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS _rhtmx_sync_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entity TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                action TEXT NOT NULL,
                data TEXT,
                version INTEGER NOT NULL,
                client_id TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;

        // Create index for efficient querying
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_sync_entity_version
            ON _rhtmx_sync_log(entity, version)
            "#,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Record a change in the sync log
    pub async fn record_change(
        &self,
        entity: &str,
        entity_id: &str,
        action: ChangeAction,
        data: Option<serde_json::Value>,
        client_id: Option<String>,
    ) -> anyhow::Result<ChangeLog> {
        use sqlx::Row;

        // Get next version number
        let version = self.next_version(entity).await?;

        // Serialize data to JSON string if present
        let data_json = data.as_ref().map(|d| serde_json::to_string(d).unwrap());

        // Insert into sync log
        let row = sqlx::query(
            r#"
            INSERT INTO _rhtmx_sync_log (entity, entity_id, action, data, version, client_id)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id, entity, entity_id, action, data, version, client_id, created_at
            "#
        )
        .bind(entity)
        .bind(entity_id)
        .bind(action.to_string())
        .bind(data_json)
        .bind(version)
        .bind(&client_id)
        .fetch_one(&*self.db_pool)
        .await?;

        // Parse row into ChangeLog
        let action_str: String = row.get("action");
        let action_parsed = match action_str.as_str() {
            "create" => ChangeAction::Create,
            "update" => ChangeAction::Update,
            "delete" => ChangeAction::Delete,
            _ => ChangeAction::Update,
        };

        let data_str: Option<String> = row.get("data");
        let data_parsed = data_str.and_then(|s| serde_json::from_str(&s).ok());

        let change = ChangeLog {
            id: row.get("id"),
            entity: row.get("entity"),
            entity_id: row.get("entity_id"),
            action: action_parsed,
            data: data_parsed,
            version: row.get("version"),
            client_id: row.get("client_id"),
            created_at: row.get("created_at"),
        };

        // Broadcast the change
        let _ = self.broadcast_tx.send(change.clone());

        Ok(change)
    }

    /// Get all changes since a specific version
    pub async fn get_changes_since(
        &self,
        entity: &str,
        since_version: i64,
    ) -> anyhow::Result<Vec<ChangeLog>> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"
            SELECT id, entity, entity_id, action, data, version, client_id, created_at
            FROM _rhtmx_sync_log
            WHERE entity = ? AND version > ?
            ORDER BY version ASC
            "#
        )
        .bind(entity)
        .bind(since_version)
        .fetch_all(&*self.db_pool)
        .await?;

        let changes = rows
            .iter()
            .map(|row| {
                let action_str: String = row.get("action");
                let action = match action_str.as_str() {
                    "create" => ChangeAction::Create,
                    "update" => ChangeAction::Update,
                    "delete" => ChangeAction::Delete,
                    _ => ChangeAction::Update,
                };

                let data_str: Option<String> = row.get("data");
                let data = data_str.and_then(|s| serde_json::from_str(&s).ok());

                ChangeLog {
                    id: row.get("id"),
                    entity: row.get("entity"),
                    entity_id: row.get("entity_id"),
                    action,
                    data,
                    version: row.get("version"),
                    client_id: row.get("client_id"),
                    created_at: row.get("created_at"),
                }
            })
            .collect();

        Ok(changes)
    }

    /// Get the latest version for an entity
    pub async fn latest_version(&self, entity: &str) -> anyhow::Result<i64> {
        let result: Option<i64> = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) FROM _rhtmx_sync_log WHERE entity = ?"
        )
        .bind(entity)
        .fetch_one(&*self.db_pool)
        .await?;

        Ok(result.unwrap_or(0))
    }

    /// Get next version number for an entity
    async fn next_version(&self, entity: &str) -> anyhow::Result<i64> {
        let current = self.latest_version(entity).await?;
        Ok(current + 1)
    }

    /// Subscribe to change events
    pub fn subscribe(&self) -> broadcast::Receiver<ChangeLog> {
        self.broadcast_tx.subscribe()
    }

    /// Clean up old sync log entries (call periodically)
    pub async fn cleanup_old_entries(&self, days: i64) -> anyhow::Result<u64> {
        let days_param = format!("-{} days", days);
        let result = sqlx::query(
            "DELETE FROM _rhtmx_sync_log WHERE created_at < datetime('now', ?)"
        )
        .bind(days_param)
        .execute(&*self.db_pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn test_change_tracker() {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let tracker = ChangeTracker::new(Arc::new(pool)).await.unwrap();

        // Record a change
        let change = tracker
            .record_change(
                "users",
                "1",
                ChangeAction::Create,
                Some(serde_json::json!({"name": "Alice"})),
                None,
            )
            .await
            .unwrap();

        assert_eq!(change.entity, "users");
        assert_eq!(change.version, 1);

        // Get latest version
        let version = tracker.latest_version("users").await.unwrap();
        assert_eq!(version, 1);

        // Record another change
        tracker
            .record_change("users", "1", ChangeAction::Update, None, None)
            .await
            .unwrap();

        // Get changes since version 0
        let changes = tracker.get_changes_since("users", 0).await.unwrap();
        assert_eq!(changes.len(), 2);
    }
}
