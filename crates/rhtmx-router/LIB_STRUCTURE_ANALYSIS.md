# RHTMX Router - lib.rs Structure Analysis (5127 lines)

## 1. PUBLIC ENUMS, STRUCTS & THEIR LINE RANGES

### Public Enums

#### `ParameterConstraint` (Lines 56-126)
- **Definition**: Lines 56-73
- **Impl Block**: Lines 75-126
- **Key Methods**:
  - `validate(&self, value: &str) -> bool` (79-107)
  - `from_str(s: &str) -> Self` (112-125)
- **Variants**: Any, Int, UInt, Alpha, AlphaNum, Slug, Uuid, Regex(String)

#### `LayoutOption` (Lines 135-152)
- **Definition**: Lines 135-146
- **Impl Block (Default)**: Lines 148-152
- **Variants**: Inherit, None, Root, Named(String), Pattern(String)

#### `InterceptLevel` (Lines 156-165)
- **Definition**: Lines 156-165
- **No impl block (simple enum)**
- **Variants**: SameLevel, OneLevelUp, FromRoot, TwoLevelsUp

### Public Structs

#### `Route` (Lines 169-1383)
- **Definition**: Lines 169-222 (24 public fields + metadata HashMap)
- **Impl Block**: Lines 365-1383
- **Key Public Methods**:
  - `from_path(file_path: &str, pages_dir: &str)` (389-457)
  - `matches(&self, path: &str)` (688-690)
  - `matches_with_options(&self, path, case_insensitive)` (698-844)
  - `layout_pattern(&self)` (845-876)
  - `with_layout_option(...)` (877-891)
  - `with_no_layout(...)` (892-905)
  - `with_root_layout(...)` (906-919)
  - `with_named_layout(...)` (920-933)
  - `with_layout_pattern(...)` (934-957)
  - `with_meta(key, value)` (958-977)
  - `with_metadata(metadata)` (978-995)
  - `get_meta(key)` (996-1012)
  - `has_meta(key)` (1013-1040)
  - `with_alias(alias)` (1042-1058)
  - `with_aliases(aliases)` (1059-1085)
  - `matches_any(path)` (1086-1106)
  - `with_name(name)` (1143-1168)
  - `generate_url(params)` (1169-1259)
  - `redirect(from_pattern, to_url, status)` (1260-1355)
  - `redirect_target(params)` (1356-1383)
- **Relationships**: Associated with RouteMatch

#### `RouteMatch` (Lines 226-289)
- **Definition**: Lines 226-231 (2 public fields: route, params)
- **Impl Block**: Lines 233-289
- **Key Public Methods**:
  - `is_redirect(&self)` (247-249)
  - `redirect_target(&self)` (267-269)
  - `redirect_status(&self)` (286-288)

#### `Router` (Lines 1507-2186)
- **Definition**: Lines 1507-1520 (11 internal collections)
- **Impl Block**: Lines 1522-2186
- **Default Impl**: Lines 2187-2191
- **Key Public Methods**:
  - **Construction**:
    - `new()` (1524-1539)
    - `with_case_insensitive(bool)` (1550-1565)
    - `set_case_insensitive(&mut self, bool)` (1568-1570)
  
  - **Route Management**:
    - `add_route(&mut self, route)` (1593-1647)
    - `remove_route(&mut self, pattern)` (1648-1676)
    - `sort_routes(&mut self)` (1677-1693)
  
  - **Route Matching**:
    - `match_route(&self, path)` (1724-1789)
  
  - **Layout Retrieval** (30+ methods):
    - `get_layout(pattern)` (1790-1825)
    - `get_layout_for_match(route_match)` (1826-1835)
    - `get_layout_with_option(...)` (1836-1871)
    - `get_layout_by_name(name)` (1893-1897)
    - `layouts()` (1903-1905)
  
  - **Specialized Page Getters**:
    - `get_error_page(pattern)` (1932-1936)
    - `error_pages()` (1937-1939)
    - `get_loading_page(pattern)` (1960-1964)
    - `loading_pages()` (1965-1967)
    - `get_template(pattern)` (1982-1986)
    - `templates()` (1987-1989)
    - `get_not_found_page(pattern)` (2004-2008)
    - `not_found_pages()` (2009-2011)
  
  - **Parallel Routes** (Phase 5.1):
    - `get_parallel_routes(pattern)` (2035-2039)
    - `parallel_routes()` (2040-2042)
    - `get_parallel_route(pattern, slot)` (2057-2075)
  
  - **Intercepting Routes** (Phase 5.2):
    - `get_intercepting_route(pattern)` (2076-2080)
    - `intercepting_routes()` (2081-2083)
  
  - **URL Generation & Named Routes**:
    - `url_for(name, params)` (2125-2154)
    - `url_for_params(name, params_slice)` (2155-2181)
    - `get_route_by_name(name)` (2182-2186)
    - `routes()` (1898-1902)

