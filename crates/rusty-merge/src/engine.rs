//! Main sync engine orchestration
//!
//! The MergeEngine is the central component that ties together:
//! - Automerge document management
//! - PostgreSQL storage
//! - WebSocket transport
//! - SQL projections

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use serde_json::Value as JsonValue;
use tokio::sync::broadcast;

use crate::document::{ChangeType, DocumentChange, EntityDocument};
use crate::error::{MergeError, MergeResult};
use crate::projection::ProjectionManager;
use crate::storage::{DocumentStorage, PostgresStorage};
use crate::transport::{ws_handler, WebSocketState};

/// Configuration for the merge engine
#[derive(Clone)]
pub struct MergeConfig {
    /// PostgreSQL connection string
    pub database_url: String,
    /// Entity types to sync (e.g., ["users", "posts"])
    pub entities: Vec<String>,
    /// Enable automatic projection to SQL tables
    pub enable_projection: bool,
    /// Maximum connections in pool
    pub max_connections: u32,
    /// Enable debug logging
    pub debug: bool,
}

impl MergeConfig {
    /// Create a new configuration
    pub fn new(database_url: &str) -> Self {
        Self {
            database_url: database_url.to_string(),
            entities: vec![],
            enable_projection: true,
            max_connections: 10,
            debug: false,
        }
    }

    /// Set the entities to sync
    pub fn with_entities(mut self, entities: Vec<&str>) -> Self {
        self.entities = entities.into_iter().map(|s| s.to_string()).collect();
        self
    }

    /// Enable or disable projection
    pub fn with_projection(mut self, enable: bool) -> Self {
        self.enable_projection = enable;
        self
    }

    /// Set max database connections
    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Enable debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
}

/// Main sync engine
pub struct MergeEngine {
    config: MergeConfig,
    storage: Arc<PostgresStorage>,
    documents: DashMap<String, EntityDocument>,
    projection: Option<Arc<ProjectionManager>>,
    broadcast_tx: broadcast::Sender<DocumentChange>,
}

impl MergeEngine {
    /// Create a new merge engine
    pub async fn new(config: MergeConfig) -> MergeResult<Self> {
        // Initialize storage
        let storage = Arc::new(PostgresStorage::new(&config.database_url).await?);

        // Run migrations
        storage.migrate().await?;

        // Initialize projection manager if enabled
        let projection = if config.enable_projection {
            Some(Arc::new(
                ProjectionManager::new(&config.database_url).await?,
            ))
        } else {
            None
        };

        // Create broadcast channel for real-time updates
        let (broadcast_tx, _) = broadcast::channel(1000);

        let engine = Self {
            config: config.clone(),
            storage,
            documents: DashMap::new(),
            projection,
            broadcast_tx,
        };

        // Load or create documents for each entity
        for entity in &config.entities {
            engine.load_or_create_document(entity).await?;
        }

        tracing::info!(
            "MergeEngine initialized with {} entities",
            config.entities.len()
        );

        Ok(engine)
    }

    /// Load or create a document for an entity type
    async fn load_or_create_document(&self, entity_type: &str) -> MergeResult<()> {
        let doc = match self.storage.load_document(entity_type).await? {
            Some(bytes) => {
                tracing::info!("Loaded existing document for {}", entity_type);
                EntityDocument::load(entity_type, &bytes)?
            }
            None => {
                tracing::info!("Creating new document for {}", entity_type);
                let doc = EntityDocument::new(entity_type)?;
                // Save immediately
                let mut doc_clone = doc;
                self.storage
                    .save_document(entity_type, &doc_clone.save())
                    .await?;
                doc_clone
            }
        };

        self.documents.insert(entity_type.to_string(), doc);
        Ok(())
    }

