# RHTMX Router

A high-performance, zero-dependency file-system-based routing library for Rust with functional programming optimizations.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

## Features

✨ **Zero Dependencies** - Only uses Rust standard library  
🚀 **High Performance** - 115ns lookups with zero-copy optimization  
🎯 **Functional Programming** - Cow, lazy iterators, and functional composition  
📁 **File-System Based** - Intuitive directory structure mapping  
🔀 **Flexible Routing** - Static, dynamic, optional, and catch-all routes  
🎨 **Nested Layouts** - Automatic layout inheritance through directory hierarchy  
❌ **Error Pages** - Scoped error handling per section  
🛡️ **Robust** - Handles malformed paths gracefully (trailing slashes, backslashes, Windows paths)  
📝 **Well Documented** - Complete rustdoc with examples  
✅ **Tested** - 30 comprehensive tests covering all features  

---

## Quick Start

```rust
use rhtmx_router::{Router, Route};

// Create router
let mut router = Router::new();

// Add routes
router.add_route(Route::from_path("pages/index.rhtml", "pages"));
router.add_route(Route::from_path("pages/about.rhtml", "pages"));
router.add_route(Route::from_path("pages/users/[id].rhtml", "pages"));

// Match routes
let route_match = router.match_route("/users/123").unwrap();
assert_eq!(route_match.params.get("id"), Some(&"123".to_string()));
```

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rhtmx-router = "0.1.0"
```

---

## Route Types

### Static Routes

```
pages/about.rhtml       → /about
pages/contact.rhtml     → /contact
```

### Dynamic Parameters

```
pages/users/[id].rhtml              → /users/:id
pages/posts/[year]/[slug].rhtml     → /posts/:year/:slug
```

### Optional Parameters

```
pages/posts/[id?].rhtml             → /posts/:id?

Matches:
  /posts/123  → id = "123"
  /posts      → id = None
