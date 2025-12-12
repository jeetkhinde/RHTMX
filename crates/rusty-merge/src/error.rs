//! Error types for rusty-merge

use thiserror::Error;

/// Result type alias for rusty-merge operations
pub type MergeResult<T> = Result<T, MergeError>;

/// Errors that can occur during sync operations
#[derive(Error, Debug)]
pub enum MergeError {
    /// Automerge document error
    #[error("Automerge error: {0}")]
    Automerge(#[from] automerge::AutomergeError),

    /// Entity not found
    #[error("Entity not found: {entity}/{id}")]
    NotFound { entity: String, id: String },

    /// Document not found
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    /// Invalid data format
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// WebSocket error
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Projection error
    #[error("Projection error: {0}")]
    Projection(String),

    /// Document too large
    #[error("Document exceeds maximum size: {size} > {max}")]
    DocumentTooLarge { size: usize, max: usize },

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Sync conflict that requires manual resolution
    #[error("Sync conflict in {entity}/{id}: {message}")]
    Conflict {
        entity: String,
        id: String,
        message: String,
    },

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for MergeError {
    fn from(e: serde_json::Error) -> Self {
        MergeError::Serialization(e.to_string())
    }
}

impl From<postcard::Error> for MergeError {
    fn from(e: postcard::Error) -> Self {
        MergeError::Serialization(e.to_string())
    }
}

#[cfg(feature = "postgres")]
impl From<sqlx::Error> for MergeError {
    fn from(e: sqlx::Error) -> Self {
        MergeError::Database(e.to_string())
    }
}

#[cfg(feature = "postgres")]
impl From<deadpool_postgres::PoolError> for MergeError {
    fn from(e: deadpool_postgres::PoolError) -> Self {
        MergeError::Database(e.to_string())
    }
}

#[cfg(feature = "postgres")]
impl From<tokio_postgres::Error> for MergeError {
    fn from(e: tokio_postgres::Error) -> Self {
        MergeError::Database(e.to_string())
    }
}

/// Error codes for client communication
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    NotFound = 404,
    InvalidData = 400,
    Conflict = 409,
    TooLarge = 413,
    Internal = 500,
}

impl MergeError {
    /// Get the error code for client communication
    pub fn code(&self) -> ErrorCode {
        match self {
            MergeError::NotFound { .. } | MergeError::DocumentNotFound(_) => ErrorCode::NotFound,
            MergeError::InvalidData(_) | MergeError::InvalidOperation(_) => ErrorCode::InvalidData,
            MergeError::Conflict { .. } => ErrorCode::Conflict,
            MergeError::DocumentTooLarge { .. } => ErrorCode::TooLarge,
            _ => ErrorCode::Internal,
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            MergeError::Connection(_) | MergeError::WebSocket(_) | MergeError::Database(_)
        )
    }
}
