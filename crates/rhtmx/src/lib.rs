// RHTMX - Rust + HTMX Framework
// Compile-time HTML generation with type safety and zero runtime overhead

pub mod form_field;
pub mod html;
pub mod value;

// Framework modules
pub mod action_executor;
pub mod actions;
pub mod config;
pub mod database;
pub mod request_context;
pub mod template_loader;

// Re-export the html! macro from rhtmx-macro
pub use rhtmx_macro::{css, delete, get, html, patch, post, put};

// Note: Validate and FormField derive macros are defined in rhtmx-macro

// Re-export core types and response builders
pub use html::{
    error, ok, redirect, Error, ErrorResponse, Html, Ok, OkResponse, Redirect, RedirectResponse,
};

// Re-export framework types
pub use actions::ActionRegistry;
pub use config::Config;
pub use request_context::{FormData, QueryParams, RequestContext};
pub use template_loader::TemplateLoader;

// Re-export form field types
pub use form_field::{FieldAttrs, FormField};

// Re-export commonly used types from dependencies
pub use axum;
pub use axum::http::StatusCode;