---

## 2. PRIVATE HELPER FUNCTIONS & THEIR LINE RANGES

### Private Enums/Structs

#### `PatternSegmentType` (Lines 293-304)
- **Private enum** for classifying route segments
- **Variants**:
  - CatchAll(String, Option<ParameterConstraint>)
  - OptionalCatchAll(String, Option<ParameterConstraint>)
  - Optional(String, Option<ParameterConstraint>)
  - Required(String, Option<ParameterConstraint>)
  - Static(String)

#### `PathHierarchy<'a>` (Lines 1452-1486)
- **Private struct** for lazy iteration over path hierarchy
- **Impl blocks**:
  - `fn new(path: &'a str)` (1457-1461)
  - **Iterator Impl** (1464-1486) with `next()` method
- **Purpose**: Generates parent paths on-demand without allocation

### Private Standalone Functions

#### `classify_segment(segment: &str) -> PatternSegmentType` (Lines 318-348)
- Classifies route pattern segments
- Handles: catch-all, optional, required, static
- Uses functional pattern matching

#### `parse_param_with_constraint(param: &str) -> (String, Option<ParameterConstraint>)` (Lines 353-363)
- Parses parameter names and constraints from "name" or "name:constraint"
- Functional parser using split_once()

#### `is_valid_path(path: &str) -> bool` (Lines 1384-1420)
- Validates path format
- Checks for invalid characters
- ~50 lines of validation logic

#### `normalize_path(path: &str) -> Cow<'_, str>` (Lines 1421-1450)
- Zero-copy normalization for valid paths (borrowed)
- Single allocation for invalid paths
- Handles trailing slashes, double slashes, backslashes

### Private Route Methods

#### `Route::extract_layout_name(filename: &str) -> Option<String>` (Lines 465-470)
- Extracts name from "_layout.name" format

#### `Route::detect_parallel_route(path: &str) -> (bool, Option<String>)` (Lines 478-487)
- Detects @slot_name markers (Phase 5.1)

#### `Route::detect_intercepting_route(path: &str) -> (bool, Option<InterceptLevel>, Option<String>)` (Lines 496-520)
- Detects (.), (..), (...), (....) markers (Phase 5.2)

#### `Route::parse_pattern(path: &str) -> Tuple` (Lines 525-654)
- Parses file path into route components
- Returns 6-tuple: (pattern, params, optional_params, dynamic_count, has_catch_all, constraints)
- ~130 lines of parsing logic

#### `Route::calculate_priority(has_catch_all, dynamic_count, depth, optional_params) -> usize` (Lines 664-685)
- Priority calculation: static(0) < dynamic(1-999) < catch-all(1000+)
- Optional catch-all gets lower priority than required

### Private Router Methods

#### `Router::get_scoped_resource<'a>(&'a self, scope: &str, pattern: &str) -> Option<&'a Route>` (Lines 1694-1723)
- Generic method for looking up scoped resources
- Used internally for hierarchical resource resolution

