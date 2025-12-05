//! Password validated types using nutype

use nutype::nutype;

/// Basic password (6+ characters)
///
/// **Security Level**: Low - Use only for non-critical accounts
///
/// **Business Rule**: Minimum 6 characters. No complexity requirements.
#[nutype(
    validate(len_char_min = 6),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Serialize,
        Deserialize,
    )
)]
pub struct PasswordBasic(String);

/// Medium-strength password (8+ characters)
///
/// **Security Level**: Medium - Standard for most applications
///
/// **Business Rule**: Minimum 8 characters.
/// Recommended to combine with form-level complexity check.
#[nutype(
    validate(len_char_min = 8),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Serialize,
        Deserialize,
    )
)]
pub struct PasswordMedium(String);

/// Strong password (10+ characters with complexity)
///
/// **Security Level**: High - For sensitive operations
///
/// **Business Rule**: Minimum 10 characters + uppercase + lowercase + digit + special
#[nutype(
    validate(
        len_char_min = 10,
        predicate = has_password_complexity_strong
    ),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Serialize,
        Deserialize,
    )
)]
pub struct PasswordStrong(String);

/// Super strong password (12+ characters with all character types)
///
/// **Security Level**: Very High - For admin accounts, financial operations
///
/// **Business Rule**: Minimum 12 characters + uppercase + lowercase + digit + special
/// + at least 2 special characters
#[nutype(
    validate(
        len_char_min = 12,
        predicate = has_password_complexity_super
    ),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Serialize,
        Deserialize,
    )
)]
pub struct SuperStrongPassword(String);

/// Password passphrase (15+ characters, easier to remember)
///
/// **Security Level**: High - Modern approach (xkcd "correct horse battery staple")
///
/// **Business Rule**: Minimum 15 characters. Favors length over complexity.
/// Example: "BlueSky-Mountain-Coffee-2024"
#[nutype(
    validate(len_char_min = 15),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Serialize,
        Deserialize,
    )
)]
pub struct PasswordPhrase(String);

/// Password passphrase with 3+ words (20+ characters)
///
/// **Security Level**: High - Multi-word passphrase
///
/// **Business Rule**: Minimum 20 characters + at least 2 spaces/hyphens (3+ words).
/// Example: "Correct-Horse-Battery-Staple"
#[nutype(
    validate(
        len_char_min = 20,
        predicate = has_multiple_words
    ),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Serialize,
        Deserialize,
    )
)]
pub struct PasswordPhrase3(String);

/// Modern password (16+ characters, NIST 2024 recommendations)
///
/// **Security Level**: Very High - Follows NIST SP 800-63B guidelines
///
/// **Business Rule**: Minimum 16 characters. Emphasizes length over complexity.
/// No forced special characters (reduces user friction).
#[nutype(
    validate(len_char_min = 16),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Serialize,
        Deserialize,
    )
)]
pub struct ModernPassword(String);

// -----------------------------------------------------------------------------
// Password validation predicates
// -----------------------------------------------------------------------------

fn has_password_complexity_strong(s: &str) -> bool {
    let has_upper = s.chars().any(|c| c.is_uppercase());
    let has_lower = s.chars().any(|c| c.is_lowercase());
    let has_digit = s.chars().any(|c| c.is_numeric());
    let has_special = s.chars().any(|c| !c.is_alphanumeric());

    has_upper && has_lower && has_digit && has_special
}

fn has_password_complexity_super(s: &str) -> bool {
    let has_upper = s.chars().any(|c| c.is_uppercase());
    let has_lower = s.chars().any(|c| c.is_lowercase());
    let has_digit = s.chars().any(|c| c.is_numeric());
    let special_count = s.chars().filter(|c| !c.is_alphanumeric()).count();

    has_upper && has_lower && has_digit && special_count >= 2
}

fn has_multiple_words(s: &str) -> bool {
    // Count spaces, hyphens, or underscores (word separators)
    let separator_count = s
        .chars()
        .filter(|&c| c == ' ' || c == '-' || c == '_')
        .count();

    separator_count >= 2 // At least 2 separators = 3+ words
}

// =============================================================================
// Password Strength Types
// =============================================================================

