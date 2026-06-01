---
task_id: 09-sync
task_name: Sync Protocol and HTTP API
sequence: 9
dependencies_complete: [01-workspace, 03-types, 04-crypto, 05-vault-core, 06-storage]
estimated_duration: 4-5 hours
difficulty: High
blockers_resolved: none
can_parallelize: false
---

# Task 9: Sync Protocol and HTTP API

## 🎯 Objective

Implement `porkpie-sync` and `porkpie-api` crates for encrypted vault synchronization. Server stores encrypted blobs only.

## ✅ Acceptance Criteria

**Sync Protocol (porkpie-sync)**
- [ ] Revision-based sync logic
- [ ] `SyncRequest` type (vault_id, last_revision)
- [ ] `SyncResponse` type (items, new_revision)
- [ ] Conflict detection (same item modified locally + remotely)
- [ ] Merge strategy (last-write-wins configurable)
- [ ] `sync_vault()` function for client
- [ ] Tests for conflict resolution

**HTTP API (porkpie-api)**
- [ ] Axum HTTP server
- [ ] SQLite persistence (encrypted data only)
- [ ] API key authentication (not password-based)
- [ ] HTTPS/TLS ready (Docker)

**Routes**
- [ ] `POST /api/v1/sync/begin` → Send local revision, get server changes
- [ ] `POST /api/v1/sync/push` → Upload encrypted items
- [ ] `GET /api/v1/health` → Server health check
- [ ] `GET /api/v1/status` → Server version + time

**Request/Response Format**
- [ ] JSON serialization
- [ ] Content-Type: application/json
- [ ] Error responses standardized

**Authentication**
- [ ] API key header: `Authorization: Bearer {api_key}`
- [ ] API key validated before each request
- [ ] Reject requests without valid API key

**Server Database**
- [ ] Same SQLite schema as client (vaults, items tables)
- [ ] Server tables: api_keys, audit_log
- [ ] Replication-ready (eventually)

**Error Handling**
- [ ] Invalid vault ID → 404
- [ ] Authentication failed → 401
- [ ] Sync conflict → 409 (return conflict data)
- [ ] Server error → 500 with message
- [ ] Validation error → 422

**Docker Deployment**
- [ ] `Dockerfile` for API server
- [ ] `docker-compose.yml` with server + SQLite
- [ ] Environment variables: DATABASE_URL, API_PORT, API_KEY
- [ ] Server starts with `docker-compose up`

**Tests**
- [ ] Sync request/response serialization
- [ ] Conflict detection logic
- [ ] API routes return correct status codes
- [ ] Authentication validation
- [ ] Database operations (no decryption)

**Code Quality**
- [ ] `cargo fmt` passes
- [ ] `cargo clippy` passes (zero warnings)
- [ ] All functions documented

## 📋 Output Specification

### File Structure

```
crates/porkpie-sync/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── protocol.rs             # Sync types + logic
│   ├── conflict.rs             # Conflict detection + merge
│   ├── state.rs                # Sync state tracking
│   └── errors.rs               # Sync-specific errors
└── tests/
    └── sync.rs                 # Sync protocol tests

crates/porkpie-api/
├── Cargo.toml
├── src/
│   ├── main.rs                 # Server entry point
│   ├── handlers.rs             # Route handlers
│   ├── models.rs               # Request/response types
│   ├── auth.rs                 # API key validation
│   ├── errors.rs               # API errors
│   ├── db.rs                   # Database operations
│   └── config.rs               # Server config
├── tests/
│   └── api.rs                  # Integration tests
├── Dockerfile
└── docker-compose.yml
```

### Cargo.toml (porkpie-sync)

```toml
[package]
name = "porkpie-sync"
version = "0.1.0"
edition = "2021"

[dependencies]
porkpie-types = { path = "../porkpie-types" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"

[dev-dependencies]
```

### Cargo.toml (porkpie-api)

```toml
[package]
name = "porkpie-api"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "porkpie-server"
path = "src/main.rs"

[dependencies]
porkpie-types = { path = "../porkpie-types" }
porkpie-sync = { path = "../porkpie-sync" }
porkpie-store = { path = "../porkpie-store" }
axum = "0.7"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors"] }

[dev-dependencies]
```

### Example: Sync Protocol (`crates/porkpie-sync/src/protocol.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub vault_id: String,
    pub last_revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub items: HashMap<String, Vec<u8>>,  // item_id → encrypted ciphertext
    pub new_revision: u64,
    pub conflicts: Vec<ConflictItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictItem {
    pub item_id: String,
    pub local_revision: u64,
    pub server_revision: u64,
    pub server_data: Vec<u8>,  // encrypted
}

