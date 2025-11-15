# Critical Missing Features

This document outlines features that are **not currently implemented** in the RHTMX router but may be needed for production applications.

---

## 🚨 CRITICAL: Layout Control Issues

### 1. No Way to Skip Parent Layouts

**Problem:**
Currently, layout resolution walks up the directory tree and uses the FIRST layout found. There's no way to skip intermediate layouts.

**Example:**
```
pages/
  ├── _layout.rhtml               # Root layout (simple header)
  ├── dashboard/
  │   ├── _layout.rhtml           # Dashboard layout (complex sidebar, heavy)
  │   └── admin/
  │       └── reports/
  │           └── print.rhtml     # Wants ROOT layout only, not dashboard
```

**Current Behavior:**
```rust
router.get_layout("/dashboard/admin/reports/print")
// Returns: /dashboard layout (WRONG - too heavy for print view)
```

**Desired Behavior:**
```rust
// Want to skip /dashboard and use / instead
// NO CURRENT WAY TO DO THIS
```

**Impact:** **HIGH** - Can't create lightweight pages under heavy sections

---

### 2. No Explicit "No Layout" Option

**Problem:**
Every route will inherit from the nearest parent or root layout. No way to disable layouts entirely.

**Use Cases:**
- Standalone pages (login, 404)
- Print views
- Embedded iframes
- API documentation pages
- Landing pages with custom structure

**Current Workaround:**
Create empty `_layout.rhtml` file - but this is unclear and still requires a file.

**Desired Solutions:**

**Option A: Special filename**
```
pages/dashboard/print/_nolayout.marker
pages/dashboard/print/report.rhtml  # No layout applied
```

**Option B: Frontmatter in route file**
```rhtml
---
layout: none
---
<h1>Standalone Page</h1>
```

**Option C: Programmatic API**
```rust
let route = Route::from_path("pages/print.rhtml", "pages")
    .with_layout(LayoutOption::None);
```

**Impact:** **HIGH** - Common requirement for real applications

---

### 3. No Layout Composition Control

**Problem:**
Can't specify which layout to use - always inherits from nearest parent. No mixing/matching.

**Use Cases:**
- Use root layout only (skip all intermediate)
- Use specific named layout
- Compose multiple layouts (header from root, footer from parent)
- Override layout priority

**Example Scenario:**
```
pages/
  ├── _layout.rhtml              # Simple layout
  ├── _layout.admin.rhtml        # Admin layout with auth
  ├── _layout.marketing.rhtml    # Marketing layout with tracking
  ├── dashboard/
  │   ├── _layout.rhtml          # Dashboard layout
  │   └── user-profile.rhtml     # Wants ADMIN layout, not dashboard
```

**Desired Syntax (Not Implemented):**
```rust
// Option A: Named layouts
router.get_layout_named("/dashboard/user-profile", "admin")

// Option B: Metadata
Route::from_path("pages/dashboard/user-profile.rhtml", "pages")
    .with_layout_name("admin")

// Option C: Frontmatter
---
layout: admin
---
```

**Impact:** **MEDIUM-HIGH** - Needed for flexible layout systems

---

## ⚠️ HIGH PRIORITY: Route Features

### 4. No Middleware/Guards

**Problem:**
No way to add authentication, rate limiting, logging, or other cross-cutting concerns.

**Use Cases:**
- Authentication checks
- Authorization (role-based access)
- Rate limiting
- Request logging
- Response transformation
- CORS handling

**Desired API:**
```rust
// Not implemented
router.add_middleware(AuthMiddleware::new());
router.add_guard("/admin/*", AdminGuard);

// Or per-route
Route::from_path("pages/admin/users.rhtml", "pages")
    .with_guard(RequireAuth)
    .with_middleware(LogRequest)
```

**Impact:** **HIGH** - Essential for secure applications

---

### 5. No Route Metadata

**Problem:**
Can't attach metadata to routes for titles, permissions, cache settings, etc.

**Use Cases:**
- Page titles for breadcrumbs
- Permission requirements
- Cache TTL settings
- SEO metadata
- API versioning info

**Desired API:**
```rust
// Not implemented
Route::from_path("pages/users/[id].rhtml", "pages")
    .with_metadata("title", "User Profile")
    .with_metadata("permission", "users.read")
    .with_metadata("cache_ttl", "300")
```

**Impact:** **MEDIUM-HIGH** - Needed for real applications

---

### 6. No Dynamic Parameter Constraints

**Problem:**
Dynamic parameters accept any string. No validation.

**Examples:**
```rust
// pages/users/[id].rhtml
// Currently matches:
//   /users/123     ✅ Valid
//   /users/abc     ✅ Matches (but ID should be numeric!)
//   /users/@#$     ✅ Matches (but invalid!)
```

**Desired API:**
```rust
// Not implemented
Route::from_path("pages/users/[id:numeric].rhtml", "pages")
Route::from_path("pages/posts/[slug:alphanum].rhtml", "pages")
Route::from_path("pages/api/[version:v\\d+].rhtml", "pages")  // Regex
```

