//! String validated types using nutype

use nutype::nutype;

/// Non-empty string
#[nutype(
    validate(not_empty),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Display,
        Serialize,
        Deserialize,
    )
)]
pub struct NonEmptyString(String);

/// Username (3-30 characters, alphanumeric + underscore/dash)
#[nutype(
    validate(
        len_char_min = 3,
        len_char_max = 30,
        predicate = is_valid_username
    ),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Display,
        Serialize,
        Deserialize,
    )
)]
pub struct Username(String);

fn is_valid_username(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

// =============================================================================
// Content Moderation Types
// =============================================================================

#[cfg(feature = "content-moderation")]
/// Safe string (no profanity or inappropriate content)
///
/// **Business Rule**: Content must not contain profanity or inappropriate language
///
/// **Use when**: User-generated content, comments, usernames, profile descriptions
///
/// # Example
///
/// ```rust,ignore
/// use rusty_forms_validation::types::SafeString;
///
/// let clean = SafeString::try_new("Hello, world!".to_string())?;  // ✓
/// let bad = SafeString::try_new("inappropriate content".to_string());  // ✗
/// ```
///
/// **Features:**
/// - Multi-language support
/// - Detects leetspeak (l33t)
/// - Detects common evasion patterns
/// - Customizable sensitivity
#[nutype(
    validate(predicate = is_safe_content),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Display,
        Serialize,
        Deserialize,
    )
)]
pub struct SafeString(String);

#[cfg(feature = "content-moderation")]
fn is_safe_content(s: &str) -> bool {
    !rustrict::CensorStr::is_inappropriate(s)
}

#[cfg(feature = "content-moderation")]
/// Safe username (no profanity, alphanumeric + basic chars)
///
/// **Business Rule**: Safe for public display, no inappropriate content
///
/// **Use when**: Public usernames, display names
#[nutype(
    validate(
        len_char_min = 3,
        len_char_max = 30,
        predicate = is_safe_username
    ),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        Hash,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Display,
        Serialize,
        Deserialize,
    )
)]
pub struct SafeUsername(String);

#[cfg(feature = "content-moderation")]
fn is_safe_username(s: &str) -> bool {
    // Must be alphanumeric + underscore/dash
    let valid_chars = s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-');
    // Must not contain profanity
    let is_appropriate = !rustrict::CensorStr::is_inappropriate(s);

    valid_chars && is_appropriate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username() {
        assert!(Username::try_new("ab".to_string()).is_err()); // Too short
        assert!(Username::try_new("abc".to_string()).is_ok());
        assert!(Username::try_new("user_name".to_string()).is_ok());
        assert!(Username::try_new("user-name".to_string()).is_ok());
        assert!(Username::try_new("user@name".to_string()).is_err()); // Invalid char
    }

    #[cfg(feature = "content-moderation")]
    #[test]
    fn test_safe_string() {
        assert!(SafeString::try_new("Hello, world!".to_string()).is_ok());
        assert!(SafeString::try_new("This is a clean message.".to_string()).is_ok());
    }

    #[cfg(feature = "content-moderation")]
    #[test]
    fn test_safe_username() {
        assert!(SafeUsername::try_new("john_doe".to_string()).is_ok());
        assert!(SafeUsername::try_new("user-123".to_string()).is_ok());
        assert!(SafeUsername::try_new("ab".to_string()).is_err()); // Too short
        assert!(SafeUsername::try_new("user@name".to_string()).is_err()); // Invalid char
    }
}