#### `Router::is_under_nolayout_marker(&self, pattern: &str) -> bool` (Lines 1872-1892)
- Checks if pattern is under a _nolayout marker directory

---

## 3. TEST MODULE LOCATION & SIZE

### Test Module
- **Location**: Lines 2197-5127 (end of file)
- **Start Marker**: `#[cfg(test)]` at line 2197, `mod tests {` at line 2198
- **Total Lines**: **2930 lines** (approximately 57% of total file!)
- **Last test**: `test_phase_5_with_all_previous_features()` around line 5097
- **Module closure**: Line 5127

### Test Categories (Roughly grouped):

1. **Route Parsing & Matching** (~50 tests, lines 2202-2302)
   - Static routes, dynamic routes, index handling
   - Basic routing, priorities

2. **Path Normalization** (~10 tests, lines 2575-2608)
   - Trailing slashes, double slashes, backslashes
   - Path hierarchy iterator

3. **Layout Options** (~15 tests, lines 2626-2807)
   - Layout inheritance, root layout, named layouts
   - Layout patterns and builder chaining

4. **No-Layout Markers** (~8 tests, lines 2842-2939)
   - Marker detection, hierarchy, overrides

5. **Metadata** (~8 tests, lines 2939-3054)
   - Meta setting/getting, batch operations
   - Chaining, override behavior

6. **Parameter Constraints** (~15 tests, lines 3054-3266)
   - Int, UInt, Alpha, AlphaNum, Slug, UUID validation
   - Constraint parsing and composition

7. **Route Aliases** (~20 tests, lines 3266-3529)
   - Single/multiple aliases, chaining
   - Alias matching, priority, i18n support

8. **Named Routes & URL Generation** (~30 tests, lines 3529-4155)
   - URL generation with params
   - Named route lookup, functional composition

9. **Redirects** (~25 tests, lines 3842-4155)
   - Static/dynamic redirects, status codes
   - Redirect chain prevention, i18n

10. **Optional Catch-All** (~20 tests, lines 4155-4420)
    - Zero/single/multiple segment matching
    - Constraint support, priority ordering

11. **Route Groups (Phase 4.2)** (~15 tests, lines 4418-4629)
    - Basic groups, nested groups, organization

12. **Loading UI, Templates, Not-Found (Phases 4.3-4.5)** (~15 tests, lines 4629-4795)
    - Special file detection and hierarchical resolution

13. **Parallel Routes (Phase 5.1)** (~12 tests, lines 4816-4937)
    - Slot detection, multiple slots, nested parallel routes

14. **Intercepting Routes (Phase 5.2)** (~10 tests, lines 4937-5065)
    - Interception levels, modal patterns, real-world use cases

15. **Integration Tests** (~5 tests, lines 5065-5127)
    - All features together across phases

---

## 4. LOGICAL GROUPINGS FOR SEPARATE MODULES

### Current Monolithic Structure (5127 lines)

The file can be refactored into these logical modules:

### A. **Core Type System Module** (~290 lines)
**Files to extract**:
- `constraint.rs` - Parameter constraints system
  - Lines 56-126: ParameterConstraint enum + impl
  - ~70 lines
  - Clean: No dependencies on Router

- `layout.rs` - Layout resolution system
  - Lines 135-152: LayoutOption enum + impl
  - ~20 lines
  - Can be separate or merge with constraint

- `intercept.rs` - Intercepting routes system
  - Lines 156-165: InterceptLevel enum
  - ~10 lines
  - Clean dependency boundary

### B. **Routing Core Module** (~1220 lines)
**Files to extract**:

- `route.rs` - Route struct and matching logic
  - Lines 169-222: Route definition
  - Lines 365-1383: Route impl (all methods)
  - **Key sections**:
    - Parsing: Lines 389-654
    - Matching: Lines 688-844
    - Layout builders: Lines 845-957
    - Metadata builders: Lines 958-1040
    - Alias builders: Lines 1042-1106
    - Named routes: Lines 1143-1259
    - Redirects: Lines 1260-1383
  
