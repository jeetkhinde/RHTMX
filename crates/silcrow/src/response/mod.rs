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

/// Framework result type.
pub type SilcrowResult<T> = std::result::Result<T, ErrorResponse>;
