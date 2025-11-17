/// Defines how a route should resolve its layout
///
/// Uses functional programming principles:
/// - Immutable values
/// - Composable options
/// - Pattern matching for resolution
///
/// # Examples
///
/// ```
/// use rhtmx_router::LayoutOption;
///
/// // Default: inherit from parent
/// let inherit = LayoutOption::default();
/// assert_eq!(inherit, LayoutOption::Inherit);
///
/// // No layout (e.g., modals, standalone pages)
/// let none = LayoutOption::None;
///
/// // Use specific named layout
/// let admin = LayoutOption::Named("admin".to_string());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutOption {
    /// Inherit from nearest parent layout (default behavior)
    Inherit,
    /// No layout - render route standalone
    None,
    /// Use root layout only, skip all intermediate layouts
    Root,
    /// Use a specific named layout (e.g., "admin", "marketing")
    Named(String),
    /// Use layout at a specific pattern (e.g., "/dashboard")
    Pattern(String),
}

impl Default for LayoutOption {
    fn default() -> Self {
        Self::Inherit
    }
}
