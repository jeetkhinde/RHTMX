/// Parameter constraint for validating dynamic route parameters
///
/// Uses functional pattern matching for validation logic.
/// Constraints ensure type safety and input validation at routing level.
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterConstraint {
    /// No constraint - accepts any value (default)
    Any,
    /// Integer numbers only: 123, -456
    Int,
    /// Unsigned integer: 123, 456 (no negatives)
    UInt,
    /// Alphabetic characters only: abc, XYZ
    Alpha,
    /// Alphanumeric: abc123, Test99
    AlphaNum,
    /// Slug format: hello-world, my_post
    Slug,
    /// UUID format: 550e8400-e29b-41d4-a716-446655440000
    Uuid,
    /// Custom regex pattern
    Regex(String),
}

impl ParameterConstraint {
    /// Validates a value against this constraint (functional predicate)
    ///
    /// Pure function that maps (constraint, value) → bool
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::ParameterConstraint;
    ///
    /// assert!(ParameterConstraint::Int.validate("123"));
    /// assert!(!ParameterConstraint::Int.validate("abc"));
    ///
    /// assert!(ParameterConstraint::Alpha.validate("hello"));
    /// assert!(!ParameterConstraint::Alpha.validate("hello123"));
    /// ```
    pub fn validate(&self, value: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Int => value.parse::<i64>().is_ok(),
            Self::UInt => value.parse::<u64>().is_ok(),
            Self::Alpha => value.chars().all(|c| c.is_alphabetic()),
            Self::AlphaNum => value.chars().all(|c| c.is_alphanumeric()),
            Self::Slug => value
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_'),
            Self::Uuid => {
                // Simple UUID validation: 8-4-4-4-12 hex digits
                // Pure functional approach: split → map → fold
                let parts: Vec<&str> = value.split('-').collect();
                parts.len() == 5
                    && parts[0].len() == 8
                    && parts[1].len() == 4
                    && parts[2].len() == 4
                    && parts[3].len() == 4
                    && parts[4].len() == 12
                    && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_hexdigit()))
            }
            Self::Regex(pattern) => {
                // For zero-dependency, use simple pattern matching
                // In real use, would use regex crate
                // For now, just check if pattern is in value
                value.contains(pattern)
            }
        }
    }

    /// Parses constraint from string (functional parser)
    ///
    /// Maps string → ParameterConstraint using pattern matching
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::ParameterConstraint;
    ///
    /// assert_eq!(ParameterConstraint::from_str("int"), ParameterConstraint::Int);
    /// assert_eq!(ParameterConstraint::from_str("alpha"), ParameterConstraint::Alpha);
    /// assert_eq!(ParameterConstraint::from_str("uuid"), ParameterConstraint::Uuid);
    /// ```
    ///
    /// Supported values: "int", "uint", "alpha", "alphanum", "slug", "uuid", "regex:pattern"
    pub fn from_str(s: &str) -> Self {
        match s {
            "int" | "integer" => Self::Int,
            "uint" | "unsigned" => Self::UInt,
            "alpha" => Self::Alpha,
            "alphanum" | "alphanumeric" => Self::AlphaNum,
            "slug" => Self::Slug,
            "uuid" => Self::Uuid,
            _ if s.starts_with("regex:") => {
                Self::Regex(s.strip_prefix("regex:").unwrap_or("").to_string())
            }
            _ => Self::Any,
        }
    }
}
