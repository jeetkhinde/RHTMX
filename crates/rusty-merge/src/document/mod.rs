//! Automerge document wrapper for CRUD operations
//!
//! This module provides a high-level API for working with Automerge documents,
//! mapping traditional CRUD operations to CRDT mutations.

mod entity;
mod convert;
mod change;

pub use entity::EntityDocument;
pub use convert::{json_to_automerge, automerge_to_json};
pub use change::{DocumentChange, ChangeType};

use automerge::{ChangeHash, ObjId};
use serde::{Deserialize, Serialize};

/// Metadata stored with each entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMeta {
    /// When the entity was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the entity was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Actor ID that created this entity
    pub created_by: Option<String>,
    /// Actor ID that last updated this entity
    pub updated_by: Option<String>,
}

impl Default for EntityMeta {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            created_by: None,
            updated_by: None,
        }
    }
}

/// A reference to an entity within a document
#[derive(Debug, Clone)]
pub struct EntityRef {
    pub entity_type: String,
    pub entity_id: String,
    pub obj_id: ObjId,
}

/// Sync state for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    /// The heads (latest change hashes) we've synced to
    pub heads: Vec<String>,
    /// Number of changes applied
    pub change_count: usize,
    /// Last sync timestamp
    pub last_sync: chrono::DateTime<chrono::Utc>,
}

impl SyncState {
    pub fn new() -> Self {
        Self {
            heads: vec![],
            change_count: 0,
            last_sync: chrono::Utc::now(),
        }
    }

    pub fn from_heads(heads: &[ChangeHash]) -> Self {
        Self {
            heads: heads.iter().map(|h| h.to_string()).collect(),
            change_count: 0,
            last_sync: chrono::Utc::now(),
        }
    }
}

impl Default for SyncState {
    fn default() -> Self {
        Self::new()
    }
}
