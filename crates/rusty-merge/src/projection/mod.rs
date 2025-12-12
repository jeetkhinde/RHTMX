//! Projection layer for SQL queries
//!
//! This module maintains SQL-queryable projections of Automerge document state.
//! While Automerge documents are stored as binary blobs, projections allow you
//! to query entity data using standard SQL.
//!
//! Each entity type gets its own table with:
//! - `id` (VARCHAR PRIMARY KEY)
//! - `data` (JSONB) - the entity data
//! - `_meta` (JSONB) - metadata (created_at, updated_at)
//! - `updated_at` (TIMESTAMP)

use serde_json::Value as JsonValue;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

use crate::error::{MergeError, MergeResult};

/// Manages SQL projections of Automerge state
pub struct ProjectionManager {
    pool: PgPool,
}

impl ProjectionManager {
    /// Create a new projection manager
    pub async fn new(database_url: &str) -> MergeResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| MergeError::Database(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Get the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Ensure projection table exists for an entity type
    pub async fn ensure_table(&self, entity_type: &str) -> MergeResult<()> {
        // Sanitize table name (only allow alphanumeric and underscore)
        let table_name = sanitize_table_name(entity_type);

        let query = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id VARCHAR(255) PRIMARY KEY,
                data JSONB NOT NULL DEFAULT '{{}}',
                _meta JSONB DEFAULT '{{}}',
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#,
            table_name
        );

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| MergeError::Projection(e.to_string()))?;

        // Create index on updated_at for efficient sync queries
        let index_query = format!(
            "CREATE INDEX IF NOT EXISTS idx_{}_updated ON {}(updated_at)",
            table_name, table_name
        );

        sqlx::query(&index_query)
            .execute(&self.pool)
            .await
            .map_err(|e| MergeError::Projection(e.to_string()))?;

        // Create GIN index on data for JSONB queries
        let gin_query = format!(
            "CREATE INDEX IF NOT EXISTS idx_{}_data ON {} USING GIN (data)",
            table_name, table_name
        );

        sqlx::query(&gin_query)
            .execute(&self.pool)
            .await
            .map_err(|e| MergeError::Projection(e.to_string()))?;

