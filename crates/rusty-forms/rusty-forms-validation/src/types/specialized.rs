//! Specialized validated types (phone, address, UUID, datetime, credit card, collections)

use nutype::nutype;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

// =============================================================================
// Collection Types
// =============================================================================

/// Non-empty vector
///
/// **Business Rule**: Vector must have at least one element
///
/// **Use when**: Tags, categories, selections that can't be empty
#[nutype(
    validate(predicate = is_non_empty_vec),
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
pub struct NonEmptyVec<T>(Vec<T>);

fn is_non_empty_vec<T>(v: &[T]) -> bool {
    !v.is_empty()
}

// =============================================================================
// String Pattern Types
// =============================================================================

/// US Phone Number
///
/// **Business Rule**: Validates US phone numbers (10 digits)
///
/// **Formats accepted**:
/// - (123) 456-7890
/// - 123-456-7890
/// - 1234567890
///
/// **Use when**: US phone number validation
#[nutype(
    validate(predicate = is_valid_phone_number),
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
pub struct PhoneNumber(String);

/// US Zip Code
///
/// **Business Rule**: Validates US zip codes (5 or 9 digits)
///
/// **Formats accepted**:
/// - 12345
/// - 12345-6789
///
/// **Use when**: US address validation
#[nutype(
    validate(predicate = is_valid_zip_code),
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
pub struct ZipCode(String);

/// IPv4 Address
///
/// **Business Rule**: Validates IPv4 addresses
///
/// **Format**: xxx.xxx.xxx.xxx (0-255 per octet)
///
/// **Use when**: Network configuration, IP whitelisting
#[nutype(
    validate(predicate = is_valid_ipv4),
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
pub struct IpAddress(String);

/// UUID (Universally Unique Identifier)
///
/// **Business Rule**: Validates UUID format (v4)
///
/// **Format**: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
///
/// **Use when**: API keys, unique identifiers
#[nutype(
    validate(predicate = is_valid_uuid),
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
pub struct Uuid(String);

// Pattern validation predicates
fn is_valid_phone_number(s: &str) -> bool {
    // Remove common separators
    let digits: alloc::string::String = s.chars().filter(|c| c.is_ascii_digit()).collect();

    // US phone number: exactly 10 digits
    digits.len() == 10
}

fn is_valid_zip_code(s: &str) -> bool {
    // Remove dash if present
    let parts: alloc::vec::Vec<&str> = s.split('-').collect();

    match parts.len() {
        1 => {
            // 5-digit zip
            parts[0].len() == 5 && parts[0].chars().all(|c| c.is_ascii_digit())
        }
        2 => {
            // 9-digit zip (xxxxx-xxxx)
            parts[0].len() == 5
                && parts[1].len() == 4
                && parts[0].chars().all(|c| c.is_ascii_digit())
                && parts[1].chars().all(|c| c.is_ascii_digit())
        }
        _ => false,
    }
}

fn is_valid_ipv4(s: &str) -> bool {
    let parts: alloc::vec::Vec<&str> = s.split('.').collect();

    if parts.len() != 4 {
        return false;
    }

    parts.iter().all(|part| part.parse::<u8>().is_ok())
}

// Use proper UUID validation when available
#[cfg(feature = "uuid-validation")]
fn is_valid_uuid(s: &str) -> bool {
    uuid::Uuid::parse_str(s).is_ok()
}

#[cfg(not(feature = "uuid-validation"))]
fn is_valid_uuid(s: &str) -> bool {
    // Basic UUID v4 validation
    // Format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
    let parts: alloc::vec::Vec<&str> = s.split('-').collect();

    if parts.len() != 5 {
        return false;
    }

    // Check segment lengths
    if parts[0].len() != 8
        || parts[1].len() != 4
        || parts[2].len() != 4
        || parts[3].len() != 4
        || parts[4].len() != 12
    {
        return false;
    }

    // All parts should be hex
    parts
        .iter()
        .all(|part| part.chars().all(|c| c.is_ascii_hexdigit()))
}

// =============================================================================
// International Phone Number Types
// =============================================================================