#[cfg(feature = "password-strength")]
/// High-entropy password (zxcvbn score >= 3)
///
/// **Security Level**: High - Based on actual password strength, not just rules
///
/// **Business Rule**: Password must have a zxcvbn score of 3 or higher
/// (out of 4, where 4 is strongest)
///
/// **Use when**: You want actual password strength, not just character rules
///
/// # Example
///
/// ```rust,ignore
/// use rusty_forms_validation::types::EntropyPassword;
///
/// // Strong password (score 3+)
/// let strong = EntropyPassword::try_new("correct-horse-battery-staple".to_string())?;  // ✓
///
/// // Weak password (low entropy)
/// let weak = EntropyPassword::try_new("Password123!".to_string());  // ✗ Too common
/// ```
///
/// **Why better than rule-based?**
/// - Detects dictionary words
/// - Detects common patterns
/// - Detects keyboard patterns (qwerty, etc.)
/// - Accounts for actual entropy, not just length
#[nutype(
    validate(
        len_char_min = 8,
        predicate = has_high_entropy
    ),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Serialize,
        Deserialize,
    )
)]
pub struct EntropyPassword(String);

#[cfg(feature = "password-strength")]
fn has_high_entropy(s: &str) -> bool {
    let entropy = zxcvbn::zxcvbn(s, &[]);
    match entropy.score() {
        zxcvbn::Score::Three | zxcvbn::Score::Four => true,
        _ => false,
    }
}

#[cfg(feature = "password-strength")]
/// Very strong password (zxcvbn score = 4)
///
/// **Security Level**: Maximum - Only accepts passwords with perfect entropy
///
/// **Business Rule**: Password must have a zxcvbn score of 4 (maximum)
///
/// **Use when**: Admin accounts, financial operations, high-security systems
#[nutype(
    validate(
        len_char_min = 12,
        predicate = has_maximum_entropy
    ),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Serialize,
        Deserialize,
    )
)]
pub struct MaxEntropyPassword(String);

#[cfg(feature = "password-strength")]
fn has_maximum_entropy(s: &str) -> bool {
    let entropy = zxcvbn::zxcvbn(s, &[]);
    matches!(entropy.score(), zxcvbn::Score::Four)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_basic() {
        assert!(PasswordBasic::try_new("12345".to_string()).is_err()); // Too short
        assert!(PasswordBasic::try_new("123456".to_string()).is_ok()); // Exactly 6
    }

    #[test]
    fn test_password_strong_complexity() {
        // Too short
        assert!(PasswordStrong::try_new("Short1!".to_string()).is_err());
        // Long enough but no special
        assert!(PasswordStrong::try_new("Password123".to_string()).is_err());
        // Missing uppercase
        assert!(PasswordStrong::try_new("password123!".to_string()).is_err());
        // All requirements met
        assert!(PasswordStrong::try_new("Password123!".to_string()).is_ok());
    }

    #[test]
    fn test_super_strong_password() {
        // Needs 12+ chars + 2 special chars
        assert!(SuperStrongPassword::try_new("Pass123!".to_string()).is_err()); // Too short
        assert!(SuperStrongPassword::try_new("Password123!".to_string()).is_err()); // Only 1 special
        assert!(SuperStrongPassword::try_new("Password123!@".to_string()).is_ok()); // ✓
    }

    #[test]
    fn test_password_phrase() {
        assert!(PasswordPhrase::try_new("short".to_string()).is_err());
        assert!(PasswordPhrase::try_new("BlueSky-Mountain".to_string()).is_ok()); // 16 chars
    }

    #[test]
    fn test_password_phrase3() {
        // Needs 20+ chars + 2+ separators (3+ words)
        assert!(PasswordPhrase3::try_new("Short-Phrase".to_string()).is_err()); // Too short (12 chars)
        assert!(PasswordPhrase3::try_new("OnlySingleWordHereNoSeparators".to_string()).is_err()); // No separators
        assert!(PasswordPhrase3::try_new("Correct-Horse-Battery-Staple".to_string()).is_ok()); // ✓ (28 chars, 3 separators)
    }

    #[test]
    fn test_modern_password() {
        assert!(ModernPassword::try_new("tooshort".to_string()).is_err());
        assert!(ModernPassword::try_new("ThisIsMyLongPassword123".to_string()).is_ok()); // 23 chars
    }

    #[cfg(feature = "password-strength")]
    #[test]
    fn test_entropy_password() {
        // Strong passwords (high entropy)
        assert!(EntropyPassword::try_new("correct-horse-battery-staple".to_string()).is_ok());
        assert!(EntropyPassword::try_new("MyVeryLongAndComplexPassword2024!".to_string()).is_ok());

        // Weak passwords (low entropy) - these should fail
        assert!(EntropyPassword::try_new("password".to_string()).is_err());
        assert!(EntropyPassword::try_new("12345678".to_string()).is_err());
    }

    #[cfg(feature = "password-strength")]
    #[test]
    fn test_max_entropy_password() {
        // Very strong passwords
        assert!(MaxEntropyPassword::try_new("correct-horse-battery-staple-2024".to_string()).is_ok());

        // Good but not perfect
        assert!(MaxEntropyPassword::try_new("Password123!".to_string()).is_err());
    }
}