        Ok(())
    }

    /// Project an entity to its SQL table
    pub async fn project_entity(
        &self,
        entity_type: &str,
        id: &str,
        data: &JsonValue,
    ) -> MergeResult<()> {
        // Ensure table exists
        self.ensure_table(entity_type).await?;

        let table_name = sanitize_table_name(entity_type);

        // Extract metadata
        let meta = data.get("_meta").cloned().unwrap_or(serde_json::json!({}));

        // Create data without _meta for the data column
        let mut entity_data = data.clone();
        if let Some(obj) = entity_data.as_object_mut() {
            obj.remove("_meta");
        }

        let query = format!(
            r#"
            INSERT INTO {} (id, data, _meta, updated_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (id) DO UPDATE SET
                data = EXCLUDED.data,
                _meta = EXCLUDED._meta,
                updated_at = NOW()
            "#,
            table_name
        );

        sqlx::query(&query)
            .bind(id)
            .bind(&entity_data)
            .bind(&meta)
            .execute(&self.pool)
            .await
            .map_err(|e| MergeError::Projection(e.to_string()))?;

        Ok(())
    }

    /// Delete an entity from its projection
    pub async fn delete_entity(&self, entity_type: &str, id: &str) -> MergeResult<()> {
        let table_name = sanitize_table_name(entity_type);

        let query = format!("DELETE FROM {} WHERE id = $1", table_name);

        sqlx::query(&query)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| MergeError::Projection(e.to_string()))?;

        Ok(())
    }

    /// Query entities using SQL
    pub async fn query(
        &self,
        entity_type: &str,
        where_clause: Option<&str>,
        order_by: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> MergeResult<Vec<JsonValue>> {
        let table_name = sanitize_table_name(entity_type);

        let mut query = format!(
            "SELECT id, data, _meta, created_at, updated_at FROM {}",
            table_name
        );

        if let Some(where_clause) = where_clause {
            query.push_str(" WHERE ");
            query.push_str(where_clause);
        }

        if let Some(order_by) = order_by {
            query.push_str(" ORDER BY ");
            query.push_str(order_by);
        }

        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| MergeError::Projection(e.to_string()))?;

        let results: Vec<JsonValue> = rows
            .iter()
            .map(|row| {
                let id: String = row.get("id");
                let data: JsonValue = row.get("data");
                let meta: JsonValue = row.get("_meta");

                let mut result = data;
                if let Some(obj) = result.as_object_mut() {
                    obj.insert("id".to_string(), JsonValue::String(id));
                    obj.insert("_meta".to_string(), meta);
                }
                result
            })
            .collect();

        Ok(results)
    }

    /// Count entities matching a condition
    pub async fn count(&self, entity_type: &str, where_clause: Option<&str>) -> MergeResult<i64> {
        let table_name = sanitize_table_name(entity_type);

        let mut query = format!("SELECT COUNT(*) FROM {}", table_name);

        if let Some(where_clause) = where_clause {
            query.push_str(" WHERE ");
            query.push_str(where_clause);
        }

        let count: (i64,) = sqlx::query_as(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| MergeError::Projection(e.to_string()))?;

        Ok(count.0)
    }

    /// Get entity by ID
    pub async fn get(&self, entity_type: &str, id: &str) -> MergeResult<Option<JsonValue>> {
        let table_name = sanitize_table_name(entity_type);

        let query = format!(
            "SELECT id, data, _meta FROM {} WHERE id = $1",
            table_name
        );

        let row = sqlx::query(&query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| MergeError::Projection(e.to_string()))?;

        match row {
            Some(row) => {
                let id: String = row.get("id");
                let data: JsonValue = row.get("data");
                let meta: JsonValue = row.get("_meta");

                let mut result = data;
                if let Some(obj) = result.as_object_mut() {
                    obj.insert("id".to_string(), JsonValue::String(id));
                    obj.insert("_meta".to_string(), meta);
                }
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    /// Query entities where a JSON field matches a value
    pub async fn query_by_field(
        &self,
        entity_type: &str,
        field: &str,
        value: &JsonValue,
    ) -> MergeResult<Vec<JsonValue>> {
        let where_clause = format!("data->>'{}' = '{}'", field, value);
        self.query(entity_type, Some(&where_clause), None, None, None)
            .await
    }

    /// Full-text search on a JSON field
    pub async fn search(
        &self,
        entity_type: &str,
        field: &str,
        search_term: &str,
    ) -> MergeResult<Vec<JsonValue>> {
        let where_clause = format!("data->>'{}' ILIKE '%{}%'", field, search_term);
        self.query(entity_type, Some(&where_clause), None, None, None)
            .await
    }

    /// Rebuild projection from Automerge document
    pub async fn rebuild(
        &self,
        entity_type: &str,
        entities: &[(String, JsonValue)],
    ) -> MergeResult<()> {
        let table_name = sanitize_table_name(entity_type);

        // Clear existing data
        let clear_query = format!("DELETE FROM {}", table_name);
        sqlx::query(&clear_query)
            .execute(&self.pool)
            .await
            .map_err(|e| MergeError::Projection(e.to_string()))?;

        // Insert all entities
        for (id, data) in entities {
            self.project_entity(entity_type, id, data).await?;
        }

        Ok(())
    }
}

/// Sanitize table name to prevent SQL injection
fn sanitize_table_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_table_name() {
        assert_eq!(sanitize_table_name("users"), "users");
        assert_eq!(sanitize_table_name("user_profiles"), "user_profiles");
        assert_eq!(sanitize_table_name("Users"), "users");
        assert_eq!(sanitize_table_name("users; DROP TABLE users;"), "usersdroptableusers");
    }
}