**Impact:** **MEDIUM** - Validation currently done after matching

---

### 7. No Route Aliases

**Problem:**
Can't have multiple patterns map to the same handler.

**Use Cases:**
- Legacy URL support
- Internationalization (`/about`, `/über`, `/acerca`)
- URL shortening
- Multiple valid spellings

**Desired API:**
```rust
// Not implemented
Route::from_path("pages/about.rhtml", "pages")
    .with_aliases(["/about-us", "/company", "/über"])
```

**Impact:** **MEDIUM** - Common requirement for migrations

---

### 8. No Named Routes or URL Generation

**Problem:**
Can't refer to routes by name or generate URLs from parameters.

**Use Cases:**
- Type-safe route references
- URL generation for links/redirects
- Route refactoring (change pattern, keep name)

**Desired API:**
```rust
// Not implemented
Route::from_path("pages/users/[id].rhtml", "pages")
    .with_name("user.profile")

// Later:
let url = router.url_for("user.profile", &[("id", "123")]);
// Returns: "/users/123"
```

**Impact:** **MEDIUM** - Quality of life improvement

---

### 9. No Redirect Routes

**Problem:**
Can't define redirect-only routes in the router.

**Use Cases:**
- Old URL → New URL redirects
- Canonical URL enforcement
- Shortlinks

**Desired API:**
```rust
// Not implemented
Route::redirect("/old-path", "/new-path", 301)
Route::redirect("/blog/*", "/articles/*", 302)
```

**Impact:** **MEDIUM** - Common requirement

---

## ⚠️ MEDIUM PRIORITY: Pattern Matching

### 10. No Regex in Routes

**Problem:**
Can only match exact segments or dynamic parameters. No pattern matching within segments.

**Use Cases:**
- File extensions: `/assets/:filename.{js,css,png}`
- Versioning: `/api/v{1,2}/users`
- Complex patterns: `/posts/:year(\\d{4})/:slug`

**Impact:** **LOW-MEDIUM** - Can work around with catch-all + manual parsing

---

### 11. No Custom Parameter Matchers

**Problem:**
Related to #6 - can't define custom matchers like `{id:uuid}` or `{date:iso}`.

**Impact:** **MEDIUM** - Nice to have

---

### 12. No Route Versioning Support

**Problem:**
No built-in support for API versioning patterns.

**Desired:**
```rust
// Not implemented
router.version("v1", |v1| {
    v1.add_route(Route::from_path("pages/api/v1/users.rhtml", "pages"));
});
```

**Impact:** **LOW** - Can be done manually

---

## 🔧 MEDIUM PRIORITY: Performance

### 13. No Route Caching

**Problem:**
Every lookup walks through all routes or parent hierarchy. No caching.

**Potential:**
- Cache recent lookups
- LRU cache for layouts
- Pre-compute common paths

**Impact:** **LOW** for <1000 routes, **HIGH** for large applications

---

### 14. No Compiled Route Tree

**Problem:**
Current O(n) linear search through routes. Could use trie/radix tree for O(m) lookup.

**Performance:**
- Current: O(n) where n = number of routes
- Trie: O(m) where m = path length

**Impact:** **LOW** for <100 routes, **MEDIUM** for 100-1000 routes, **HIGH** for 1000+ routes

---

### 15. No Route Precompilation

**Problem:**
Routes parsed at runtime. Could precompile to static data structure.

**Impact:** **LOW** - Runtime parsing is already fast

---

## 🔧 LOW PRIORITY: Developer Experience

### 16. No Route Groups/Prefixes

**Problem:**
Can't group routes with common prefix.

**Desired:**
```rust
// Not implemented
router.group("/api", |api| {
    api.add_route(...);  // Automatically prefixed with /api
});
```

**Impact:** **LOW** - Convenience feature

---

### 17. No Route Debugging Tools

**Problem:**
No built-in tools to visualize routes, test patterns, or debug matching.

**Desired:**
- `router.print_routes()` - ASCII tree
- `router.explain_match("/path")` - Why it matched
- `router.test_pattern("/:id")` - Test pattern

**Impact:** **LOW** - Development convenience

---

### 18. No Route Comments/Documentation

**Problem:**
Can't attach documentation to routes.

**Impact:** **LOW** - Nice to have

---

## 📊 Priority Summary

