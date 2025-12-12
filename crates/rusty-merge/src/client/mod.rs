//! Client-side sync implementation
//!
//! This module provides:
//! - JavaScript client for browser-based sync (`rusty-merge.js`)
//! - WASM client for Rust-based browser apps (with `wasm` feature)

// The JavaScript client is served via include_str! in engine.rs
// See: src/client/rusty-merge.js

#[cfg(feature = "wasm")]
mod wasm;

#[cfg(feature = "wasm")]
pub use wasm::*;
