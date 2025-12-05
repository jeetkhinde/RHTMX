//! Validated types using nutype
//!
//! This module provides reusable, validated newtype wrappers using the `nutype` crate.
//! These types ensure domain constraints AND business rules are enforced at the type level.
//!
//! # Philosophy: Business Rules in Types
//!
//! Instead of combining type validation + form validators:
//! ```rust,ignore
//! #[nutype]
//! #[no_public_domains]  // ← Business rule at form level
//! email: EmailAddress
//! ```
//!
//! **Embed business rules directly in the type**:
//! ```rust,ignore
//! email: WorkEmailAddress  // ← Type IS the business rule!
//! ```
//!
//! # WASM Compatibility
//!
//! All types work in WebAssembly environments:
//! - Serializable/deserializable with serde
//! - Validation happens at construction time
//! - Same types on server and client
//!
//! # Type Hierarchies
//!
//! ## Email Types
//! - `EmailAddress` / `AnyEmailAddress` - Any valid email (blocks disposable only)
//! - `WorkEmailAddress` - No public domains (Gmail, Yahoo, etc.)
//! - `BusinessEmailAddress` - Only corporate/verified domains
//!
//! ## Password Types
//! - `PasswordBasic` - 6+ characters
//! - `PasswordMedium` - 8+ characters + complexity
//! - `PasswordStrong` - 10+ characters + high complexity
//! - `PasswordPhrase` - 15+ characters (passphrase style)
//! - `PasswordPhrase3` - 3+ words, 20+ characters
//! - `SuperStrongPassword` - 12+ characters + all character types
//! - `ModernPassword` - 16+ characters (NIST 2024 recommendations)
//! - `EntropyPassword` - zxcvbn score >= 3 (requires `password-strength` feature)
//! - `MaxEntropyPassword` - zxcvbn score = 4 (requires `password-strength` feature)
//!
//! ## String Types
//! - `NonEmptyString` - Cannot be empty
//! - `Username` - 3-30 characters, alphanumeric + underscore/dash
//! - `SafeString` - No profanity (requires `content-moderation` feature)
//! - `SafeUsername` - Safe username (requires `content-moderation` feature)
//!
//! ## Numeric Types
//! - `PositiveInt` - Greater than 0
//! - `NonNegativeInt` - Greater than or equal to 0
//! - `Age` - 18-120 years
//! - `Percentage` - 0-100
//! - `Port` - 1-65535
//!
//! ## URL Types
//! - `UrlAddress` - Any valid URL
//! - `HttpsUrl` - HTTPS-only URLs
//!
//! ## Specialized Types
//! - `PhoneNumber` - US phone number (10 digits)
//! - `InternationalPhoneNumber` - International phone (requires `intl-phone` feature)
//! - `USPhoneNumber` - US phone E.164 format (requires `intl-phone` feature)
//! - `ZipCode` - US zip code (5 or 9 digits)
//! - `IpAddress` - IPv4 address
//! - `Uuid` - UUID v4
//! - `DateString` - ISO 8601 date (requires `datetime` feature)
//! - `DateTimeString` - ISO 8601 datetime (requires `datetime` feature)
//! - `TimeString` - HH:MM:SS time (requires `datetime` feature)
//! - `CreditCardNumber` - Valid credit card (requires `credit-card` feature)
//! - `VisaCardNumber` - Visa card only (requires `credit-card` feature)
//! - `CVVCode` - CVV/CVC code (requires `credit-card` feature)
//! - `NonEmptyVec<T>` - Non-empty vector

pub mod email;
pub mod numbers;
pub mod password;
pub mod specialized;
pub mod strings;
pub mod url;

// Re-export all types for convenient access
pub use email::*;
pub use numbers::*;
pub use password::*;
pub use specialized::*;
pub use strings::*;
pub use url::*;