- `pattern.rs` - Pattern parsing and classification
  - Lines 293-304: PatternSegmentType enum
  - Lines 318-348: classify_segment()
  - Lines 353-363: parse_param_with_constraint()
  - Lines 465-520: Route detection methods (parallel, intercepting)
  - Lines 525-654: parse_pattern()
  - Lines 664-685: calculate_priority()
  - **Total**: ~150 lines, pure functional, no Router dependency

- `route_match.rs` - Route matching result wrapper
  - Lines 226-289: RouteMatch struct + impl
  - ~65 lines
  - Clean dependency boundary

### C. **Path Normalization Module** (~70 lines)
**File to extract**: `path.rs`
- Lines 1384-1450: Path utilities
  - `is_valid_path()` (37 lines)
  - `normalize_path()` (30 lines)
- Pure functions, zero dependencies

### D. **Path Hierarchy Module** (~35 lines)
**File to extract**: `hierarchy.rs`
- Lines 1452-1486: PathHierarchy iterator
  - Custom iterator for lazy path traversal
  - Performance-critical component
  - Zero-copy with lifetime bounds

### E. **Router Core Module** (~680 lines)
**File to extract**: `router/mod.rs` or `router_core.rs`
- Lines 1507-2191: Router struct and impl
- **Subdivide internally**:
  
  - **Construction** (Lines 1524-1570): new(), with_case_insensitive()
  
  - **Route Management** (Lines 1593-1693): add_route(), remove_route(), sort_routes()
  
  - **Route Matching** (Lines 1724-1789): match_route(), get_scoped_resource()
  
  - **Layout Resolution** (Lines 1790-1905): 
    - get_layout(), get_layout_for_match(), get_layout_with_option()
    - get_layout_by_name(), is_under_nolayout_marker(), layouts()
    
  - **Specialized Pages** (Lines 1932-2011):
    - Error pages, loading pages, templates, not-found pages
    - Accessor methods for all 4 collections
    
  - **Parallel Routes** (Lines 2035-2075):
    - get_parallel_routes(), parallel_routes(), get_parallel_route()
    
  - **Intercepting Routes** (Lines 2076-2083):
    - get_intercepting_route(), intercepting_routes()
    
  - **URL Generation** (Lines 2125-2186):
    - url_for(), url_for_params(), get_route_by_name()

### F. **Tests Module** (~2930 lines)
**File to extract**: `tests/mod.rs` or `lib_tests.rs`

Can be further subdivided:
- `tests/routing_tests.rs` - Route parsing, matching, priorities
- `tests/layout_tests.rs` - Layout resolution, nolayout markers
- `tests/metadata_tests.rs` - Metadata operations
- `tests/constraints_tests.rs` - Parameter constraint validation
- `tests/alias_tests.rs` - Route aliases and matching
- `tests/named_routes_tests.rs` - URL generation, named routes
- `tests/redirect_tests.rs` - Redirect routes and handling
- `tests/optional_catchall_tests.rs` - Optional catch-all patterns
- `tests/route_groups_tests.rs` - Route group organization
- `tests/special_files_tests.rs` - Loading UI, templates, not-found
- `tests/parallel_routes_tests.rs` - Parallel route handling
- `tests/intercepting_routes_tests.rs` - Intercepting route modals
- `tests/integration_tests.rs` - Cross-feature integration

---

## 5. RECOMMENDED MODULE STRUCTURE (Post-Refactor)