| Priority | Feature | Impact |
|----------|---------|--------|
| 🚨 **CRITICAL** | Skip parent layouts | **HIGH** |
| 🚨 **CRITICAL** | No layout option | **HIGH** |
| 🚨 **CRITICAL** | Layout composition | **MEDIUM-HIGH** |
| ⚠️ **HIGH** | Middleware/guards | **HIGH** |
| ⚠️ **HIGH** | Route metadata | **MEDIUM-HIGH** |
| ⚠️ **HIGH** | Parameter constraints | **MEDIUM** |
| ⚠️ **HIGH** | Route aliases | **MEDIUM** |
| ⚠️ **HIGH** | Named routes | **MEDIUM** |
| ⚠️ **HIGH** | Redirect routes | **MEDIUM** |
| 🔧 **MEDIUM** | Regex patterns | **LOW-MEDIUM** |
| 🔧 **MEDIUM** | Route caching | **LOW-HIGH** (depends on scale) |
| 🔧 **MEDIUM** | Compiled route tree | **LOW-HIGH** (depends on scale) |
| 🔧 **LOW** | Route groups | **LOW** |
| 🔧 **LOW** | Debugging tools | **LOW** |

---

## 🎯 Recommended Implementation Order

### Phase 1: Layout Control (Critical)
1. **Skip parent layouts** - Add layout override mechanism
2. **No layout option** - Add `_nolayout` marker or similar
3. **Layout composition** - Named layouts system

**Estimated Effort:** 2-3 days

---

### Phase 2: Security & Metadata (High Priority)
4. **Middleware/guards** - Add middleware chain
5. **Route metadata** - Add HashMap for arbitrary metadata
6. **Parameter constraints** - Add basic type constraints

**Estimated Effort:** 3-5 days

---

### Phase 3: Flexibility (High Priority)
7. **Route aliases** - Allow multiple patterns per route
8. **Named routes** - Add route naming system
9. **Redirect routes** - Add redirect route type

**Estimated Effort:** 2-3 days

---

### Phase 4: Performance (If Needed)
10. **Route caching** - Add LRU cache
11. **Compiled route tree** - Replace Vec with trie/radix tree

**Estimated Effort:** 5-7 days

---

### Phase 5: Polish (Low Priority)
12. **Regex patterns** - Add regex support
13. **Route groups** - Add grouping API
14. **Debugging tools** - Add visualization

**Estimated Effort:** 3-5 days

---

## 🏗️ Proposed API Designs

### Layout Control

```rust
// Option 1: Special marker file
pages/dashboard/print/_nolayout    # Empty marker file

// Option 2: Layout override in frontmatter (requires file parsing)
---
layout: none
---

// Option 3: Programmatic (breaking change)
Route::from_path("pages/print.rhtml", "pages")
    .with_layout(LayoutOption::None)
    .with_layout(LayoutOption::Root)
    .with_layout(LayoutOption::Named("admin"))

// Option 4: Special filename convention
pages/dashboard/print.nolayout.rhtml
pages/dashboard/settings.rootlayout.rhtml
```

**Recommendation:** Option 1 (marker file) - no breaking changes, clear intent

---

### Middleware

```rust
trait Middleware {
    fn handle(&self, route: &Route) -> Result<(), Error>;
}

router.add_middleware(Box::new(AuthMiddleware));
router.add_middleware(Box::new(LoggingMiddleware));

// Per-route middleware
Route::from_path("pages/admin/users.rhtml", "pages")
    .with_middleware(RequireAdmin)
```

---

### Metadata

```rust
Route::from_path("pages/users/[id].rhtml", "pages")
    .with_meta("title", "User Profile")
    .with_meta("permission", "users.read")
    .with_meta("cache_ttl", 300)

// Access
let title = route.meta.get("title");
```

---

### Parameter Constraints

```rust
// Simple types
Route::from_path("pages/users/[id:int].rhtml", "pages")
Route::from_path("pages/posts/[slug:slug].rhtml", "pages")
Route::from_path("pages/tags/[name:alpha].rhtml", "pages")

// Regex
Route::from_path("pages/api/[version:v\\d+].rhtml", "pages")

// Custom
Route::from_path("pages/users/[id:uuid].rhtml", "pages")
```

---

## 📝 Notes

### What Currently Works Well
- ✅ Basic routing (static, dynamic, optional, catch-all)
- ✅ Nested layouts with inheritance
- ✅ Error pages
- ✅ Path normalization
- ✅ Performance (for small-medium apps)

### What's Blocking Production Use
- ❌ Layout control (critical for real apps)
- ❌ Middleware (critical for security)
- ❌ Metadata (needed for titles, SEO, etc.)

### Workarounds Available
- **No layout:** Create empty `_layout.rhtml`
- **Middleware:** Implement at handler level
- **Metadata:** Store separately, lookup by pattern
- **Constraints:** Validate after matching

---

## 🤝 Contributing

If you'd like to implement any of these features:

1. Open an issue to discuss approach
2. Reference this document
3. Consider backward compatibility
4. Add comprehensive tests
5. Update documentation

---

## 📚 References

### Similar Projects
- **Next.js** - File-system routing with layouts
- **SvelteKit** - Nested layouts, load functions
- **Remix** - Nested routes with loaders
- **Rails** - Constraints, named routes
- **Actix-web** - Guards, middleware

### Useful Patterns
- **Trie/Radix tree** for O(m) matching
- **LRU cache** for hot paths
- **Middleware chain** for cross-cutting concerns
- **Builder pattern** for route configuration
