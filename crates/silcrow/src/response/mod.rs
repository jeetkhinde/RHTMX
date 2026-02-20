// silcrow/src/response/mod.rs

pub mod base;
pub mod error;
pub mod html;
pub mod json;
pub mod redirect;

pub use error::ErrorResponse;
pub use html::HtmlOkResponse;
pub use json::JsonOkResponse;
pub use redirect::RedirectResponse;

// Clean public type aliases (convenience usage for Silcrow-oriented responses)
pub type Json = JsonOkResponse;
pub type Html = HtmlOkResponse;
pub type Redirect = RedirectResponse;
pub type Error = ErrorResponse;

/// Framework result type
pub type Result<T> = std::result::Result<T, ErrorResponse>;
