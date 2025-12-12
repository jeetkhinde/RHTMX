//! Entity document operations - CRUD on Automerge

use automerge::{
    transaction::Transactable, AutoCommit, ChangeHash, ObjId, ObjType, ReadDoc, Value, ROOT,
};
use chrono::Utc;
use serde_json::Value as JsonValue;

use super::convert::{automerge_to_json, json_to_automerge};
use super::{EntityMeta, SyncState};
use crate::error::{MergeError, MergeResult};

/// Wrapper around Automerge document for entity-level CRUD operations
///
/// Document structure:
/// ```json
/// {
///   "_meta": { "entity_type": "users", "created_at": "..." },
///   "entities": {
///     "user_1": { "_meta": {...}, "name": "Alice", "email": "..." },
///     "user_2": { "_meta": {...}, "name": "Bob", "email": "..." }
///   }
/// }
/// ```
pub struct EntityDocument {
    doc: AutoCommit,
    entity_type: String,
    entities_obj: ObjId,
}

impl EntityDocument {
    /// Create a new document for an entity type
    pub fn new(entity_type: &str) -> MergeResult<Self> {
        let mut doc = AutoCommit::new();

        // Create root structure
        let meta_obj = doc.put_object(ROOT, "_meta", ObjType::Map)?;
        doc.put(&meta_obj, "entity_type", entity_type)?;
        doc.put(
            &meta_obj,
            "created_at",
            Utc::now().to_rfc3339().as_str(),
        )?;

        // Create entities container
        let entities_obj = doc.put_object(ROOT, "entities", ObjType::Map)?;

        Ok(Self {
            doc,
            entity_type: entity_type.to_string(),
            entities_obj,
        })
    }

    /// Load document from binary data
    pub fn load(entity_type: &str, data: &[u8]) -> MergeResult<Self> {
        let doc = AutoCommit::load(data)?;

        // Get entities object
        let entities_obj = match doc.get(ROOT, "entities")? {
            Some((Value::Object(ObjType::Map), obj_id)) => obj_id,
            _ => return Err(MergeError::InvalidData("Missing entities object".into())),
        };

        Ok(Self {
            doc,
            entity_type: entity_type.to_string(),
            entities_obj,
        })
    }

    /// Get the entity type this document manages
    pub fn entity_type(&self) -> &str {
        &self.entity_type
    }

    /// Get the underlying Automerge document
    pub fn doc(&self) -> &AutoCommit {
        &self.doc
    }

    /// Get mutable reference to the underlying document
    pub fn doc_mut(&mut self) -> &mut AutoCommit {
        &mut self.doc
    }