```
rhtmx-router/src/
├── lib.rs (re-exports only, ~50 lines)
├── constraint.rs (ParameterConstraint enum, ~70 lines)
├── layout.rs (LayoutOption enum, ~20 lines)
├── intercept.rs (InterceptLevel enum, ~10 lines)
├── route/
│   ├── mod.rs (Route struct definition + impl, ~1200 lines)
│   ├── pattern.rs (Pattern parsing, ~150 lines)
│   └── match_result.rs (RouteMatch, ~65 lines)
├── path/
│   ├── mod.rs (Path normalization utilities, ~70 lines)
│   └── hierarchy.rs (PathHierarchy iterator, ~35 lines)
├── router/
│   ├── mod.rs (Router main impl, ~680 lines)
│   ├── construction.rs (new(), with_case_insensitive(), etc.)
│   ├── route_management.rs (add_route, remove_route, sort)
│   ├── matching.rs (match_route, get_scoped_resource)
│   ├── layout.rs (All layout-related methods)
│   ├── pages.rs (Error, loading, template, not-found pages)
│   ├── parallel.rs (Parallel route methods)
│   ├── intercepting.rs (Intercepting route methods)
│   └── url_generation.rs (URL generation and named routes)
└── tests/
    ├── mod.rs (test setup)
    ├── routing.rs (~300 lines)
    ├── layout.rs (~250 lines)
    ├── metadata.rs (~100 lines)
    ├── constraints.rs (~200 lines)
    ├── aliases.rs (~250 lines)
    ├── named_routes.rs (~300 lines)
    ├── redirects.rs (~250 lines)
    ├── optional_catchall.rs (~250 lines)
    ├── route_groups.rs (~150 lines)
    ├── special_files.rs (~150 lines)
    ├── parallel_routes.rs (~120 lines)
    ├── intercepting_routes.rs (~100 lines)
    └── integration.rs (~50 lines)
```

---

## 6. DEPENDENCY GRAPH FOR REFACTORING

```
lib.rs (re-exports)
  ├── constraint.rs (no deps)
  ├── layout.rs (no deps)
  ├── intercept.rs (no deps)
  ├── route/
  │   ├── pattern.rs (deps: constraint, intercept)
  │   ├── mod.rs (deps: constraint, layout, pattern, intercept)
  │   └── match_result.rs (deps: route)
  ├── path/
  │   ├── hierarchy.rs (no deps)
  │   └── mod.rs (deps: hierarchy)
  ├── router/
  │   ├── mod.rs (deps: route, path, constraint, layout, intercept)
  │   ├── construction.rs (part of router/mod.rs)
  │   ├── route_management.rs (part of router/mod.rs)
  │   ├── matching.rs (part of router/mod.rs, deps: route)
  │   ├── layout.rs (part of router/mod.rs, deps: path)
  │   ├── pages.rs (part of router/mod.rs)
  │   ├── parallel.rs (part of router/mod.rs)
  │   ├── intercepting.rs (part of router/mod.rs)
  │   └── url_generation.rs (part of router/mod.rs, deps: route)
  └── tests/ (deps: all modules)
```

---

## 7. REFACTORING PRIORITY & IMPACT

### High Priority (Core abstractions)
1. **constraint.rs** - Self-contained, zero dependencies
2. **pattern.rs** - Pure functions, no Router dependency
3. **path/mod.rs** + **hierarchy.rs** - Performance critical, standalone
4. **layout.rs**, **intercept.rs** - Minimal enums, clean boundaries

### Medium Priority (Core functionality)
1. **route_match.rs** - Wrapper type, clean interface
2. **route/mod.rs** - Large but coherent, careful splitting needed
3. **router/** - Split into sub-modules for manageability

### Lower Priority (Tests)
1. **tests/** - Can be split last; all dependencies will be clear after core modules

---

## 8. KEY METRICS

| Aspect | Count |
|--------|-------|
| **Public Enums** | 3 (ParameterConstraint, LayoutOption, InterceptLevel) |
| **Public Structs** | 3 (Route, RouteMatch, Router) |
| **Public Methods** | 70+ (Route: 30+, Router: 40+) |
| **Private Enums** | 1 (PatternSegmentType) |
| **Private Structs** | 1 (PathHierarchy) |
| **Private Functions** | 4 standalone + 8+ impl helpers |
| **Test Functions** | 100+ tests in ~2930 lines |
| **Lines per Major Component** | Route: ~1200, Router: ~680, Tests: ~2930 |
| **Coupling** | Route <-> Router high, Path utils standalone |

