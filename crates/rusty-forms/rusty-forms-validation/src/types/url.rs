//! URL validated types using nutype

use nutype::nutype;
use crate::string::is_valid_url as validate_url;

/// Valid URL address
///
/// **Business Rule**: Accepts any valid URL (http, https, ftp, etc.)
///
/// **Use when**: You need to validate URL format
#[nutype(
    validate(predicate = is_valid_url_type),
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
pub struct UrlAddress(String);

/// HTTPS-only URL
///
/// **Business Rule**: Only accepts HTTPS URLs (secure connections only)
///
/// **Use when**: You need to enforce secure connections
#[nutype(
    validate(predicate = is_https_url),
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
pub struct HttpsUrl(String);

// URL validation predicates using existing validation function
fn is_valid_url_type(s: &str) -> bool {
    validate_url(s)
}

#[cfg(feature = "rfc-url")]
fn is_https_url(s: &str) -> bool {
    match url::Url::parse(s) {
        Ok(parsed) => parsed.scheme() == "https",
        Err(_) => false,
    }
}

#[cfg(not(feature = "rfc-url"))]
fn is_https_url(s: &str) -> bool {
    s.starts_with("https://") && validate_url(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_address() {
        assert!(UrlAddress::try_new("https://example.com".to_string()).is_ok());
        assert!(UrlAddress::try_new("http://example.com".to_string()).is_ok());
        assert!(UrlAddress::try_new("not-a-url".to_string()).is_err());
        assert!(UrlAddress::try_new("http://".to_string()).is_err()); // No domain
    }

    #[test]
    fn test_https_url() {
        assert!(HttpsUrl::try_new("https://example.com".to_string()).is_ok());
        assert!(HttpsUrl::try_new("http://example.com".to_string()).is_err()); // Must be HTTPS
    }
}
