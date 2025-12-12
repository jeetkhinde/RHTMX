//! Storage backends for Automerge documents
//!
//! This module provides persistent storage for Automerge documents.
//! The primary backend is PostgreSQL, storing documents as binary blobs.

mod postgres;

pub use postgres::PostgresStorage;

use async_trait::async_trait;
use crate::error::MergeResult;

/// Trait for document storage backends
#[async_trait]
pub trait DocumentStorage: Send + Sync {
    /// Save a document
    async fn save_document(&self, entity_type: &str, data: &[u8]) -> MergeResult<()>;

    /// Load a document
    async fn load_document(&self, entity_type: &str) -> MergeResult<Option<Vec<u8>>>;

    /// Delete a document
    async fn delete_document(&self, entity_type: &str) -> MergeResult<()>;

    /// List all document IDs
    async fn list_documents(&self) -> MergeResult<Vec<String>>;

    /// Run migrations
    async fn migrate(&self) -> MergeResult<()>;
}
