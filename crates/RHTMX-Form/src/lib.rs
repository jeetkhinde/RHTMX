// RHTMX Procedural Macros for Form

use proc_macro::TokenStream;

use syn::{parse_macro_input, DeriveInput};

mod validation;

/// Derive macro for automatic validation
///
/// Generates a `Validate` trait implementation that validates struct fields
/// based on attributes like #[email], #[min], #[max], etc.
///
/// # Example
///
/// ```ignore
/// use rhtmx::Validate;
/// use serde::Deserialize;
///
/// #[derive(Validate, Deserialize)]
/// struct CreateUserRequest {
///     #[min_length(3)]
///     #[max_length(50)]
///     name: String,
///
///     #[email]
///     #[no_public_domains]
///     email: String,
///
///     #[password("strong")]
///     password: String,
///
///     #[min(18)]
///     #[max(120)]
///     age: i32,
///
///     bio: Option<String>,  // Optional fields
/// }
/// ```
///
/// # Available Validators
///
/// **Email Validators:**
/// - `#[email]` - Valid email format
/// - `#[no_public_domains]` - Reject gmail, yahoo, etc.
/// - `#[blocked_domains("a.com", "b.com")]` - Block specific domains
///
/// **Password Validators:**
/// - `#[password("strong")]` - 8+ chars, upper, lower, digit, special
/// - `#[password("medium")]` - 8+ chars, upper, lower, digit
/// - `#[password("basic")]` - 6+ chars
/// - `#[password(r"regex")]` - Custom regex pattern
///
/// **Numeric Validators:**
/// - `#[min(n)]` - Minimum value
/// - `#[max(n)]` - Maximum value
/// - `#[range(min, max)]` - Value range
///
/// **String Validators:**
/// - `#[min_length(n)]` - Minimum length
/// - `#[max_length(n)]` - Maximum length
/// - `#[length(min, max)]` - Length range
/// - `#[regex(r"pattern")]` - Custom regex
/// - `#[url]` - Valid URL format
///
/// **String Matching:**
/// - `#[contains("text")]` - String must contain substring
/// - `#[not_contains("text")]` - String must not contain substring
/// - `#[starts_with("prefix")]` - String must start with prefix
/// - `#[ends_with("suffix")]` - String must end with suffix
///
/// **Equality:**
/// - `#[equals("value")]` - Must equal exact value
/// - `#[not_equals("value")]` - Must not equal value
/// - `#[equals_field("other_field")]` - Must match another field
///
/// **Conditional:**
/// - `#[depends_on("field", "value")]` - Required when another field has specific value
///
/// **Collections:**
/// - `#[min_items(n)]` - Minimum number of items in Vec/HashSet
/// - `#[max_items(n)]` - Maximum number of items
/// - `#[unique]` - All items must be unique
///
/// **Enum/Values:**
/// - `#[enum_variant("val1", "val2")]` - Must be one of allowed values
///
/// **Custom:**
/// - `#[custom("func_name")]` - Call custom validation function
/// - `#[message = "text"]` - Override default error message
/// - `#[label("Name")]` - Use friendly name in errors
/// - `#[message_key("key")]` - i18n message key
///
/// **General:**
/// - `#[required]` - Required for Option<T> fields
/// - `#[allow_whitespace]` - Don't trim whitespace
///
#[proc_macro_derive(
    Validate,
    attributes(
        email,
        no_public_domains,
        blocked_domains,
        password,
        min,
        max,
        range,
        min_length,
        max_length,
        length,
        regex,
        url,
        allow_whitespace,
        required,
        contains,
        not_contains,
        starts_with,
        ends_with,
        equals,
        not_equals,
        equals_field,
        depends_on,
        min_items,
        max_items,
        unique,
        enum_variant,
        message,
        label,
        message_key,
        custom,
        query,
        form,
        path
    )
)]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    validation::impl_validate(&input).into()
}