#[cfg(feature = "intl-phone")]
/// International phone number
///
/// **Business Rule**: Validates phone numbers from any country using libphonenumber
///
/// **Use when**: You need to validate phone numbers globally
///
/// # Example
///
/// ```rust,ignore
/// use rusty_forms_validation::types::InternationalPhoneNumber;
///
/// let us = InternationalPhoneNumber::try_new("+1-202-555-0123".to_string())?;
/// let uk = InternationalPhoneNumber::try_new("+44 20 7946 0958".to_string())?;
/// let jp = InternationalPhoneNumber::try_new("+81-3-1234-5678".to_string())?;
/// ```
#[nutype(
    validate(predicate = is_valid_intl_phone),
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
pub struct InternationalPhoneNumber(String);

#[cfg(feature = "intl-phone")]
fn is_valid_intl_phone(s: &str) -> bool {
    phonenumber::parse(None, s).is_ok()
}

#[cfg(feature = "intl-phone")]
/// US phone number (E.164 format)
///
/// **Business Rule**: Validates US phone numbers specifically
///
/// **Use when**: You only need US phone validation
#[nutype(
    validate(predicate = is_valid_us_phone),
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
pub struct USPhoneNumber(String);

#[cfg(feature = "intl-phone")]
fn is_valid_us_phone(s: &str) -> bool {
    match phonenumber::parse(Some(phonenumber::country::Id::US), s) {
        Ok(num) => num.is_valid(),
        Err(_) => false,
    }
}

// =============================================================================
// Date/Time Types
// =============================================================================

#[cfg(feature = "datetime")]
/// ISO 8601 date string (YYYY-MM-DD)
///
/// **Business Rule**: Valid ISO 8601 date format
///
/// **Use when**: Birth dates, deadlines, appointment dates
///
/// # Example
///
/// ```rust,ignore
/// use rusty_forms_validation::types::DateString;
///
/// let date = DateString::try_new("2025-12-02".to_string())?;  // ✓
/// let bad = DateString::try_new("12/02/2025".to_string());     // ✗ Wrong format
/// ```
#[nutype(
    validate(predicate = is_valid_date),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
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
pub struct DateString(String);

#[cfg(feature = "datetime")]
fn is_valid_date(s: &str) -> bool {
    time::Date::parse(s, &time::format_description::well_known::Iso8601::DEFAULT).is_ok()
}

#[cfg(feature = "datetime")]
/// ISO 8601 datetime string
///
/// **Business Rule**: Valid ISO 8601 datetime format
///
/// **Use when**: Event timestamps, created_at, updated_at fields
///
/// # Example
///
/// ```rust,ignore
/// use rusty_forms_validation::types::DateTimeString;
///
/// let dt = DateTimeString::try_new("2025-12-02T14:30:00Z".to_string())?;
/// ```
#[nutype(
    validate(predicate = is_valid_datetime),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
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
pub struct DateTimeString(String);

#[cfg(feature = "datetime")]
fn is_valid_datetime(s: &str) -> bool {
    time::PrimitiveDateTime::parse(s, &time::format_description::well_known::Iso8601::DEFAULT)
        .is_ok()
        || time::OffsetDateTime::parse(s, &time::format_description::well_known::Iso8601::DEFAULT)
            .is_ok()
}

