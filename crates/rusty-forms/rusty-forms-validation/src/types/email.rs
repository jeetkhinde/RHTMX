//! Email validated types using nutype

use nutype::nutype;
use crate::email::{is_valid_email, is_public_domain};

/// Public email domains to block (Gmail, Yahoo, Hotmail, etc.)
const PUBLIC_DOMAINS: &[&str] = &[
    "gmail.com",
    "yahoo.com",
    "hotmail.com",
    "outlook.com",
    "aol.com",
    "icloud.com",
    "mail.com",
    "protonmail.com",
    "zoho.com",
    "yandex.com",
    "live.com",
    "msn.com",
    "inbox.com",
    "gmx.com",
    "me.com",
];

/// Always-blocked domains (disposable/temporary email services)
const BLOCKED_DOMAINS: &[&str] = &[
    "tempmail.com",
    "guerrillamail.com",
    "10minutemail.com",
    "mailinator.com",
    "throwaway.email",
    "temp-mail.org",
    "maildrop.cc",
    "getnada.com",
];

/// Basic validated email address (format only, blocks disposable)
///
/// **Business Rule**: Accepts any email domain EXCEPT disposable/temporary email services.
///
/// **Use when**: You want to accept both personal (Gmail, Yahoo) and work emails,
/// but block throwaway addresses.
///
/// # Example
///
/// ```rust,ignore
/// use rusty_forms_validation::types::EmailAddress;
///
/// // Valid - any real domain
/// let personal = EmailAddress::try_new("user@gmail.com".to_string())?;  // ✓
/// let work = EmailAddress::try_new("user@company.com".to_string())?;     // ✓
///
/// // Invalid - disposable email blocked
/// let bad = EmailAddress::try_new("user@tempmail.com".to_string()); // ✗
/// ```
#[nutype(
    validate(predicate = is_valid_email_any_domain),
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
pub struct EmailAddress(String);

/// Alias: Any email address (same as EmailAddress)
///
/// **Business Rule**: Same as `EmailAddress` - blocks disposable only.
///
/// Use this when you want to be explicit that any email is accepted.
pub type AnyEmailAddress = EmailAddress;

/// Work email address (no public domains)
///
/// **Business Rule**: Blocks public email providers (Gmail, Yahoo, Hotmail, etc.)
/// AND disposable email services. Only accepts corporate/private domains.
///
/// **Use when**: Registration should use work/corporate email only (B2B apps, enterprise tools).
///
/// # Example
///
/// ```rust,ignore
/// use rusty_forms_validation::types::WorkEmailAddress;
///
/// // Valid - corporate domain
/// let work = WorkEmailAddress::try_new("john@acme.com".to_string())?;  // ✓
///
/// // Invalid - public domain
/// let gmail = WorkEmailAddress::try_new("john@gmail.com".to_string()); // ✗
/// let yahoo = WorkEmailAddress::try_new("john@yahoo.com".to_string()); // ✗
/// ```
#[nutype(
    validate(predicate = is_work_email_type),
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
pub struct WorkEmailAddress(String);

/// Business email address (stricter than WorkEmailAddress)
///
/// **Business Rule**: Same as WorkEmailAddress for now.
/// Future: Can be extended with domain allowlist for verified partners.
///
/// **Use when**: You need maximum validation (verified corporate domains only).
///
/// # Example
///
/// ```rust,ignore
/// use rusty_forms_validation::types::BusinessEmailAddress;
///
/// let biz = BusinessEmailAddress::try_new("ceo@verified-corp.com".to_string())?;
/// ```
#[nutype(
    validate(predicate = is_business_email_type),
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
pub struct BusinessEmailAddress(String);

// -----------------------------------------------------------------------------
// Email validation predicates using existing validation functions
// -----------------------------------------------------------------------------

fn extract_domain(email: &str) -> &str {
    email.split('@').nth(1).unwrap_or("")
}

fn is_blocked_domain(domain: &str) -> bool {
    let domain_lower = domain.to_lowercase();
    BLOCKED_DOMAINS.iter().any(|&d| d == domain_lower)
}

fn is_public_domain_check(domain: &str) -> bool {
    let domain_lower = domain.to_lowercase();
    PUBLIC_DOMAINS.iter().any(|&d| d == domain_lower)
}

fn is_valid_email_any_domain(s: &str) -> bool {
    if !is_valid_email(s) {
        return false;
    }
    let domain = extract_domain(s);
    !is_blocked_domain(domain)
}

fn is_work_email_type(s: &str) -> bool {
    if !is_valid_email(s) {
        return false;
    }
    let domain = extract_domain(s);
    !is_blocked_domain(domain) && !is_public_domain_check(domain)
}

fn is_business_email_type(s: &str) -> bool {
    // Same as work email for now
    // Future: check against allowlist of verified corporate domains
    is_work_email_type(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_address_any_domain() {
        // Accepts personal email
        assert!(EmailAddress::try_new("user@gmail.com".to_string()).is_ok());
        // Accepts work email
        assert!(EmailAddress::try_new("user@company.com".to_string()).is_ok());
        // Blocks disposable
        assert!(EmailAddress::try_new("user@tempmail.com".to_string()).is_err());
    }

    #[test]
    fn test_work_email_blocks_public() {
        // Accepts corporate email
        assert!(WorkEmailAddress::try_new("user@acme.com".to_string()).is_ok());
        // Blocks Gmail
        assert!(WorkEmailAddress::try_new("user@gmail.com".to_string()).is_err());
        // Blocks Yahoo
        assert!(WorkEmailAddress::try_new("user@yahoo.com".to_string()).is_err());
        // Blocks disposable
        assert!(WorkEmailAddress::try_new("user@tempmail.com".to_string()).is_err());
    }

    #[test]
    fn test_business_email() {
        // Same as work email for now
        assert!(BusinessEmailAddress::try_new("ceo@corp.com".to_string()).is_ok());
        assert!(BusinessEmailAddress::try_new("user@gmail.com".to_string()).is_err());
    }
}
