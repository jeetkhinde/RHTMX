//! Rusty-Forms-Validation Core
//!
//! Pure Rust validation functions compatible with both std and no_std environments.
//! Used by both server-side validation and WASM client-side validation.
//!
//! This crate provides two main APIs:
//!
//! 1. **Validation Functions** - Pure functions for validating data (no_std compatible)
//! 2. **Validated Types** - Type-safe wrappers using nutype (requires nutype/serde dependencies)
//!
//! # Validation Functions
//!
//! Use these for flexible, composable validation:
//! ```rust,ignore
//! use rusty_forms_validation::is_valid_email;
//!
//! if is_valid_email("user@example.com") {
//!     // Valid email
//! }
//! ```
//!
//! # Validated Types
//!
//! Use these for type-safe validation at construction time:
//! ```rust,ignore
//! use rusty_forms_validation::types::EmailAddress;
//!
//! let email = EmailAddress::try_new("user@example.com".to_string())?;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

// Validation function modules
pub mod collection;
pub mod email;
pub mod numeric;
pub mod password;
pub mod string;

// Validated type modules
pub mod types;

// Re-export all validation functions
pub use collection::*;
pub use email::*;
pub use numeric::*;
pub use password::*;
pub use string::*;

/// Core validation trait that all forms implement
///
/// This trait is automatically implemented when you use `#[derive(Validate)]`
pub trait Validate {
    /// Validate the form and return errors by field name
    fn validate(&self) -> Result<(), BTreeMap<String, Vec<String>>>;
}

/// Form field attributes for HTML5 and client-side validation
///
/// This trait is automatically implemented when you use `#[derive(FormField)]`
pub trait FormField {
    /// Get validation attributes for a specific field
    fn field_attrs(&self, field_name: &str) -> FieldAttrs;

    /// Get list of all field names
    fn field_names(&self) -> Vec<&'static str>;
}

/// Attributes for a form field (HTML5 + data-validate JSON)
#[derive(Debug, Clone, Default)]
pub struct FieldAttrs {
    /// HTML5 validation attributes (type, required, min, max, etc.)
    pub html5_attrs: BTreeMap<String, String>,

    /// JSON for data-validate attribute (for WASM validation)
    pub data_validate: String,
}