```

### Catch-All Routes

```
pages/docs/[...slug].rhtml          → /docs/*slug

Matches:
  /docs/guide/intro  → slug = "guide/intro"
  /docs/api         → slug = "api"
  /docs             → slug = ""
```

### Index Routes

```
pages/index.rhtml           → /
pages/users/index.rhtml     → /users
```

---

## Layouts

Layouts are automatically inherited through the directory hierarchy.

### File Structure

```
pages/
  ├── _layout.rhtml              # Root layout
  ├── index.rhtml                # Uses root layout
  ├── dashboard/
  │   ├── _layout.rhtml          # Dashboard layout
  │   ├── index.rhtml            # Uses dashboard layout
  │   └── settings.rhtml         # Uses dashboard layout
  └── api/
      ├── _error.rhtml           # API error page
      └── users.rhtml            # Uses root layout (no API layout exists)
```

### Layout Resolution

```rust
router.get_layout("/dashboard/settings")
// Checks in order:
// 1. /dashboard/settings  → Not found
// 2. /dashboard           → FOUND! Returns dashboard layout
```

For deep paths:
```rust
router.get_layout("/dashboard/admin/users/edit")
// Checks: /dashboard/admin/users/edit → /dashboard/admin/users 
//         → /dashboard/admin → /dashboard → /
```

---

## Error Pages

Error pages work identically to layouts:

```
pages/
  ├── _error.rhtml           # Root error page
  └── api/
      ├── _error.rhtml       # API-specific error page
      └── users.rhtml
```

```rust
router.get_error_page("/api/users")
// Returns: /api error page

router.get_error_page("/other")
// Returns: / root error page
```

---

## Path Normalization

The router automatically handles malformed paths:

```rust
// All of these work correctly:
router.get_layout("/dashboard/settings")     // ✅ Valid
router.get_layout("/dashboard/settings/")    // ✅ Trailing slash
router.get_layout("/dashboard//settings")    // ✅ Double slash
router.get_layout("/dashboard\\settings")    // ✅ Backslash
router.get_layout("\\dashboard\\settings")   // ✅ Windows path
```

**Performance:**
- Valid paths: ~115ns (zero allocations)
- Invalid paths: ~310ns (single allocation)

---

## Priority System

Routes are automatically sorted by priority (lower = higher priority):

| Type | Example | Priority | Formula |
|------|---------|----------|---------|
| Static | `/about` | 0 | 0 |
| Optional | `/posts/:id?` | 2 | params + depth |
| Dynamic | `/users/:id` | 4 | params + depth + 1 |
| Catch-all | `/docs/*slug` | 1001 | 1000 + depth |

### Matching Order

```rust
router.add_route(Route::from_path("pages/users/new.rhtml", "pages"));
router.add_route(Route::from_path("pages/users/[id].rhtml", "pages"));

// /users/new → Matches static route (priority 0)
// /users/123 → Matches dynamic route (priority 4)
```

Static routes always match before dynamic routes at the same path depth.

---

## Case-Insensitive Matching

```rust
let router = Router::with_case_insensitive(true);
router.add_route(Route::from_path("pages/about.rhtml", "pages"));

// All match:
router.match_route("/about");   // ✅
router.match_route("/ABOUT");   // ✅
router.match_route("/About");   // ✅
```

---

## Functional Programming Approach

### Three Core Techniques

#### 1. Zero-Copy Optimization (Cow)

```rust
fn normalize_path(path: &str) -> Cow<'_, str> {
    if is_valid_path(path) {
        return Cow::Borrowed(path);  // No allocation!
    }
    Cow::Owned(fix_path(path))  // Allocate only if needed
}
```

#### 2. Lazy Evaluation (Iterator)

```rust
struct PathHierarchy<'a> {
    current: Option<&'a str>,
}

// Yields: "/a/b/c" → "/a/b" → "/a" → "/"
// Stops on first match (short-circuit)
```

#### 3. Functional Composition

```rust
pub fn get_layout(&self, pattern: &str) -> Option<&Route> {
    let normalized = normalize_path(pattern);
    PathHierarchy::new(&normalized)
        .find_map(|path| self.layouts.get(path))
}
```

**3 lines vs 17 lines imperative!**

---

## Performance

### Benchmarks

| Operation | Time | Allocations |
|-----------|------|-------------|
| Valid path lookup | 115ns | 0 |
| Invalid path lookup | 310ns | 1 |
| Windows path lookup | 360ns | 1 |
| Route matching | ~100ns | 1 (Route clone) |

### Comparison with Other Approaches

| Approach | Valid Path | Invalid Path | Memory |
|----------|-----------|--------------|--------|
| **Functional** ✅ | **115ns** | **310ns** | **16B** |
| Imperative | 250ns | 250ns | 70B |
| Vec Split | 650ns | 650ns | 198B |

**2.2x faster for common case!**

---

## API Reference

### Route

```rust
// Create from file path
let route = Route::from_path("pages/users/[id].rhtml", "pages");

// Match against path
let params = route.matches("/users/123");

// Get parent pattern
let parent = route.layout_pattern();  // Some("/users")
```

### Router

```rust
// Create router
let mut router = Router::new();
let mut router = Router::with_case_insensitive(true);

// Add/remove routes
router.add_route(route);
router.remove_route("/about");

// Match routes
let route_match = router.match_route("/users/123");

// Get layouts/error pages
let layout = router.get_layout("/dashboard/settings");
let error_page = router.get_error_page("/api/users");

// Access collections
let routes = router.routes();
let layouts = router.layouts();
let error_pages = router.error_pages();
```

---

## Examples

### Basic Routing

```rust
use rhtmx_router::{Router, Route};

let mut router = Router::new();

router.add_route(Route::from_path("pages/index.rhtml", "pages"));
router.add_route(Route::from_path("pages/about.rhtml", "pages"));
router.add_route(Route::from_path("pages/users/[id].rhtml", "pages"));
router.add_route(Route::from_path("pages/docs/[...slug].rhtml", "pages"));

// Match routes
let m = router.match_route("/").unwrap();
assert_eq!(m.route.pattern, "/");

let m = router.match_route("/users/123").unwrap();
assert_eq!(m.params.get("id"), Some(&"123".to_string()));

let m = router.match_route("/docs/api/reference").unwrap();
assert_eq!(m.params.get("slug"), Some(&"api/reference".to_string()));
```

### Nested Layouts

```rust
let mut router = Router::new();

router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));
router.add_route(Route::from_path("pages/dashboard/admin/_layout.rhtml", "pages"));

// Get layout for deep path
let layout = router.get_layout("/dashboard/admin/settings").unwrap();
assert_eq!(layout.pattern, "/dashboard/admin");

// Skips missing intermediate levels
let layout = router.get_layout("/dashboard/admin/users/edit").unwrap();
assert_eq!(layout.pattern, "/dashboard/admin");  // No /dashboard/admin/users layout
```

### Error Pages

```rust
let mut router = Router::new();

router.add_route(Route::from_path("pages/_error.rhtml", "pages"));
router.add_route(Route::from_path("pages/api/_error.rhtml", "pages"));

let error = router.get_error_page("/api/users").unwrap();
assert_eq!(error.pattern, "/api");

let error = router.get_error_page("/other").unwrap();
assert_eq!(error.pattern, "/");
```

### Malformed Path Handling

```rust
let mut router = Router::new();
router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));

// All work correctly:
assert!(router.get_layout("/dashboard/settings").is_some());
assert!(router.get_layout("/dashboard/settings/").is_some());    // Trailing slash
assert!(router.get_layout("/dashboard//settings").is_some());    // Double slash
assert!(router.get_layout("/dashboard\\settings").is_some());    // Backslash
assert!(router.get_layout("\\dashboard\\settings").is_some());   // Windows path
```

---

## Testing

Run tests:

```bash
cargo test
```

Run with output:

```bash
cargo test -- --nocapture
```

---

## Architecture

### File Structure

```
src/
  └── lib.rs                    # Main library (1074 lines)
      ├── Core Types
      │   ├── Route             # Individual route definition
      │   └── RouteMatch        # Matching result with params
      ├── Path Utilities
      │   ├── normalize_path()  # Zero-copy normalization
      │   ├── is_valid_path()   # Validation helper
      │   └── PathHierarchy     # Lazy iterator
      ├── Route Implementation
      │   ├── from_path()       # Create from file
      │   ├── matches()         # Pattern matching
      │   └── layout_pattern()  # Parent lookup
      ├── Router Implementation
      │   ├── add_route()       # Auto-sorting insertion
      │   ├── match_route()     # Find matching route
      │   ├── get_layout()      # Layout resolution
      │   └── get_error_page()  # Error page resolution
      └── Tests (30)            # Comprehensive coverage
```

### Design Principles

1. **Zero Dependencies** - Only std library
2. **Functional First** - Cow, iterators, composition
3. **Performance** - Zero-copy, lazy evaluation
4. **Robustness** - Handle all edge cases
5. **Simplicity** - Clean API, intuitive patterns

---

## Builder Methods for Layout Control

Configure layouts with functional builder methods:

```rust
use rhtmx_router::{Route, LayoutOption};

// Skip all layouts
let route = Route::from_path("pages/api/data.rhtml", "pages")
    .with_no_layout();

// Use root layout only
let route = Route::from_path("pages/dashboard/print.rhtml", "pages")
    .with_root_layout();

// Use specific named layout
let route = Route::from_path("pages/vendors/settings.rhtml", "pages")
    .with_named_layout("vendor");

// Use layout at specific pattern
let route = Route::from_path("pages/api/v2/users.rhtml", "pages")
    .with_layout_pattern("/api");

// Or use LayoutOption enum directly
let route = Route::from_path("pages/print.rhtml", "pages")
    .with_layout_option(LayoutOption::Root);
```

See [ADVANCED_LAYOUTS_GUIDE.md](ADVANCED_LAYOUTS_GUIDE.md) for comprehensive patterns.

---

## Advanced Layout Patterns

For complex layout scenarios, see the **[Advanced Layouts Guide](ADVANCED_LAYOUTS_GUIDE.md)**:

- 🚫 The `_nolayout` convention for blocking layout inheritance
- 📋 Layout hierarchy skipping (skip parent, use grandparent)
- 🏷️ Named layouts for multiple layout options
- 🔀 Intercepting routes for modals and overlays
- 🛠️ Integration with other web frameworks
- 🎯 Dynamic sidebar loading patterns

---

## Known Limitations

See [CRITICAL_MISSING_FEATURES.md](CRITICAL_MISSING_FEATURES.md) for details.

**Resolved in v0.1.0+:**
- ✅ Way to skip parent layouts (via `LayoutOption::Root`)
- ✅ Explicit "no layout" option (via `with_no_layout()`)
- ✅ Layout composition control (via builder methods)

**Minor:**
- No middleware/guards
- No regex patterns
- No named routes
- O(n) route matching (consider trie for 1000+ routes)

---

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new features
4. Ensure all tests pass
5. Submit a pull request

---

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

## Changelog

### v0.1.0 (Current)

**Features:**
- ✅ File-system based routing
- ✅ Static, dynamic, optional, catch-all routes
- ✅ Nested layouts with inheritance
- ✅ Scoped error pages
- ✅ Path normalization (7 edge cases)
- ✅ Functional programming optimizations
- ✅ Case-insensitive matching
- ✅ Zero dependencies

**Performance:**
- ✅ 115ns lookups (zero-copy)
- ✅ 2.2x faster than imperative approach
- ✅ 4.4x less memory usage

**Quality:**
- ✅ 30 comprehensive tests
- ✅ 100% documentation coverage
- ✅ Zero code duplication

---

## Resources & Documentation

### Getting Started

- **[README.md](README.md)** - You are here! Basic routing guide
- **[LAYOUT_QUICK_REFERENCE.md](LAYOUT_QUICK_REFERENCE.md)** - One-page cheat sheet for layouts ⭐ **START HERE**

### Layout Configuration

- **[ADVANCED_LAYOUTS_GUIDE.md](ADVANCED_LAYOUTS_GUIDE.md)** - Comprehensive layout patterns (350+ lines)
  - _nolayout convention
  - Layout hierarchy skipping
  - Named layouts
  - Intercepting routes
  - Framework integration

### Framework Integration

- **[SLOTS_FRAMEWORK_INTEGRATION.md](../rhtmx/docs/SLOTS_FRAMEWORK_INTEGRATION.md)** - Framework examples (400+ lines)
  - Axum, Rocket, Actix-Web, Warp, Tonic
  - Dynamic sidebar loading patterns
  - Real-world complete examples

### Architecture & Performance

- [Improvements Summary](IMPROVEMENTS_SUMMARY.md) - Full changelog
- [Functional Programming Guide](FUNCTIONAL_QUICK_REFERENCE.md) - Techniques used
- [Approach Comparison](FUNCTIONAL_APPROACH_COMPARISON.md) - Benchmarks
- [Missing Features](CRITICAL_MISSING_FEATURES.md) - Known limitations

---

## Credits

Created with functional programming principles and zero-dependency philosophy.

Inspired by file-system routing from Next.js, SvelteKit, and other modern frameworks.
