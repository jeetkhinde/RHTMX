//! # rusty-merge
//!
//! Automerge-based real-time synchronization for Rust applications.
//!
//! This crate provides automatic CRDT-based sync with:
//! - Conflict-free merging (no data loss)
//! - Offline-first with automatic sync
//! - Full change history and time travel
//! - Undo/redo support
//! - PostgreSQL persistence with SQL query support
//! - WebSocket real-time sync
//! - WASM client for browsers
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use rusty_merge::{MergeEngine, MergeConfig};
//!
//! // Initialize the sync engine
//! let config = MergeConfig::new("postgres://localhost/mydb")
//!     .with_entities(vec!["users", "posts", "comments"])
//!     .build();
//!
//! let engine = MergeEngine::new(config).await?;
//!
//! // CRUD operations - automatically synced!
//! engine.create("users", "user_1", json!({"name": "Alice", "email": "alice@example.com"})).await?;
//! engine.update("users", "user_1", "name", json!("Alice Smith")).await?;
//!
//! // Add routes to your Axum app
//! let app = Router::new().merge(engine.routes());
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Client (Browser/WASM)                     │
//! │  ┌─────────────┐    ┌──────────────┐    ┌───────────────┐  │
//! │  │ MergeClient │───▶│ Automerge    │───▶│ IndexedDB     │  │
//! │  └─────────────┘    │ Document     │    │ (persistent)  │  │
//! │         │           └──────────────┘    └───────────────┘  │
//! └─────────┼───────────────────────────────────────────────────┘
//!           │ WebSocket (binary sync protocol)
//!           ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Server                                    │
//! │  ┌─────────────┐    ┌──────────────┐    ┌───────────────┐  │
//! │  │ MergeEngine │───▶│ Automerge    │───▶│ PostgreSQL    │  │
//! │  └─────────────┘    │ Document     │    │ (doc store)   │  │
//! │         │           └──────────────┘    └───────────────┘  │
//! │         │                                      │            │
//! │         ▼                                      ▼            │
//! │  ┌─────────────┐                       ┌───────────────┐   │
//! │  │ Projection  │──────────────────────▶│ SQL Tables    │   │
//! │  │ Layer       │                       │ (queryable)   │   │
//! │  └─────────────┘                       └───────────────┘   │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod document;
pub mod error;

#[cfg(feature = "server")]
pub mod engine;
#[cfg(feature = "server")]
pub mod storage;
#[cfg(feature = "server")]
pub mod transport;
#[cfg(feature = "server")]
pub mod projection;

#[cfg(feature = "wasm")]
pub mod client;

// Re-exports
pub use document::{EntityDocument, DocumentChange};
pub use error::{MergeError, MergeResult};

#[cfg(feature = "server")]
pub use engine::{MergeEngine, MergeConfig};
#[cfg(feature = "server")]
pub use storage::PostgresStorage;
#[cfg(feature = "server")]
pub use transport::{ws_handler, WebSocketState};
#[cfg(feature = "server")]
pub use projection::ProjectionManager;

/// Protocol version for sync compatibility
pub const PROTOCOL_VERSION: &str = "1.0.0";

/// Maximum document size (10MB)
pub const MAX_DOC_SIZE: usize = 10 * 1024 * 1024;

/// Default sync batch size
pub const DEFAULT_BATCH_SIZE: usize = 100;