    /// Save document to binary format
    pub fn save(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    /// Get incremental changes since given heads
    pub fn save_incremental(&mut self, heads: &[ChangeHash]) -> Vec<u8> {
        self.doc.save_after(heads)
    }

    /// Load incremental changes
    pub fn load_incremental(&mut self, data: &[u8]) -> MergeResult<()> {
        self.doc.load_incremental(data)?;
        Ok(())
    }

    /// Get current heads (for sync state)
    pub fn heads(&mut self) -> Vec<ChangeHash> {
        self.doc.get_heads()
    }

    /// Get sync state
    pub fn sync_state(&mut self) -> SyncState {
        SyncState::from_heads(&self.heads())
    }

    /// Merge with another document
    pub fn merge(&mut self, other: &mut AutoCommit) -> MergeResult<()> {
        self.doc.merge(other)?;
        Ok(())
    }

    // =========================================================================
    // CRUD Operations
    // =========================================================================

    /// Create a new entity
    pub fn create(&mut self, id: &str, data: JsonValue) -> MergeResult<()> {
        // Check if entity already exists
        if self.exists(id)? {
            return Err(MergeError::InvalidOperation(format!(
                "Entity {} already exists",
                id
            )));
        }

        // Create entity object
        let entity_obj = self.doc.put_object(&self.entities_obj, id, ObjType::Map)?;

        // Add metadata
        let meta = EntityMeta::default();
        let meta_obj = self.doc.put_object(&entity_obj, "_meta", ObjType::Map)?;
        self.doc
            .put(&meta_obj, "created_at", meta.created_at.to_rfc3339().as_str())?;
        self.doc
            .put(&meta_obj, "updated_at", meta.updated_at.to_rfc3339().as_str())?;

        // Set data fields
        if let JsonValue::Object(map) = data {
            for (key, value) in map {
                if key != "_meta" {
                    json_to_automerge(&mut self.doc, &entity_obj, &key, &value)?;
                }
            }
        } else {
            return Err(MergeError::InvalidData(
                "Entity data must be an object".into(),
            ));
        }

        Ok(())
    }

    /// Read an entity by ID
    pub fn read(&self, id: &str) -> MergeResult<Option<JsonValue>> {
        match self.doc.get(&self.entities_obj, id)? {
            Some((Value::Object(ObjType::Map), obj_id)) => {
                let json = automerge_to_json(&self.doc, &obj_id)?;
                Ok(Some(json))
            }
            Some(_) => Err(MergeError::InvalidData(format!(
                "Entity {} is not an object",
                id
            ))),
            None => Ok(None),
        }
    }

    /// Check if an entity exists
    pub fn exists(&self, id: &str) -> MergeResult<bool> {
        Ok(self.doc.get(&self.entities_obj, id)?.is_some())
    }

    /// Update a specific field of an entity
    pub fn update_field(&mut self, id: &str, field: &str, value: JsonValue) -> MergeResult<()> {
        let entity_obj = self.get_entity_obj(id)?;

        // Update the field
        json_to_automerge(&mut self.doc, &entity_obj, field, &value)?;

        // Update metadata
        self.update_entity_meta(&entity_obj)?;

        Ok(())
    }

    /// Update multiple fields of an entity
    pub fn update(&mut self, id: &str, updates: JsonValue) -> MergeResult<()> {
        let entity_obj = self.get_entity_obj(id)?;

        if let JsonValue::Object(map) = updates {
            for (key, value) in map {
                if key != "_meta" && key != "id" {
                    json_to_automerge(&mut self.doc, &entity_obj, &key, &value)?;
                }
            }
        } else {
            return Err(MergeError::InvalidData("Updates must be an object".into()));
        }

        // Update metadata
        self.update_entity_meta(&entity_obj)?;

        Ok(())
    }

    /// Replace an entire entity (delete + create)
    pub fn replace(&mut self, id: &str, data: JsonValue) -> MergeResult<()> {
        // Get existing metadata if present
        let existing_meta = self.read(id)?.and_then(|e| {
            e.get("_meta")
                .and_then(|m| m.get("created_at"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        });

        // Delete existing
        self.doc.delete(&self.entities_obj, id)?;

        // Create new
        let entity_obj = self.doc.put_object(&self.entities_obj, id, ObjType::Map)?;

        // Add metadata (preserve created_at if existed)
        let now = Utc::now().to_rfc3339();
        let meta_obj = self.doc.put_object(&entity_obj, "_meta", ObjType::Map)?;
        self.doc.put(
            &meta_obj,
            "created_at",
            existing_meta.as_deref().unwrap_or(&now),
        )?;
        self.doc.put(&meta_obj, "updated_at", now.as_str())?;

        // Set data fields
        if let JsonValue::Object(map) = data {
            for (key, value) in map {
                if key != "_meta" {
                    json_to_automerge(&mut self.doc, &entity_obj, &key, &value)?;
                }
            }
        }

        Ok(())
    }

    /// Delete an entity
    pub fn delete(&mut self, id: &str) -> MergeResult<bool> {
        if self.exists(id)? {
            self.doc.delete(&self.entities_obj, id)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// List all entities
    pub fn list(&self) -> MergeResult<Vec<(String, JsonValue)>> {
        let mut result = vec![];

        for key in self.doc.keys(&self.entities_obj) {
            if let Some((Value::Object(ObjType::Map), obj_id)) =
                self.doc.get(&self.entities_obj, &key)?
            {
                let json = automerge_to_json(&self.doc, &obj_id)?;
                result.push((key.to_string(), json));
            }
        }

        Ok(result)
    }

    /// List entity IDs only
    pub fn list_ids(&self) -> Vec<String> {
        self.doc
            .keys(&self.entities_obj)
            .map(|k| k.to_string())
            .collect()
    }

    /// Count entities
    pub fn count(&self) -> usize {
        self.doc.length(&self.entities_obj)
    }

    /// Get entities matching a simple filter (field = value)
    pub fn filter(&self, field: &str, value: &JsonValue) -> MergeResult<Vec<(String, JsonValue)>> {
        let mut result = vec![];

        for key in self.doc.keys(&self.entities_obj) {
            if let Some((Value::Object(ObjType::Map), obj_id)) =
                self.doc.get(&self.entities_obj, &key)?
            {
                let json = automerge_to_json(&self.doc, &obj_id)?;

                // Check if field matches
                if json.get(field) == Some(value) {
                    result.push((key.to_string(), json));
                }
            }
        }

        Ok(result)
    }

    // =========================================================================
    // History & Time Travel
    // =========================================================================

    /// Get change history
    pub fn history(&mut self) -> Vec<ChangeInfo> {
        self.doc
            .get_changes(&[])
            .iter()
            .map(|c| ChangeInfo {
                hash: c.hash().to_string(),
                timestamp: c.timestamp(),
                actor: c.actor_id().to_string(),
                message: c.message().map(|s| s.to_string()),
            })
            .collect()
    }

    /// Get document at a specific point in history
    pub fn at_heads(&self, heads: &[ChangeHash]) -> MergeResult<AutoCommit> {
        let mut doc = self.doc.clone();
        // Fork at specific heads
        let forked = doc.fork_at(heads)?;
        Ok(forked)
    }

    // =========================================================================
    // Helpers
    // =========================================================================

    /// Get entity object ID, returning error if not found
    fn get_entity_obj(&self, id: &str) -> MergeResult<ObjId> {
        match self.doc.get(&self.entities_obj, id)? {
            Some((Value::Object(ObjType::Map), obj_id)) => Ok(obj_id),
            Some(_) => Err(MergeError::InvalidData(format!(
                "Entity {} is not an object",
                id
            ))),
            None => Err(MergeError::NotFound {
                entity: self.entity_type.clone(),
                id: id.to_string(),
            }),
        }
    }

    /// Update entity metadata (updated_at)
    fn update_entity_meta(&mut self, entity_obj: &ObjId) -> MergeResult<()> {
        if let Some((Value::Object(ObjType::Map), meta_obj)) = self.doc.get(entity_obj, "_meta")? {
            self.doc
                .put(&meta_obj, "updated_at", Utc::now().to_rfc3339().as_str())?;
        }
        Ok(())
    }
}

/// Information about a change in history
#[derive(Debug, Clone)]
pub struct ChangeInfo {
    pub hash: String,
    pub timestamp: i64,
    pub actor: String,
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_and_read() {
        let mut doc = EntityDocument::new("users").unwrap();

        doc.create(
            "user_1",
            json!({
                "name": "Alice",
                "email": "alice@example.com",
                "age": 30
            }),
        )
        .unwrap();

        let user = doc.read("user_1").unwrap().unwrap();
        assert_eq!(user["name"], "Alice");
        assert_eq!(user["email"], "alice@example.com");
        assert_eq!(user["age"], 30);
        assert!(user["_meta"]["created_at"].is_string());
    }

    #[test]
    fn test_update_field() {
        let mut doc = EntityDocument::new("users").unwrap();

        doc.create("user_1", json!({"name": "Alice", "email": "alice@example.com"}))
            .unwrap();

        doc.update_field("user_1", "name", json!("Alice Smith"))
            .unwrap();

        let user = doc.read("user_1").unwrap().unwrap();
        assert_eq!(user["name"], "Alice Smith");
        assert_eq!(user["email"], "alice@example.com");
    }

    #[test]
    fn test_delete() {
        let mut doc = EntityDocument::new("users").unwrap();

        doc.create("user_1", json!({"name": "Alice"})).unwrap();
        assert!(doc.exists("user_1").unwrap());

        doc.delete("user_1").unwrap();
        assert!(!doc.exists("user_1").unwrap());
    }

    #[test]
    fn test_list() {
        let mut doc = EntityDocument::new("users").unwrap();

        doc.create("user_1", json!({"name": "Alice"})).unwrap();
        doc.create("user_2", json!({"name": "Bob"})).unwrap();

        let users = doc.list().unwrap();
        assert_eq!(users.len(), 2);
    }

    #[test]
    fn test_save_and_load() {
        let mut doc = EntityDocument::new("users").unwrap();
        doc.create("user_1", json!({"name": "Alice"})).unwrap();

        let bytes = doc.save();

        let doc2 = EntityDocument::load("users", &bytes).unwrap();
        let user = doc2.read("user_1").unwrap().unwrap();
        assert_eq!(user["name"], "Alice");
    }

    #[test]
    fn test_merge() {
        // Create two documents
        let mut doc1 = EntityDocument::new("users").unwrap();
        let mut doc2 = EntityDocument::new("users").unwrap();

        // Make changes in both
        doc1.create("user_1", json!({"name": "Alice"})).unwrap();
        doc2.create("user_2", json!({"name": "Bob"})).unwrap();

        // Merge doc2 into doc1
        doc1.merge(doc2.doc_mut()).unwrap();

        // Both entities should exist
        assert!(doc1.exists("user_1").unwrap());
        assert!(doc1.exists("user_2").unwrap());
    }

    #[test]
    fn test_concurrent_update() {
        // Simulate concurrent updates
        let mut doc1 = EntityDocument::new("users").unwrap();
        doc1.create("user_1", json!({"name": "Alice", "count": 0}))
            .unwrap();

        // Fork the document
        let bytes = doc1.save();
        let mut doc2 = EntityDocument::load("users", &bytes).unwrap();

        // Make different updates
        doc1.update_field("user_1", "name", json!("Alice Smith"))
            .unwrap();
        doc2.update_field("user_1", "count", json!(1)).unwrap();

        // Merge - both changes should be preserved!
        doc1.merge(doc2.doc_mut()).unwrap();

        let user = doc1.read("user_1").unwrap().unwrap();
        assert_eq!(user["name"], "Alice Smith"); // From doc1
        assert_eq!(user["count"], 1); // From doc2
    }

    #[test]
    fn test_nested_objects() {
        let mut doc = EntityDocument::new("users").unwrap();

        doc.create(
            "user_1",
            json!({
                "name": "Alice",
                "address": {
                    "street": "123 Main St",
                    "city": "Springfield"
                },
                "tags": ["admin", "user"]
            }),
        )
        .unwrap();

        let user = doc.read("user_1").unwrap().unwrap();
        assert_eq!(user["address"]["city"], "Springfield");
        assert_eq!(user["tags"][0], "admin");
    }
}