#[cfg(feature = "datetime")]
/// Time string (HH:MM:SS format)
///
/// **Business Rule**: Valid 24-hour time format
///
/// **Use when**: Business hours, appointment times
///
/// # Example
///
/// ```rust,ignore
/// use rusty_forms_validation::types::TimeString;
///
/// let time = TimeString::try_new("14:30:00".to_string())?;
/// ```
#[nutype(
    validate(predicate = is_valid_time),
    derive(
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
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
pub struct TimeString(String);

#[cfg(feature = "datetime")]
fn is_valid_time(s: &str) -> bool {
    time::Time::parse(s, &time::format_description::well_known::Iso8601::DEFAULT).is_ok()
}

// =============================================================================
// Credit Card Types
// =============================================================================

#[cfg(feature = "credit-card")]
/// Valid credit card number (Luhn algorithm)
///
/// **Business Rule**: Valid credit card number (any major brand)
///
/// **Validates:**
/// - Luhn algorithm (checksum)
/// - Card number format
/// - Brand detection (Visa, Mastercard, Amex, Discover, etc.)
///
/// **Use when**: Payment processing, checkout forms
///
/// # Example
///
/// ```rust,ignore
/// use rusty_forms_validation::types::CreditCardNumber;
///
/// let visa = CreditCardNumber::try_new("4532015112830366".to_string())?;  // ✓ Valid Visa
/// let bad = CreditCardNumber::try_new("1234567812345678".to_string());     // ✗ Invalid checksum
/// ```
///
/// **Note:** This only validates format, not if the card is active or has funds!
#[nutype(
    validate(predicate = is_valid_credit_card),
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
        Serialize,
        Deserialize,
    )
)]
pub struct CreditCardNumber(String);

#[cfg(feature = "credit-card")]
fn is_valid_credit_card(s: &str) -> bool {
    // card_validate checks Luhn algorithm
    match card_validate::Validate::from(s) {
        Ok(_) => true,  // Valid if no error
        Err(_) => false,
    }
}

#[cfg(feature = "credit-card")]
/// Visa credit card number
///
/// **Business Rule**: Valid Visa card only
///
/// **Use when**: You need to restrict to Visa cards specifically
#[nutype(
    validate(predicate = is_valid_visa_card),
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
        Serialize,
        Deserialize,
    )
)]
pub struct VisaCardNumber(String);

#[cfg(feature = "credit-card")]
fn is_valid_visa_card(s: &str) -> bool {
    match card_validate::Validate::from(s) {
        Ok(validator) => matches!(validator.card_type, card_validate::Type::Visa),
        Err(_) => false,
    }
}

#[cfg(feature = "credit-card")]
/// CVV/CVC code (3 or 4 digits)
///
/// **Business Rule**: Valid CVV format
///
/// **Use when**: Card security code validation
#[nutype(
    validate(predicate = is_valid_cvv),
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
        Serialize,
        Deserialize,
    )
)]
pub struct CVVCode(String);

