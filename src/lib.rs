pub mod response;

pub use axum;
pub use axum::http::StatusCode;
pub use maud;

pub use response::{
    Error, ErrorResponse, HtmlOk, HtmlOkResponse, JsonOk, JsonOkResponse, Ok, OkResponse,
    Redirect, RedirectResponse,
};

/// Parsed Silcrow request metadata from incoming headers.
pub struct SilcrowRequest {
    /// `true` when the `silcrow-target` header is present (request came from Silcrow navigation).
    pub is_silcrow: bool,
    /// `true` when `Accept` header contains `text/html` (element has `s-html` attribute).
    pub wants_html: bool,
    /// `true` when `Accept` header contains `application/json` (default for `s-action` without `s-html`).
    pub wants_json: bool,
}

impl SilcrowRequest {
    pub fn from_headers(headers: &axum::http::HeaderMap) -> Self {
        let accept = headers
            .get(axum::http::header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        Self {
            is_silcrow: headers.get("silcrow-target").is_some(),
            wants_html: accept.contains("text/html"),
            wants_json: accept.contains("application/json"),
        }
    }
}