pub async fn sync_vault(
    client_items: HashMap<String, Vec<u8>>,
    server_response: &SyncResponse,
) -> Result<()> {
    // Merge server items with client items
    // Detect conflicts (same item_id with different revision)
    // Return merged state or conflict list
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_no_conflicts() {
        // Client has item A, server has item B
        // Expected: merged state has both A and B
    }

    #[test]
    fn sync_with_conflict() {
        // Client modified item A at rev 5, server modified at rev 4
        // Expected: conflict reported
    }
}
```

### Example: API Handler (`crates/porkpie-api/src/handlers.rs`)

```rust
use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};

#[derive(serde::Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: i64,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: "0.1.0".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    })
}

pub async fn sync_begin(
    State(db): State<SqlitePool>,
    Json(req): Json<porkpie_sync::SyncRequest>,
) -> Result<Json<porkpie_sync::SyncResponse>, ApiError> {
    // Load vault from database
    // Get items since last_revision
    // Return response
    
    Ok(Json(porkpie_sync::SyncResponse {
        items: HashMap::new(),
        new_revision: 1,
        conflicts: Vec::new(),
    }))
}

pub async fn sync_push(
    State(db): State<SqlitePool>,
    Json(req): Json<SyncPushRequest>,
) -> Result<StatusCode, ApiError> {
    // Store encrypted items to database
    // Update revision
    
    Ok(StatusCode::OK)
}
```

### Example: Server Entry (`crates/porkpie-api/src/main.rs`)

```rust
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::sqlite::SqlitePool;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or("sqlite:porkpie.db".to_string());
    
    let pool = SqlitePool::connect(&database_url).await?;
    
    // Run migrations
    sqlx::migrate!().run(&pool).await?;

    // Build router
    let app = Router::new()
        .route("/api/v1/health", get(handlers::health))
        .route("/api/v1/sync/begin", post(handlers::sync_begin))
        .route("/api/v1/sync/push", post(handlers::sync_push))
        .with_state(pool);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    println!("🚀 Server listening on 0.0.0.0:8000");
    
    axum::serve(listener, app).await?;

    Ok(())
}
```

### Docker Compose (`docker-compose.yml`)

```yaml
version: '3.8'

services:
  porkpie-server:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "8000:8000"
    environment:
      DATABASE_URL: sqlite:porkpie.db
      API_PORT: 8000
      API_KEY: "dev-key-change-in-production"
    volumes:
      - ./data:/app/data

  caddy:
    image: caddy:latest
    ports:
      - "443:443"
    volumes:
      - ./infra/caddy/Caddyfile:/etc/caddy/Caddyfile
```

## 🔗 References

- **Vault Core:** Task 5 (porkpie-core)
- **Storage:** Task 6 (porkpie-store)
- **Sync Protocol Details:** Porkpie Architecture and Coding Plan — Section 5

## ✔️ Success Verification

```bash
# Build sync crate
cargo build --package porkpie-sync

# Build API crate
cargo build --package porkpie-api

# Tests
cargo test --package porkpie-sync
cargo test --package porkpie-api

# Lint
cargo clippy --package porkpie-sync -- -D warnings
cargo clippy --package porkpie-api -- -D warnings

# Try starting server
docker-compose up --build
# Wait for "Server listening on 0.0.0.0:8000"
# Then: curl http://localhost:8000/api/v1/health
```

## 🚨 If Blocked...

| Problem | Solution |
|---------|----------|
| "Axum routing confusing" | Use Router builder with `.route(path, handler)`. handlers return `Json<T>` or status codes. |
| "Sync protocol complex" | Start simple: client sends vault_id + last_rev. Server returns new items. No conflicts yet. |
| "API key auth complex" | Use middleware: `extract Authorization header, lookup key in DB, reject if not found`. |
| "Docker compose intimidating" | Template provided. Just adjust ports/paths. `docker-compose up` starts server. |

## 🔒 STRICT TYPECHECK REQUIREMENTS

**Type safety is non-negotiable.** Rust's type system is your first line of defense.

- ✓ **All type errors must compile** — `cargo build` must succeed with zero type errors
- ✓ **No `unsafe` blocks without justification** — Document why in code comment
- ✓ **No unchecked casts** — Use `as` only where necessary (document reasoning)
- ✓ **No `unwrap()` on external input** — Use `.map_err()` or `?` operator
- ✓ **No `todo!()` or `unimplemented!()` in production code** — Only in stubs
- ✓ **Compiler warnings are failures** — `cargo clippy` must have zero warnings
- ✓ **Type inference must be clear** — Add explicit types where ambiguous
- ✓ **Trait bounds must be explicit** — Don't hide requirements

**Verification command:**
```bash
cargo check --workspace
cargo build --workspace
```

**If ANY type error appears, stop and fix it. Type errors = broken code.**

## 📌 What Comes Next

**Task 10: Import/Export and Final Validation**

Next agent will finish import/export and run full test suite. Project ready for MVP.

---

**Status:** Ready for agent assignment