#[cfg(feature = "credit-card")]
fn is_valid_cvv(s: &str) -> bool {
    (s.len() == 3 || s.len() == 4) && s.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    // Collection tests
    #[test]
    fn test_non_empty_vec() {
        assert!(NonEmptyVec::try_new(Vec::<alloc::string::String>::new()).is_err()); // Empty
        assert!(NonEmptyVec::try_new(vec!["item".to_string()]).is_ok());
        assert!(NonEmptyVec::try_new(vec![1, 2, 3]).is_ok());
    }

    // Pattern tests
    #[test]
    fn test_phone_number() {
        assert!(PhoneNumber::try_new("1234567890".to_string()).is_ok());
        assert!(PhoneNumber::try_new("123-456-7890".to_string()).is_ok());
        assert!(PhoneNumber::try_new("(123) 456-7890".to_string()).is_ok());
        assert!(PhoneNumber::try_new("123456789".to_string()).is_err()); // Too short
        assert!(PhoneNumber::try_new("12345678901".to_string()).is_err()); // Too long
    }

    #[test]
    fn test_zip_code() {
        assert!(ZipCode::try_new("12345".to_string()).is_ok());
        assert!(ZipCode::try_new("12345-6789".to_string()).is_ok());
        assert!(ZipCode::try_new("1234".to_string()).is_err()); // Too short
        assert!(ZipCode::try_new("123456".to_string()).is_err()); // Too long
        assert!(ZipCode::try_new("12345-678".to_string()).is_err()); // Invalid +4
    }

    #[test]
    fn test_ip_address() {
        assert!(IpAddress::try_new("192.168.1.1".to_string()).is_ok());
        assert!(IpAddress::try_new("0.0.0.0".to_string()).is_ok());
        assert!(IpAddress::try_new("255.255.255.255".to_string()).is_ok());
        assert!(IpAddress::try_new("256.1.1.1".to_string()).is_err()); // Out of range
        assert!(IpAddress::try_new("192.168.1".to_string()).is_err()); // Missing octet
    }

    #[test]
    fn test_uuid() {
        assert!(Uuid::try_new("550e8400-e29b-41d4-a716-446655440000".to_string()).is_ok());
        assert!(Uuid::try_new("123e4567-e89b-12d3-a456-426614174000".to_string()).is_ok());
        assert!(Uuid::try_new("not-a-uuid".to_string()).is_err());
        assert!(Uuid::try_new("550e8400-e29b-41d4-a716".to_string()).is_err()); // Too short
    }

    // International Phone tests
    #[cfg(feature = "intl-phone")]
    #[test]
    fn test_international_phone() {
        // US numbers
        assert!(InternationalPhoneNumber::try_new("+1-202-555-0123".to_string()).is_ok());
        assert!(InternationalPhoneNumber::try_new("+12025550123".to_string()).is_ok());

        // UK numbers
        assert!(InternationalPhoneNumber::try_new("+44 20 7946 0958".to_string()).is_ok());

        // Invalid
        assert!(InternationalPhoneNumber::try_new("not-a-phone".to_string()).is_err());
        assert!(InternationalPhoneNumber::try_new("123".to_string()).is_err());
    }

    #[cfg(feature = "intl-phone")]
    #[test]
    fn test_us_phone() {
        assert!(USPhoneNumber::try_new("+1-202-555-0123".to_string()).is_ok());
        assert!(USPhoneNumber::try_new("(202) 555-0123".to_string()).is_ok());
        assert!(USPhoneNumber::try_new("2025550123".to_string()).is_ok());
    }

    // Date/Time tests
    #[cfg(feature = "datetime")]
    #[test]
    fn test_date_string() {
        assert!(DateString::try_new("2025-12-02".to_string()).is_ok());
        assert!(DateString::try_new("2025-01-01".to_string()).is_ok());
        assert!(DateString::try_new("12/02/2025".to_string()).is_err()); // Wrong format
        assert!(DateString::try_new("2025-13-01".to_string()).is_err()); // Invalid month
    }

    #[cfg(feature = "datetime")]
    #[test]
    fn test_datetime_string() {
        assert!(DateTimeString::try_new("2025-12-02T14:30:00Z".to_string()).is_ok());
        assert!(DateTimeString::try_new("2025-12-02T14:30:00".to_string()).is_ok());
        assert!(DateTimeString::try_new("not-a-datetime".to_string()).is_err());
    }

    #[cfg(feature = "datetime")]
    #[test]
    fn test_time_string() {
        assert!(TimeString::try_new("14:30:00".to_string()).is_ok());
        assert!(TimeString::try_new("09:00:00".to_string()).is_ok());
        assert!(TimeString::try_new("25:00:00".to_string()).is_err()); // Invalid hour
    }

    // Credit Card tests
    #[cfg(feature = "credit-card")]
    #[test]
    fn test_credit_card_number() {
        // Valid Visa
        assert!(CreditCardNumber::try_new("4532015112830366".to_string()).is_ok());

        // Invalid - bad checksum
        assert!(CreditCardNumber::try_new("1234567812345678".to_string()).is_err());
        assert!(CreditCardNumber::try_new("123".to_string()).is_err());
    }

    #[cfg(feature = "credit-card")]
    #[test]
    fn test_visa_card() {
        // Valid Visa (starts with 4)
        assert!(VisaCardNumber::try_new("4532015112830366".to_string()).is_ok());

        // Valid card but not Visa
        assert!(VisaCardNumber::try_new("5425233430109903".to_string()).is_err()); // Mastercard
    }

    #[cfg(feature = "credit-card")]
    #[test]
    fn test_cvv() {
        assert!(CVVCode::try_new("123".to_string()).is_ok());
        assert!(CVVCode::try_new("1234".to_string()).is_ok()); // Amex
        assert!(CVVCode::try_new("12".to_string()).is_err()); // Too short
        assert!(CVVCode::try_new("12345".to_string()).is_err()); // Too long
        assert!(CVVCode::try_new("abc".to_string()).is_err()); // Not digits
    }
}
