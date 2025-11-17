# Slots Framework Integration Guide

Complete practical guide for using RHTMX slots with different web frameworks, with working code examples.

## Table of Contents

1. [What Are Slots?](#what-are-slots)
2. [Core Principles](#core-principles)
3. [Framework Integration Examples](#framework-integration-examples)
4. [Dynamic Sidebar Loading Patterns](#dynamic-sidebar-loading-patterns)
5. [Layout Hierarchy Skipping Examples](#layout-hierarchy-skipping-examples)

---

## What Are Slots?

**Slots are plain Rust structs** that define the interface between a layout and the content being rendered.

```rust
pub struct Slots {
    pub title: String,
    pub description: Option<String>,
    pub header: Option<Html>,
    pub footer: Option<Html>,
    pub head_extra: Option<Html>,
}
```

Key characteristics:
- ✅ **Framework-agnostic** - Work with any web framework
- ✅ **Type-safe** - Compiler ensures all slots are valid
- ✅ **Zero runtime overhead** - Just Rust functions and structs
- ✅ **Builder pattern** - Intuitive and discoverable API
- ✅ **Composable** - Nest layouts and slots easily

---

## Core Principles

### 1. Slots Are Type Contracts

```rust
// This struct defines what data the root layout needs
pub struct Slots {
    pub title: String,                 // Required
    pub description: Option<String>,   // Optional
    pub header: Option<Html>,          // Optional
    pub footer: Option<Html>,          // Optional
    pub head_extra: Option<Html>,      // Optional
}

// Compiler ensures you provide the required `title`
// at compile time - no runtime surprises!
```

### 2. Builder Pattern for Ergonomics

```rust
// Clean, readable API
Slots::new("Page Title")
    .description("Meta description")
    .header(custom_header)
    .footer(custom_footer)

// vs. Error-prone struct construction
Slots {
    title: "Page Title".into(),
    description: Some("Meta description".into()),
    header: Some(custom_header),
    footer: Some(custom_footer),
    head_extra: None,  // Easy to forget
}
```

### 3. Decoupled from Content

```rust
// Layout doesn't care how content was generated
// Can use html!, templates, database, whatever

let content: Html = html! {
    <div>"Hello"</div>
};

let content: Html = Html(external_template_engine::render());

let content: Html = Html(database_query().render());

// All work the same with layouts
layout(content, Slots::new("Title"))
```

---

## Framework Integration Examples

### With Axum (Recommended)

Axum is the most common framework paired with RHTMX:

```rust
use axum::{
    routing::get,
    Router,
    response::Html as AxumHtml,
};
use rhtmx::{html, Html};
use rhtmx::layouts::root;

// Simple GET handler
async fn home() -> AxumHtml<String> {
    let content = html! {
        <div class="container">
            <h1>"Welcome to Axum + RHTMX"</h1>
            <p>"Type-safe, performant web application"</p>
        </div>
    };

    let page = root::layout(
        content,
        root::Slots::new("Home - MyApp")
            .description("Welcome to our application")
    );

    AxumHtml(page.0)
}

// Handler with state
#[derive(Clone)]
struct AppState {
    db: Database,
    config: AppConfig,
}

async fn dashboard(
    state: axum::extract::State<AppState>,
) -> AxumHtml<String> {
    let user = state.db.get_current_user().await?;

    let content = html! {
        <div class="dashboard">
            <h1>"Dashboard"</h1>
            <p>"Welcome, " {user.name}</p>
        </div>
    };

    let page = root::layout(
        content,
        root::Slots::new("Dashboard")
            .description(&format!("Welcome, {}", user.name))
    );

    AxumHtml(page.0)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(home))
        .route("/dashboard", get(dashboard));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

### With Rocket

Rocket provides native HTML response type:

```rust
#[macro_use]
extern crate rocket;

use rocket::response::content::RawHtml;
use rhtmx::{html, Html};
use rhtmx::layouts::{root, admin};

#[get("/")]
fn index() -> RawHtml<String> {
    let content = html! {
        <div class="hero">
            <h1>"Rocket + RHTMX"</h1>
            <p>"Dynamic web framework with type safety"</p>
        </div>
    };

    let page = root::layout(
        content,
        root::Slots::new("Home")
            .description("Welcome to Rocket")
    );

    RawHtml(page.0)
}

#[get("/admin/dashboard")]
fn admin_dashboard() -> RawHtml<String> {
    let content = html! {
        <div class="dashboard">
            <h1>"Admin Dashboard"</h1>
        </div>
    };

    let page = admin::layout(
        content,
        admin::Slots::new("Dashboard")
    );

    RawHtml(page.0)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, admin_dashboard])
}
```

### With Actix-Web

Actix requires manual response construction:

```rust
use actix_web::{web, App, HttpResponse, HttpServer};
use rhtmx::{html, Html};
use rhtmx::layouts::root;

async fn index() -> HttpResponse {
    let content = html! {
        <div class="hero">
            <h1>"Actix-Web + RHTMX"</h1>
            <p>"High-performance async web framework"</p>
        </div>
    };

    let page = root::layout(
        content,
        root::Slots::new("Home")
            .description("Welcome to Actix")
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page.0)
}

async fn api_list() -> HttpResponse {
    // No layout - just return JSON
    let items = vec![
        ("id", "1"),
        ("name", "Item 1"),
    ];

    let json = serde_json::json!(items);

    HttpResponse::Ok()
        .json(json)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/api/items", web::get().to(api_list))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

### With Warp

Warp uses functional composition:

```rust
use warp::Filter;
use rhtmx::{html, Html};
use rhtmx::layouts::root;

fn index() -> String {
    let content = html! {
        <div class="hero">
            <h1>"Warp + RHTMX"</h1>
            <p>"Composable web framework"</p>
        </div>
    };

    root::layout(
        content,
        root::Slots::new("Home")
    ).0
}

#[tokio::main]
async fn main() {
    let index_route = warp::path::end()
        .map(index);

    let routes = index_route;

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
```

### With Tonic (gRPC)

For hybrid gRPC + HTTP services:

```rust
use rhtmx::{html, Html};
use rhtmx::layouts::root;
use tonic::{Request, Response, Status};

// Regular HTTP handler for web UI
async fn admin_panel() -> String {
    let content = html! {
        <div class="admin">
            <h1>"Admin Panel"</h1>
            <p>"Manage your gRPC services"</p>
        </div>
    };

    root::layout(
        content,
        root::Slots::new("Admin")
    ).0
}

// Tonic service handler (can use same layouts!)
pub struct MyService;

#[tonic::async_trait]
impl my_service::my_service_server::MyService for MyService {
    async fn get_status(
        &self,
        request: Request<StatusRequest>,
    ) -> Result<Response<StatusReply>, Status> {
        // Business logic here
        Ok(Response::new(StatusReply {
            status: "ok".into(),
        }))
    }
}
```

---

## Dynamic Sidebar Loading Patterns

### Pattern 1: Role-Based Sidebar

Load different navigation based on user role:

```rust
use rhtmx::{html, Html};
use rhtmx::layouts::admin;

#[derive(Clone)]
pub enum Role {
    SuperAdmin,
    Admin,
    Moderator,
    User,
}

fn sidebar_for_role(role: Role) -> Html {
    match role {
        Role::SuperAdmin => html! {
            <nav class="sidebar">
                <a href="/admin" class="nav-item">"Dashboard"</a>
                <a href="/admin/users" class="nav-item">"Users"</a>
                <a href="/admin/roles" class="nav-item">"Roles"</a>
                <a href="/admin/settings" class="nav-item">"Settings"</a>
                <a href="/admin/audit" class="nav-item">"Audit Log"</a>
            </nav>
        },
        Role::Admin => html! {
            <nav class="sidebar">
                <a href="/admin" class="nav-item">"Dashboard"</a>
                <a href="/admin/users" class="nav-item">"Users"</a>
                <a href="/admin/settings" class="nav-item">"Settings"</a>
            </nav>
        },
        Role::Moderator => html! {
            <nav class="sidebar">
                <a href="/admin" class="nav-item">"Dashboard"</a>
                <a href="/admin/content" class="nav-item">"Content"</a>
            </nav>
        },
        Role::User => html! {
            <nav class="sidebar">
                <a href="/dashboard" class="nav-item">"My Dashboard"</a>
            </nav>
        },
    }
}

// In handler:
async fn admin_dashboard(user: User) -> String {
    let sidebar = sidebar_for_role(user.role.clone());

    let content = html! {
        <div class="dashboard">
            <h1>"Welcome, " {user.name}</h1>
        </div>
    };

    admin::layout(
        content,
        admin::Slots::new("Dashboard")
            .sidebar(sidebar)
    ).0
}
```

### Pattern 2: Database-Driven Dynamic Sidebar

Load menu items from database based on user permissions:

```rust
use rhtmx::{html, Html};
use rhtmx::layouts::admin;
use sqlx::PgPool;

#[derive(sqlx::FromRow)]
struct MenuItem {
    id: i32,
    label: String,
    href: String,
    icon: Option<String>,
    permission_required: Option<String>,
}

async fn load_sidebar_items(
    user_id: i32,
    db: &PgPool,
) -> Result<Vec<MenuItem>, sqlx::Error> {
    sqlx::query_as::<_, MenuItem>(
        r#"
        SELECT m.* FROM menu_items m
        LEFT JOIN permissions p ON m.permission_required = p.name
        WHERE p.id IS NULL  -- No permission required
           OR EXISTS (
               SELECT 1 FROM user_permissions up
               WHERE up.user_id = $1 AND up.permission_id = p.id
           )
        ORDER BY m.position
        "#
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}

fn render_menu_items(items: Vec<MenuItem>) -> Html {
    let mut html = String::from(r#"<nav class="sidebar"><ul>"#);

    for item in items {
        let icon = item.icon
            .map(|i| format!(r#"<span class="icon">{}</span>"#, i))
            .unwrap_or_default();

        html.push_str(&format!(
            r#"<li><a href="{}" class="nav-item">{}{}</a></li>"#,
            item.href, icon, item.label
        ));
    }

    html.push_str("</ul></nav>");
    Html(html)
}

async fn admin_dashboard(
    user_id: i32,
    db: web::Data<PgPool>,
) -> Result<String, ApiError> {
    let items = load_sidebar_items(user_id, &db).await?;
    let sidebar = render_menu_items(items);

    let content = html! {
        <div class="dashboard">
            <h1>"Dashboard"</h1>
        </div>
    };

    Ok(admin::layout(
        content,
        admin::Slots::new("Dashboard")
            .sidebar(sidebar)
    ).0)
}
```

### Pattern 3: Conditional Sidebar with Builder

```rust
use rhtmx::{html, Html};
use rhtmx::layouts::admin;

pub struct SidebarBuilder {
    items: Vec<(&'static str, &'static str)>,  // (label, href)
}

impl SidebarBuilder {
    pub fn new() -> Self {
        Self {
            items: vec![("Dashboard", "/admin")],
        }
    }

    pub fn with_users(mut self) -> Self {
        self.items.push(("Users", "/admin/users"));
        self
    }

    pub fn with_content(mut self) -> Self {
        self.items.push(("Content", "/admin/content"));
        self
    }

    pub fn with_settings(mut self) -> Self {
        self.items.push(("Settings", "/admin/settings"));
        self
    }

    pub fn build(self) -> Html {
        let nav_items = self.items
            .iter()
            .map(|(label, href)| {
                format!(r#"<li><a href="{}">{}</a></li>"#, href, label)
            })
            .collect::<String>();

        Html(format!(
            r#"<nav class="sidebar"><ul>{}</ul></nav>"#,
            nav_items
        ))
    }
}

// Usage:
let user_role = user.role;

let sidebar = SidebarBuilder::new()
    .with_users()
    .with_content()
    .with_settings()
    .build();

// Or conditional:
let sidebar = if user.is_admin {
    SidebarBuilder::new()
        .with_users()
        .with_content()
        .with_settings()
        .build()
} else {
    SidebarBuilder::new()
        .with_content()
        .build()
};
```

### Pattern 4: Context-Driven Sidebar

```rust
use rhtmx::{html, Html};
use rhtmx::layouts::admin;
use std::sync::Arc;

pub struct AppContext {
    user_id: i32,
    user_role: String,
    permissions: Vec<String>,
    theme: String,
}

fn build_sidebar_from_context(ctx: Arc<AppContext>) -> Html {
    let can_manage_users = ctx.permissions.contains(&"manage:users".to_string());
    let can_manage_content = ctx.permissions.contains(&"manage:content".to_string());
    let can_manage_roles = ctx.permissions.contains(&"manage:roles".to_string());

    let mut nav = String::from(r#"<nav class="sidebar" data-theme="{}">"#);
    nav = nav.replace("{}", &ctx.theme);

    nav.push_str(r#"<ul>"#);
    nav.push_str(r#"<li><a href="/admin">Dashboard</a></li>"#);

    if can_manage_users {
        nav.push_str(r#"<li><a href="/admin/users">Users</a></li>"#);
    }

    if can_manage_content {
        nav.push_str(r#"<li><a href="/admin/content">Content</a></li>"#);
    }

    if can_manage_roles {
        nav.push_str(r#"<li><a href="/admin/roles">Roles</a></li>"#);
    }

    nav.push_str("</ul></nav>");
    Html(nav)
}

async fn admin_dashboard(ctx: Arc<AppContext>) -> String {
    let sidebar = build_sidebar_from_context(ctx);

    let content = html! {
        <div class="dashboard">
            <h1>"Dashboard"</h1>
        </div>
    };

    admin::layout(
        content,
        admin::Slots::new("Dashboard")
            .sidebar(sidebar)
    ).0
}
```

---

## Layout Hierarchy Skipping Examples

### Example 1: Skip Parent, Use Root

**Scenario:** Print pages should use minimal layout without dashboard sidebar

```
pages/
├── _layout.rhtml           # Root: Full page with header/footer
├── dashboard/
│   ├── _layout.rhtml       # Dashboard: Adds sidebar
│   ├── index.rhtml         # Normal: Uses dashboard layout
│   └── print/
│       ├── report.rhtml    # SHOULD use root, not dashboard!
│       └── invoice.rhtml
```

**Solution with Router:**

```rust
use rhtmx_router::{Router, Route, LayoutOption};

let mut router = Router::new();

// Add layouts
router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
router.add_route(Route::from_path("pages/dashboard/_layout.rhtml", "pages"));

// Normal dashboard routes
router.add_route(
    Route::from_path("pages/dashboard/index.rhtml", "pages")
        .with_layout_option(LayoutOption::Inherit)  // Uses dashboard
);

// Print routes - skip to root
router.add_route(
    Route::from_path("pages/dashboard/print/report.rhtml", "pages")
        .with_root_layout()  // Skips dashboard!
);

router.add_route(
    Route::from_path("pages/dashboard/print/invoice.rhtml", "pages")
        .with_root_layout()  // Skips dashboard!
);

// Verify:
assert_eq!(
    router.get_layout("/dashboard/index").unwrap().pattern,
    "/dashboard"  // Dashboard layout
);

assert_eq!(
    router.get_layout("/dashboard/print/report").unwrap().pattern,
    "/"  // Root layout only!
);
```

### Example 2: Multiple Named Layouts

**Scenario:** Different sections need different layouts

```
pages/
├── _layout.rhtml              # Default
├── _layout.marketing.rhtml    # Marketing pages
├── _layout.admin.rhtml        # Admin pages
├── home.rhtml
├── marketing/
│   └── features.rhtml
└── admin/
    └── dashboard.rhtml
```

**Route Configuration:**

```rust
use rhtmx_router::{Router, Route};

let mut router = Router::new();

// Add layouts
router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
router.add_route(Route::from_path("pages/_layout.marketing.rhtml", "pages"));
router.add_route(Route::from_path("pages/_layout.admin.rhtml", "pages"));

// Routes with named layouts
router.add_route(
    Route::from_path("pages/marketing/features.rhtml", "pages")
        .with_named_layout("marketing")
);

router.add_route(
    Route::from_path("pages/admin/dashboard.rhtml", "pages")
        .with_named_layout("admin")
);

router.add_route(
    Route::from_path("pages/home.rhtml", "pages")
        // Uses default layout (no explicit layout specified)
);
```

### Example 3: Conditional Layout Based on Route Match

```rust
use rhtmx_router::{Router, Route, RouteMatch};

let mut router = Router::new();

// Add layouts
router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
router.add_route(Route::from_path("pages/api/_nolayout", "pages"));  // No layout zone

// In handler:
async fn render_page(path: &str, router: &Router) -> String {
    match router.match_route(path) {
        Some(route_match) => {
            let content = load_page_content(&route_match);

            // Get layout based on route
            let layout = router.get_layout(path);

            match layout {
                Some(layout) => {
                    // Render with layout
                    format!("{}{}", layout.template_path, content)
                }
                None => {
                    // Render without layout (HTMX partial or API)
                    content
                }
            }
        }
        None => "404 Not Found".to_string(),
    }
}
```

### Example 4: API Routes Without Layout

**Scenario:** API endpoints should never have layout, but regular pages should

```
pages/
├── _layout.rhtml
├── index.rhtml
└── api/
    ├── _nolayout        # Block all layouts under api/
    ├── users.json.rhtml
    └── posts.json.rhtml
```

**Verification:**

```rust
let mut router = Router::new();

router.add_route(Route::from_path("pages/_layout.rhtml", "pages"));
router.add_route(Route::from_path("pages/api/_nolayout", "pages"));

// Results:
assert!(router.get_layout("/").is_some());              // Has layout
assert!(router.get_layout("/index").is_some());         // Has layout
assert!(router.get_layout("/api/users").is_none());     // No layout!
assert!(router.get_layout("/api/posts").is_none());     // No layout!
```

---

## Complete Integration Example

Here's a real-world example combining everything:

```rust
use axum::{
    extract::{Path, State},
    routing::get,
    Router,
    response::Html as AxumHtml,
};
use rhtmx::{html, Html};
use rhtmx::layouts::{root, admin};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[derive(sqlx::FromRow, Clone)]
struct User {
    id: i32,
    name: String,
    email: String,
    role: String,
}

// Pattern 1: Simple page with root layout
async fn home() -> AxumHtml<String> {
    let page = root::layout(
        html! {
            <h1>"Welcome"</h1>
            <p>"Home page with default layout"</p>
        },
        root::Slots::new("Home")
    );
    AxumHtml(page.0)
}

// Pattern 2: Admin page with dynamic sidebar
async fn admin_dashboard(
    State(state): State<AppState>,
) -> AxumHtml<String> {
    let user = get_current_user(&state.db).await.unwrap();

    let sidebar = build_sidebar_for_user(&user);

    let page = admin::layout(
        html! {
            <h1>"Admin Dashboard"</h1>
            <p>"Welcome, " {user.name}</p>
        },
        admin::Slots::new("Admin")
            .sidebar(sidebar)
    );

    AxumHtml(page.0)
}

// Pattern 3: API endpoint (no layout)
async fn api_users(
    State(state): State<AppState>,
) -> String {
    let users = get_users(&state.db).await.unwrap();
    serde_json::to_string(&users).unwrap()
}

// Helper functions
fn build_sidebar_for_user(user: &User) -> Html {
    match user.role.as_str() {
        "admin" => html! {
            <nav class="sidebar">
                <a href="/admin/users">"Users"</a>
                <a href="/admin/settings">"Settings"</a>
            </nav>
        },
        _ => html! {
            <nav class="sidebar">
                <a href="/admin">"Dashboard"</a>
            </nav>
        },
    }
}

async fn get_current_user(db: &PgPool) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users LIMIT 1")
        .fetch_one(db)
        .await
}

async fn get_users(db: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(db)
        .await
}

#[tokio::main]
async fn main() {
    let db = PgPool::connect("postgres://user:pass@localhost/db").await.unwrap();

    let state = AppState { db };

    let app = Router::new()
        .route("/", get(home))
        .route("/admin", get(admin_dashboard))
        .route("/api/users", get(api_users))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

---

## Summary

**Slots work with any web framework because they're:**
- Plain Rust structs
- No special macros or runtime magic
- Just functions that return strings
- Fully type-safe at compile time
- Zero performance overhead

Pick your favorite framework and integrate RHTMX slots - they work everywhere!
