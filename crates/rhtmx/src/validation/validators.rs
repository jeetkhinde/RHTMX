// File: src/validation/validators.rs
// Purpose: Validation helper functions

use regex::Regex;
use once_cell::sync::Lazy;

// Email validation regex
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
});

// Public email domains
const PUBLIC_DOMAINS: &[&str] = &[
    "gmail.com", "yahoo.com", "hotmail.com", "outlook.com",
    "aol.com", "icloud.com", "mail.com", "protonmail.com"
];

/// Check if an email address is valid
pub fn is_valid_email(email: &str) -> bool {
    EMAIL_REGEX.is_match(email)
}

/// Check if email uses a public domain
pub fn is_public_domain(email: &str) -> bool {
    if let Some(domain) = email.split('@').nth(1) {
        PUBLIC_DOMAINS.iter().any(|d| d.eq_ignore_ascii_case(domain))
    } else {
        false
    }
}

/// Check if email uses a blocked domain
pub fn is_blocked_domain(_email: &str) -> bool {
    // Placeholder - implement custom blocked domain logic as needed
    false
}

/// Validate password strength
pub fn validate_password(password: &str, strength: &str) -> Result<(), String> {
    let len = password.len();

    match strength {
        "strong" => {
            if len < 8 {
                return Err("Password must be at least 8 characters".into());
            }
            if !password.chars().any(|c| c.is_uppercase()) {
                return Err("Password must contain uppercase letter".into());
            }
            if !password.chars().any(|c| c.is_lowercase()) {
                return Err("Password must contain lowercase letter".into());
            }
            if !password.chars().any(|c| c.is_ascii_digit()) {
                return Err("Password must contain a number".into());
            }
            if !password.chars().any(|c| !c.is_alphanumeric()) {
                return Err("Password must contain a special character".into());
            }
            Ok(())
        }
        "medium" => {
            if len < 6 {
                return Err("Password must be at least 6 characters".into());
            }
            if !password.chars().any(|c| c.is_alphabetic()) {
                return Err("Password must contain a letter".into());
            }
            if !password.chars().any(|c| c.is_ascii_digit()) {
                return Err("Password must contain a number".into());
            }
            Ok(())
        }
        _ => { // "basic"
            if len < 4 {
                return Err("Password must be at least 4 characters".into());
            }
            Ok(())
        }
    }
}

/// Check if a string is a valid URL
pub fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

// Regex matching (requires std, so kept here)
#[allow(dead_code)]
static URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap()
});

/// Check if string matches regex pattern (std-only function)
pub fn matches_regex(value: &str, pattern: &str) -> bool {
    if let Ok(regex) = Regex::new(pattern) {
        regex.is_match(value)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name+tag@example.co.uk"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("test@"));
    }

    #[test]
    fn test_public_domain() {
        assert!(is_public_domain("user@gmail.com"));
        assert!(is_public_domain("test@YAHOO.COM"));
        assert!(!is_public_domain("admin@company.com"));
    }

    #[test]
    fn test_password_validation() {
        // Strong password
        assert!(validate_password("Pass123!@#", "strong").is_ok());
        assert!(validate_password("weak", "strong").is_err());

        // Medium password
        assert!(validate_password("Pass1234", "medium").is_ok());
        assert!(validate_password("password", "medium").is_err());

        // Basic password
        assert!(validate_password("simple", "basic").is_ok());
        assert!(validate_password("bad", "basic").is_err());
    }

    #[test]
    fn test_url_validation() {
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://sub.example.com/path?query=1"));
        assert!(!is_valid_url("not a url"));
        assert!(!is_valid_url("ftp://example.com"));
    }

    #[test]
    fn test_regex_matching() {
        assert!(matches_regex("123-456-7890", r"^\d{3}-\d{3}-\d{4}$"));
        assert!(!matches_regex("123456789", r"^\d{3}-\d{3}-\d{4}$"));
    }
}
