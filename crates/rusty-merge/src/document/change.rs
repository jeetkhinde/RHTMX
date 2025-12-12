//! Document change tracking for sync notifications

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Type of change made to an entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    /// Entity was created
    Create,
    /// Entity was updated (one or more fields changed)
    Update,
    /// Entity was deleted
    Delete,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Create => write!(f, "create"),
            ChangeType::Update => write!(f, "update"),
            ChangeType::Delete => write!(f, "delete"),
        }
    }
}

/// Represents a change to a document that should be synced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChange {
    /// Entity type (e.g., "users", "posts")
    pub entity_type: String,
    /// Entity ID
    pub entity_id: String,
    /// Type of change
    pub change_type: ChangeType,
    /// The new data (None for deletes)
    pub data: Option<JsonValue>,
    /// Automerge change hash
    pub change_hash: String,
    /// Timestamp of the change
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Actor ID who made the change
    pub actor_id: String,
    /// Optional client-provided request ID for correlation
    pub request_id: Option<String>,
}

impl DocumentChange {
    pub fn new(
        entity_type: String,
        entity_id: String,
        change_type: ChangeType,
        data: Option<JsonValue>,
        change_hash: String,
        actor_id: String,
    ) -> Self {
        Self {
            entity_type,
            entity_id,
            change_type,
            data,
            change_hash,
            timestamp: chrono::Utc::now(),
            actor_id,
            request_id: None,
        }
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

/// Batch of changes for efficient sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeBatch {
    /// Entity type these changes belong to
    pub entity_type: String,
    /// Individual changes
    pub changes: Vec<DocumentChange>,
    /// Binary Automerge update containing all changes
    pub automerge_update: Vec<u8>,
    /// The heads after applying these changes
    pub heads: Vec<String>,
}

impl ChangeBatch {
    pub fn new(entity_type: String, automerge_update: Vec<u8>, heads: Vec<String>) -> Self {
        Self {
            entity_type,
            changes: vec![],
            automerge_update,
            heads,
        }
    }

    pub fn with_changes(mut self, changes: Vec<DocumentChange>) -> Self {
        self.changes = changes;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty() && self.automerge_update.is_empty()
    }
}

/// Sync request from client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    /// Entity type to sync
    pub entity_type: String,
    /// Client's current heads (empty for initial sync)
    pub heads: Vec<String>,
    /// Optional: specific entity IDs to sync
    pub entity_ids: Option<Vec<String>>,
}

/// Sync response to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    /// Entity type
    pub entity_type: String,
    /// Binary Automerge update
    pub update: Vec<u8>,
    /// New heads after applying update
    pub heads: Vec<String>,
    /// Number of changes included
    pub change_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_document_change_serialization() {
        let change = DocumentChange::new(
            "users".into(),
            "user_1".into(),
            ChangeType::Create,
            Some(json!({"name": "Alice"})),
            "abc123".into(),
            "actor_1".into(),
        );

        let json = serde_json::to_string(&change).unwrap();
        assert!(json.contains("users"));
        assert!(json.contains("create"));

        let parsed: DocumentChange = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.entity_type, "users");
        assert_eq!(parsed.change_type, ChangeType::Create);
    }
}