    /// Get Axum routes for the sync API
    pub fn routes(self: Arc<Self>) -> Router {
        let ws_state = Arc::new(WebSocketState::new(
            self.clone(),
            self.broadcast_tx.subscribe(),
        ));

        Router::new()
            // WebSocket endpoint for real-time sync
            .route("/api/merge/ws", get(ws_handler))
            // HTTP endpoints
            .route("/api/merge/:entity", get(Self::http_list_handler))
            .route("/api/merge/:entity", post(Self::http_create_handler))
            .route("/api/merge/:entity/:id", get(Self::http_read_handler))
            .route("/api/merge/:entity/:id", post(Self::http_update_handler))
            .route(
                "/api/merge/:entity/:id",
                axum::routing::delete(Self::http_delete_handler),
            )
            // Sync endpoints
            .route("/api/merge/:entity/sync", post(Self::http_sync_handler))
            // Client JS
            .route("/api/merge/client.js", get(Self::serve_client_js))
            .with_state(ws_state)
    }

    // =========================================================================
    // CRUD Operations
    // =========================================================================

    /// Create a new entity
    pub async fn create(
        &self,
        entity_type: &str,
        id: &str,
        data: JsonValue,
    ) -> MergeResult<JsonValue> {
        let mut doc = self
            .documents
            .get_mut(entity_type)
            .ok_or_else(|| MergeError::DocumentNotFound(entity_type.to_string()))?;

        // Create in Automerge
        doc.create(id, data.clone())?;

        // Get the created entity (with metadata)
        let entity = doc.read(id)?.ok_or_else(|| MergeError::Internal(
            "Entity not found after create".into(),
        ))?;

        // Save to storage
        self.storage.save_document(entity_type, &doc.save()).await?;

        // Project to SQL
        if let Some(proj) = &self.projection {
            proj.project_entity(entity_type, id, &entity).await?;
        }

        // Broadcast change
        let change = DocumentChange::new(
            entity_type.to_string(),
            id.to_string(),
            ChangeType::Create,
            Some(entity.clone()),
            doc.heads().first().map(|h| h.to_string()).unwrap_or_default(),
            "server".to_string(),
        );
        let _ = self.broadcast_tx.send(change);

        Ok(entity)
    }

    /// Read an entity by ID
    pub async fn read(&self, entity_type: &str, id: &str) -> MergeResult<Option<JsonValue>> {
        let doc = self
            .documents
            .get(entity_type)
            .ok_or_else(|| MergeError::DocumentNotFound(entity_type.to_string()))?;

        doc.read(id)
    }

    /// Update specific fields of an entity
    pub async fn update(
        &self,
        entity_type: &str,
        id: &str,
        updates: JsonValue,
    ) -> MergeResult<JsonValue> {
        let mut doc = self
            .documents
            .get_mut(entity_type)
            .ok_or_else(|| MergeError::DocumentNotFound(entity_type.to_string()))?;

        // Update in Automerge
        doc.update(id, updates)?;

        // Get updated entity
        let entity = doc.read(id)?.ok_or_else(|| MergeError::NotFound {
            entity: entity_type.to_string(),
            id: id.to_string(),
        })?;

        // Save to storage
        self.storage.save_document(entity_type, &doc.save()).await?;

        // Project to SQL
        if let Some(proj) = &self.projection {
            proj.project_entity(entity_type, id, &entity).await?;
        }

        // Broadcast change
        let change = DocumentChange::new(
            entity_type.to_string(),
            id.to_string(),
            ChangeType::Update,
            Some(entity.clone()),
            doc.heads().first().map(|h| h.to_string()).unwrap_or_default(),
            "server".to_string(),
        );
        let _ = self.broadcast_tx.send(change);

        Ok(entity)
    }

