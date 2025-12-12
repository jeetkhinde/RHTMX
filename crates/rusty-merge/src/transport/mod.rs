//! Transport layer for real-time sync
//!
//! This module provides WebSocket-based real-time synchronization.

mod websocket;
mod message;

pub use websocket::{ws_handler, WebSocketState};
pub use message::SyncMessage;
