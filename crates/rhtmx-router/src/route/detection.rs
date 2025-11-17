/// Route detection functions for special file patterns
///
/// Pure functional detection of:
/// - Named layouts (_layout.name)
/// - Parallel routes (@slot_name)
/// - Intercepting routes ((.), (..), (...), (....))
///
/// All functions are **pure**: same input → same output, no side effects.

use crate::InterceptLevel;

/// Extracts layout name from filename (pure function)
///
/// **Pure function**: Maps filename → Option<layout_name>
///
/// # Examples
///
/// ```
/// use rhtmx_router::route::detection::extract_layout_name;
///
/// assert_eq!(extract_layout_name("_layout.admin"), Some("admin".to_string()));
/// assert_eq!(extract_layout_name("_layout.marketing"), Some("marketing".to_string()));
/// assert_eq!(extract_layout_name("_layout"), None);
/// assert_eq!(extract_layout_name("page"), None);
/// ```
///
/// # Performance
///
/// - O(n) where n is filename length
/// - Zero allocations if no match (returns None)
/// - Single allocation if matched (layout name string)
pub fn extract_layout_name(filename: &str) -> Option<String> {
    // Functional approach: strip_prefix → map → Option<String>
    filename
        .strip_prefix("_layout.")
        .map(|name| name.to_string())
}

/// Detects parallel route slot from path (pure function, Phase 5.1)
///
/// **Pure function**: Maps path → (is_parallel, Option<slot_name>)
///
/// Parallel routes render multiple pages simultaneously in the same layout.
/// Pattern: `@slot_name` segments in path.
///
/// # Examples
///
/// ```
/// use rhtmx_router::route::detection::detect_parallel_route;
///
/// // Parallel route with @analytics slot
/// let (is_parallel, slot) = detect_parallel_route("dashboard/@analytics/page");
/// assert_eq!(is_parallel, true);
/// assert_eq!(slot, Some("analytics".to_string()));
///
/// // Regular route
/// let (is_parallel, slot) = detect_parallel_route("dashboard/users");
/// assert_eq!(is_parallel, false);
/// assert_eq!(slot, None);
///
/// // Multiple @ - only first matters
/// let (is_parallel, slot) = detect_parallel_route("@team/settings/@nested");
/// assert_eq!(is_parallel, true);
/// assert_eq!(slot, Some("team".to_string()));
/// ```
///
/// # Performance
///
/// - O(n) where n is path length
/// - Short-circuits on first @ segment found
/// - Functional iterator pipeline: split → find → map
pub fn detect_parallel_route(path: &str) -> (bool, Option<String>) {
    // Functional approach: split → find → map → unwrap_or
    path.split('/')
        .find(|seg| seg.starts_with('@') && seg.len() > 1)
        .map(|seg| {
            let slot_name = seg[1..].to_string();
            (true, Some(slot_name))
        })
        .unwrap_or((false, None))
}