    /// Update a single field
    pub async fn update_field(
        &self,
        entity_type: &str,
        id: &str,
        field: &str,
        value: JsonValue,
    ) -> MergeResult<JsonValue> {
        let mut doc = self
            .documents
            .get_mut(entity_type)
            .ok_or_else(|| MergeError::DocumentNotFound(entity_type.to_string()))?;

        // Update field in Automerge
        doc.update_field(id, field, value)?;

        // Get updated entity
        let entity = doc.read(id)?.ok_or_else(|| MergeError::NotFound {
            entity: entity_type.to_string(),
            id: id.to_string(),
        })?;

        // Save to storage
        self.storage.save_document(entity_type, &doc.save()).await?;

        // Project to SQL
        if let Some(proj) = &self.projection {
            proj.project_entity(entity_type, id, &entity).await?;
        }

        // Broadcast change
        let change = DocumentChange::new(
            entity_type.to_string(),
            id.to_string(),
            ChangeType::Update,
            Some(entity.clone()),
            doc.heads().first().map(|h| h.to_string()).unwrap_or_default(),
            "server".to_string(),
        );
        let _ = self.broadcast_tx.send(change);

        Ok(entity)
    }

    /// Delete an entity
    pub async fn delete(&self, entity_type: &str, id: &str) -> MergeResult<bool> {
        let mut doc = self
            .documents
            .get_mut(entity_type)
            .ok_or_else(|| MergeError::DocumentNotFound(entity_type.to_string()))?;

        // Delete in Automerge
        let deleted = doc.delete(id)?;

        if deleted {
            // Save to storage
            self.storage.save_document(entity_type, &doc.save()).await?;

            // Remove from projection
            if let Some(proj) = &self.projection {
                proj.delete_entity(entity_type, id).await?;
            }

            // Broadcast change
            let change = DocumentChange::new(
                entity_type.to_string(),
                id.to_string(),
                ChangeType::Delete,
                None,
                doc.heads().first().map(|h| h.to_string()).unwrap_or_default(),
                "server".to_string(),
            );
            let _ = self.broadcast_tx.send(change);
        }

        Ok(deleted)
    }

    /// List all entities of a type
    pub async fn list(&self, entity_type: &str) -> MergeResult<Vec<(String, JsonValue)>> {
        let doc = self
            .documents
            .get(entity_type)
            .ok_or_else(|| MergeError::DocumentNotFound(entity_type.to_string()))?;

        doc.list()
    }

    /// Get entity count
    pub async fn count(&self, entity_type: &str) -> MergeResult<usize> {
        let doc = self
            .documents
            .get(entity_type)
            .ok_or_else(|| MergeError::DocumentNotFound(entity_type.to_string()))?;

        Ok(doc.count())
    }

    // =========================================================================
    // Sync Operations
    // =========================================================================

    /// Get changes since given heads (for sync)
    pub fn get_changes_since(
        &self,
        entity_type: &str,
        heads: &[automerge::ChangeHash],
    ) -> MergeResult<Vec<u8>> {
        let mut doc = self
            .documents
            .get_mut(entity_type)
            .ok_or_else(|| MergeError::DocumentNotFound(entity_type.to_string()))?;

        Ok(doc.save_incremental(heads))
    }

    /// Apply changes from a client
    pub async fn apply_changes(&self, entity_type: &str, changes: &[u8]) -> MergeResult<()> {
        let mut doc = self
            .documents
            .get_mut(entity_type)
            .ok_or_else(|| MergeError::DocumentNotFound(entity_type.to_string()))?;

        // Apply changes to Automerge
        doc.load_incremental(changes)?;

        // Save to storage
        self.storage.save_document(entity_type, &doc.save()).await?;

        // Rebuild projections for affected entities
        if let Some(proj) = &self.projection {
            for (id, data) in doc.list()? {
                proj.project_entity(entity_type, &id, &data).await?;
            }
        }

        Ok(())
    }

    /// Get current heads for an entity
    pub fn get_heads(&self, entity_type: &str) -> MergeResult<Vec<automerge::ChangeHash>> {
        let mut doc = self
            .documents
            .get_mut(entity_type)
            .ok_or_else(|| MergeError::DocumentNotFound(entity_type.to_string()))?;

        Ok(doc.heads())
    }

