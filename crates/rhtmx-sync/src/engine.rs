// File: rhtmx-sync/src/engine.rs
// Purpose: Main sync engine orchestration

use axum::{
    routing::{get, post},
    Router, Extension,
};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::{
    change_tracker::ChangeTracker,
    conflict::SyncStrategy,
    sse::sync_events_handler,
    sync_api::{get_sync_handler, post_sync_handler},
};

/// Configuration for the sync engine
#[derive(Clone)]
pub struct SyncConfig {
    /// Database connection pool
    pub db_pool: SqlitePool,

    /// Entities to sync (table names)
    pub entities: Vec<String>,

    /// Conflict resolution strategy
    pub strategy: SyncStrategy,

    /// Enable debug logging
    pub debug: bool,
}

impl SyncConfig {
    pub fn new(db_pool: SqlitePool, entities: Vec<String>) -> Self {
        Self {
            db_pool,
            entities,
            strategy: SyncStrategy::default(),
            debug: false,
        }
    }
}

/// Main sync engine
pub struct SyncEngine {
    #[allow(dead_code)]
    config: SyncConfig,
    change_tracker: Arc<ChangeTracker>,
}

impl SyncEngine {
    /// Create a new sync engine
    pub async fn new(config: SyncConfig) -> anyhow::Result<Self> {
        let db_pool = Arc::new(config.db_pool.clone());
        let change_tracker = Arc::new(ChangeTracker::new(db_pool).await?);

        Ok(Self {
            config,
            change_tracker,
        })
    }

    /// Get Axum routes for the sync API
    pub fn routes(&self) -> Router {
        let tracker = self.change_tracker.clone();
        let broadcast_tx = Arc::new(tracker.subscribe().resubscribe());

        Router::new()
            // Sync API endpoints
            .route("/api/sync/:entity", get(get_sync_handler))
            .route("/api/sync/:entity", post(post_sync_handler))
            // SSE endpoint for real-time updates
            .route("/api/sync/events", get(sync_events_handler))
            // Client JavaScript library
            .route("/api/sync/client.js", get(serve_client_js))
            // Inject dependencies
            .layer(Extension(tracker))
            .layer(Extension(broadcast_tx))
    }

    /// Get the change tracker
    pub fn tracker(&self) -> &Arc<ChangeTracker> {
        &self.change_tracker
    }

    /// Clean up old sync log entries
    pub async fn cleanup(&self, days: i64) -> anyhow::Result<u64> {
        self.change_tracker.cleanup_old_entries(days).await
    }
}

/// Serve the client-side JavaScript library
async fn serve_client_js() -> ([(axum::http::HeaderName, &'static str); 1], &'static str) {
    (
        [(axum::http::header::CONTENT_TYPE, "application/javascript")],
        include_str!("js/rhtmx-sync.js"),
    )
}