/// Detects intercepting route level from path (pure function, Phase 5.2)
///
/// **Pure function**: Maps path → (is_intercepting, Option<level>, Option<target>)
///
/// Intercepting routes enable modal/overlay patterns by intercepting navigation.
/// Patterns:
/// - `(.)` - Intercept at same directory level
/// - `(..)` - Intercept one directory level up
/// - `(...)` - Intercept from root
/// - `(....)` - Intercept two directory levels up
///
/// # Examples
///
/// ```
/// use rhtmx_router::route::detection::detect_intercepting_route;
/// use rhtmx_router::InterceptLevel;
///
/// // Same level intercept
/// let (is_int, level, target) = detect_intercepting_route("feed/(.)/photo");
/// assert_eq!(is_int, true);
/// assert_eq!(level, Some(InterceptLevel::SameLevel));
/// assert_eq!(target, Some("photo".to_string()));
///
/// // One level up with dynamic param
/// let (is_int, level, target) = detect_intercepting_route("feed/(..)/photo/[id]");
/// assert_eq!(is_int, true);
/// assert_eq!(level, Some(InterceptLevel::OneLevelUp));
/// assert_eq!(target, Some("photo/[id]".to_string()));
///
/// // From root
/// let (is_int, level, target) = detect_intercepting_route("(...)/photo/[id]");
/// assert_eq!(is_int, true);
/// assert_eq!(level, Some(InterceptLevel::FromRoot));
/// assert_eq!(target, Some("photo/[id]".to_string()));
///
/// // Regular route (no interception)
/// let (is_int, level, target) = detect_intercepting_route("normal/path");
/// assert_eq!(is_int, false);
/// assert_eq!(level, None);
/// assert_eq!(target, None);
/// ```
///
/// # Performance
///
/// - O(n) where n is number of segments
/// - Short-circuits on first intercept marker found
/// - Functional iteration with early return
pub fn detect_intercepting_route(path: &str) -> (bool, Option<InterceptLevel>, Option<String>) {
    let segments: Vec<&str> = path.split('/').collect();

    for (idx, seg) in segments.iter().enumerate() {
        // Pattern match on intercept markers (functional approach)
        let level = match *seg {
            "(.)" => Some(InterceptLevel::SameLevel),
            "(..)" => Some(InterceptLevel::OneLevelUp),
            "(...)" => Some(InterceptLevel::FromRoot),
            "(....)" => Some(InterceptLevel::TwoLevelsUp),
            _ => None,
        };

        if let Some(intercept_level) = level {
            // Capture the remaining path after the intercept marker
            // Functional approach: slice → join
            let target = if idx + 1 < segments.len() {
                Some(segments[idx + 1..].join("/"))
            } else {
                None
            };
            return (true, Some(intercept_level), target);
        }
    }

    (false, None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_layout_name_with_name() {
        assert_eq!(
            extract_layout_name("_layout.admin"),
            Some("admin".to_string())
        );
        assert_eq!(
            extract_layout_name("_layout.marketing"),
            Some("marketing".to_string())
        );
    }

    #[test]
    fn test_extract_layout_name_without_name() {
        assert_eq!(extract_layout_name("_layout"), None);
        assert_eq!(extract_layout_name("page"), None);
        assert_eq!(extract_layout_name(""), None);
    }

    #[test]
    fn test_detect_parallel_route_with_slot() {
        let (is_parallel, slot) = detect_parallel_route("dashboard/@analytics/page");
        assert_eq!(is_parallel, true);
        assert_eq!(slot, Some("analytics".to_string()));
    }

    #[test]
    fn test_detect_parallel_route_without_slot() {
        let (is_parallel, slot) = detect_parallel_route("dashboard/users");
        assert_eq!(is_parallel, false);
        assert_eq!(slot, None);
    }

    #[test]
    fn test_detect_parallel_route_multiple_slots() {
        // Only first @ matters
        let (is_parallel, slot) = detect_parallel_route("@team/settings/@nested");
        assert_eq!(is_parallel, true);
        assert_eq!(slot, Some("team".to_string()));
    }

    #[test]
    fn test_detect_intercepting_route_same_level() {
        let (is_int, level, target) = detect_intercepting_route("feed/(.)/photo");
        assert_eq!(is_int, true);
        assert_eq!(level, Some(InterceptLevel::SameLevel));
        assert_eq!(target, Some("photo".to_string()));
    }

    #[test]
    fn test_detect_intercepting_route_one_level_up() {
        let (is_int, level, target) = detect_intercepting_route("feed/(..)/photo/[id]");
        assert_eq!(is_int, true);
        assert_eq!(level, Some(InterceptLevel::OneLevelUp));
        assert_eq!(target, Some("photo/[id]".to_string()));
    }

    #[test]
    fn test_detect_intercepting_route_from_root() {
        let (is_int, level, target) = detect_intercepting_route("(...)/photo/[id]");
        assert_eq!(is_int, true);
        assert_eq!(level, Some(InterceptLevel::FromRoot));
        assert_eq!(target, Some("photo/[id]".to_string()));
    }

    #[test]
    fn test_detect_intercepting_route_two_levels_up() {
        let (is_int, level, target) = detect_intercepting_route("feed/(....)/settings");
        assert_eq!(is_int, true);
        assert_eq!(level, Some(InterceptLevel::TwoLevelsUp));
        assert_eq!(target, Some("settings".to_string()));
    }

    #[test]
    fn test_detect_intercepting_route_no_intercept() {
        let (is_int, level, target) = detect_intercepting_route("normal/path");
        assert_eq!(is_int, false);
        assert_eq!(level, None);
        assert_eq!(target, None);
    }
}
