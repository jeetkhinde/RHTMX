//! Numeric validated types using nutype

use nutype::nutype;

/// Positive integer (> 0)
#[nutype(
    validate(greater = 0),
    derive(
        Debug,
        Clone,
        Copy,
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
pub struct PositiveInt(i64);

/// Non-negative integer (>= 0)
#[nutype(
    validate(greater_or_equal = 0),
    derive(
        Debug,
        Clone,
        Copy,
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
pub struct NonNegativeInt(i64);

/// Age (18-120 years)
///
/// **Business Rule**: Standard adult age range
///
/// **Use when**: Age verification, user registration
#[nutype(
    validate(greater_or_equal = 18, less_or_equal = 120),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Display,
        Serialize,
        Deserialize,
    )
)]
pub struct Age(i64);

/// Percentage (0-100)
///
/// **Business Rule**: Standard percentage value
///
/// **Use when**: Progress, discounts, ratings
#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 100),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Display,
        Serialize,
        Deserialize,
    )
)]
pub struct Percentage(i64);

/// Network port (1-65535)
///
/// **Business Rule**: Valid TCP/UDP port range
///
/// **Use when**: Network configuration
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 65535),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        AsRef,
        TryFrom,
        Into,
        Deref,
        Display,
        Serialize,
        Deserialize,
    )
)]
pub struct Port(i64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_int() {
        assert!(PositiveInt::try_from(0).is_err());
        assert!(PositiveInt::try_from(1).is_ok());
        assert!(PositiveInt::try_from(-1).is_err());
    }

    #[test]
    fn test_age() {
        assert!(Age::try_from(17).is_err()); // Too young
        assert!(Age::try_from(18).is_ok());
        assert!(Age::try_from(65).is_ok());
        assert!(Age::try_from(120).is_ok());
        assert!(Age::try_from(121).is_err()); // Too old
    }

    #[test]
    fn test_percentage() {
        assert!(Percentage::try_from(-1).is_err());
        assert!(Percentage::try_from(0).is_ok());
        assert!(Percentage::try_from(50).is_ok());
        assert!(Percentage::try_from(100).is_ok());
        assert!(Percentage::try_from(101).is_err());
    }

    #[test]
    fn test_port() {
        assert!(Port::try_from(0).is_err()); // Port 0 invalid
        assert!(Port::try_from(1).is_ok());
        assert!(Port::try_from(80).is_ok());
        assert!(Port::try_from(443).is_ok());
        assert!(Port::try_from(65535).is_ok());
        assert!(Port::try_from(65536).is_err()); // Out of range
    }
}