    /// Subscribe to changes
    pub fn subscribe(&self) -> broadcast::Receiver<DocumentChange> {
        self.broadcast_tx.subscribe()
    }

    /// Get storage reference
    pub fn storage(&self) -> &Arc<PostgresStorage> {
        &self.storage
    }

    /// Get projection manager
    pub fn projection(&self) -> Option<&Arc<ProjectionManager>> {
        self.projection.as_ref()
    }

    // =========================================================================
    // HTTP Handlers
    // =========================================================================

    async fn http_list_handler(
        axum::extract::State(state): axum::extract::State<Arc<WebSocketState>>,
        axum::extract::Path(entity): axum::extract::Path<String>,
    ) -> axum::response::Result<axum::Json<Vec<JsonValue>>> {
        let entities = state
            .engine
            .list(&entity)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let with_ids: Vec<JsonValue> = entities
            .into_iter()
            .map(|(id, mut data)| {
                if let Some(obj) = data.as_object_mut() {
                    obj.insert("id".to_string(), JsonValue::String(id));
                }
                data
            })
            .collect();

        Ok(axum::Json(with_ids))
    }

    async fn http_create_handler(
        axum::extract::State(state): axum::extract::State<Arc<WebSocketState>>,
        axum::extract::Path(entity): axum::extract::Path<String>,
        axum::Json(payload): axum::Json<JsonValue>,
    ) -> axum::response::Result<axum::Json<JsonValue>> {
        let id = payload
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let result = state
            .engine
            .create(&entity, &id, payload)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(axum::Json(result))
    }

    async fn http_read_handler(
        axum::extract::State(state): axum::extract::State<Arc<WebSocketState>>,
        axum::extract::Path((entity, id)): axum::extract::Path<(String, String)>,
    ) -> axum::response::Result<axum::Json<JsonValue>> {
        let result = state
            .engine
            .read(&entity, &id)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        match result {
            Some(data) => Ok(axum::Json(data)),
            None => Err((axum::http::StatusCode::NOT_FOUND, "Not found".to_string()).into()),
        }
    }

    async fn http_update_handler(
        axum::extract::State(state): axum::extract::State<Arc<WebSocketState>>,
        axum::extract::Path((entity, id)): axum::extract::Path<(String, String)>,
        axum::Json(payload): axum::Json<JsonValue>,
    ) -> axum::response::Result<axum::Json<JsonValue>> {
        let result = state
            .engine
            .update(&entity, &id, payload)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        Ok(axum::Json(result))
    }

    async fn http_delete_handler(
        axum::extract::State(state): axum::extract::State<Arc<WebSocketState>>,
        axum::extract::Path((entity, id)): axum::extract::Path<(String, String)>,
    ) -> axum::response::Result<axum::http::StatusCode> {
        let deleted = state
            .engine
            .delete(&entity, &id)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        if deleted {
            Ok(axum::http::StatusCode::NO_CONTENT)
        } else {
            Ok(axum::http::StatusCode::NOT_FOUND)
        }
    }

    async fn http_sync_handler(
        axum::extract::State(state): axum::extract::State<Arc<WebSocketState>>,
        axum::extract::Path(entity): axum::extract::Path<String>,
        body: axum::body::Bytes,
    ) -> axum::response::Result<Vec<u8>> {
        // Apply incoming changes
        if !body.is_empty() {
            state
                .engine
                .apply_changes(&entity, &body)
                .await
                .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }

        // Return current state
        let mut doc = state
            .engine
            .documents
            .get_mut(&entity)
            .ok_or((
                axum::http::StatusCode::NOT_FOUND,
                "Entity not found".to_string(),
            ))?;

        Ok(doc.save())
    }

    async fn serve_client_js() -> ([(axum::http::HeaderName, &'static str); 1], &'static str) {
        (
            [(axum::http::header::CONTENT_TYPE, "application/javascript")],
            include_str!("client/rusty-merge.js"),
        )
    }
}
