//! Sync protocol messages

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::document::DocumentChange;

/// WebSocket sync protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncMessage {
    // =========================================================================
    // Connection & Subscription
    // =========================================================================

    /// Client subscribes to entity types
    Subscribe {
        /// Entity types to subscribe to
        entities: Vec<String>,
        /// Optional: client's current sync state (heads per entity)
        #[serde(default)]
        sync_state: Option<SyncStateMap>,
    },

    /// Server confirms subscription
    Subscribed {
        /// Entities successfully subscribed
        entities: Vec<String>,
    },

    /// Client unsubscribes from entity types
    Unsubscribe {
        entities: Vec<String>,
    },

    // =========================================================================
    // Sync Operations
    // =========================================================================

    /// Client requests sync (initial or catch-up)
    SyncRequest {
        /// Entity type to sync
        entity: String,
        /// Client's current heads (empty for initial sync)
        #[serde(default)]
        heads: Vec<String>,
    },

    /// Server sends sync response with Automerge update
    SyncResponse {
        /// Entity type
        entity: String,
        /// Binary Automerge update (base64 encoded)
        update: String,
        /// New heads after applying update
        heads: Vec<String>,
        /// Number of entities
        count: usize,
    },

    /// Server pushes a change notification
    Change {
        /// The change details
        change: DocumentChange,
    },

    // =========================================================================
    // CRUD Operations (via WebSocket)
    // =========================================================================

    /// Client creates an entity
    Create {
        /// Request ID for correlation
        request_id: String,
        /// Entity type
        entity: String,
        /// Entity ID (optional, server generates if not provided)
        #[serde(default)]
        id: Option<String>,
        /// Entity data
        data: JsonValue,
    },

    /// Client updates an entity
    Update {
        request_id: String,
        entity: String,
        id: String,
        /// Fields to update
        data: JsonValue,
    },

    /// Client updates a single field
    UpdateField {
        request_id: String,
        entity: String,
        id: String,
        field: String,
        value: JsonValue,
    },

    /// Client deletes an entity
    Delete {
        request_id: String,
        entity: String,
        id: String,
    },

    /// Server acknowledges a mutation
    Ack {
        /// Correlates with request_id
        request_id: String,
        /// Success or failure
        success: bool,
        /// Error message if failed
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        /// The resulting entity data (for create/update)
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<JsonValue>,
    },

    // =========================================================================
    // Binary Sync (for efficient bulk sync)
    // =========================================================================

    /// Client sends binary Automerge changes
    BinarySync {
        entity: String,
        /// Base64 encoded Automerge changes
        data: String,
    },

    /// Server sends binary Automerge state
    BinaryState {
        entity: String,
        /// Base64 encoded Automerge document
        data: String,
        heads: Vec<String>,
    },

    // =========================================================================
    // Connection Management
    // =========================================================================

    /// Heartbeat ping
    Ping {
        #[serde(default)]
        timestamp: Option<i64>,
    },

    /// Heartbeat pong
    Pong {
        #[serde(default)]
        timestamp: Option<i64>,
    },

    /// Error message
    Error {
        /// Error message
        message: String,
        /// Error code
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
        /// Related request ID if applicable
        #[serde(skip_serializing_if = "Option::is_none")]
        request_id: Option<String>,
    },
}

/// Sync state map (entity -> heads)
pub type SyncStateMap = std::collections::HashMap<String, Vec<String>>;

impl SyncMessage {
    /// Create an error message
    pub fn error(message: impl Into<String>) -> Self {
        SyncMessage::Error {
            message: message.into(),
            code: None,
            request_id: None,
        }
    }

    /// Create an error with code
    pub fn error_with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        SyncMessage::Error {
            message: message.into(),
            code: Some(code.into()),
            request_id: None,
        }
    }

    /// Create an ack for a request
    pub fn ack(request_id: String, success: bool) -> Self {
        SyncMessage::Ack {
            request_id,
            success,
            error: None,
            data: None,
        }
    }

    /// Create an ack with data
    pub fn ack_with_data(request_id: String, data: JsonValue) -> Self {
        SyncMessage::Ack {
            request_id,
            success: true,
            error: None,
            data: Some(data),
        }
    }

    /// Create an error ack
    pub fn ack_error(request_id: String, error: impl Into<String>) -> Self {
        SyncMessage::Ack {
            request_id,
            success: false,
            error: Some(error.into()),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = SyncMessage::Subscribe {
            entities: vec!["users".into(), "posts".into()],
            sync_state: None,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("subscribe"));
        assert!(json.contains("users"));

        let parsed: SyncMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            SyncMessage::Subscribe { entities, .. } => {
                assert_eq!(entities.len(), 2);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_create_message() {
        let msg = SyncMessage::Create {
            request_id: "req_123".into(),
            entity: "users".into(),
            id: Some("user_1".into()),
            data: serde_json::json!({"name": "Alice"}),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("create"));
        assert!(json.contains("Alice"));
    }
}
