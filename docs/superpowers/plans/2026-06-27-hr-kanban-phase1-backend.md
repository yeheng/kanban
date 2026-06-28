# HR Kanban — Phase 1: Backend (Service + Command Layer) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Rust service + Tauri-command layer with repositories for projects, tasks (+dependencies, +skill-requirements, +tags), teams (+members, +overrides), skills, and tags — plus the IPC error model — so the Phase 1b frontend can drive full CRUD and render a Kanban.

**Architecture:** A new `app` crate sits above `db` and `domain`. It owns the IPC error model (`AppError{code,detail}` with `Serialize`), an `AppState` holding the `SqlitePool`, plain-Rust **services** (transactional, fully unit/integration-tested headlessly), and thin `#[tauri::command]` wrappers that delegate to services. New repositories are added to the `db` crate next to the Phase 0 `resources`/`allocations` repos. `domain::DomainError` (thiserror, **zero serde**) maps to `AppError` via an explicit `From` impl (design §6.4) — no `#[from]` embedding.

**Tech Stack:** Rust, `sqlx` (SQLite), `tokio`, `thiserror`, `serde` (on `db` row models + `AppError`), `tauri` v2 (command macro + `State` only; no window/context yet — that is Phase 1b).

**Prerequisite:** Phase 0 plan (`docs/superpowers/plans/2026-06-27-hr-kanban-phase0-foundation.md`) is implemented: `domain` (UnitConfig/Calendar/workload/types) and `db` (full schema migration, `connect`, `with_write_tx`, `ResourcesRepo`, `AllocationsRepo`) exist and `cargo test --workspace` passes.

**Scope note:** This is the **backend half** of roadmap Phase 1. The **frontend** (Vite+Vue+Pinio setup, Kanban view, CRUD UIs) is **Phase 1b**, the next plan. Calendar-table repos (`holiday`/`time_off`/`work_week_template`), `resource_project_rates`, and the `workload_cache` materialization belong to Phase 2 (allocations/workload UI) and are intentionally not here. `#[tauri::command]` functions are written and compile-checked; the `tauri::Builder`/window wiring lands in Phase 1b.

**Reference design:** `docs/design/2026-06-27-hr-kanban-design.md` (§2.5/§6.4 error model, §3.3 schema, §6 service/command split).

---

## File Structure

```
kanban/
├── Cargo.toml                       # add member "crates/app"
├── crates/
│   ├── domain/src/
│   │   ├── error.rs                 # NEW: DomainError (thiserror, no serde)
│   │   └── lib.rs                   # MOD: pub mod error; re-export
│   ├── db/
│   │   ├── Cargo.toml               # MOD: add serde
│   │   ├── src/models.rs            # MOD: derive Serialize; add Project/Task/Skill/Tag/Team/...
│   │   └── src/repo/
│   │       ├── mod.rs               # MOD: declare new repos
│   │       ├── projects.rs          # NEW
│   │       ├── skills.rs            # NEW
│   │       ├── tags.rs              # NEW
│   │       ├── tasks.rs             # NEW (tasks + skill_reqs + tags + deps)
│   │       └── teams.rs             # NEW (teams + members + overrides)
│   └── app/                         # NEW crate
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── state.rs             # AppState { pool }
│           ├── error.rs             # AppError {code,detail} + From<DomainError/DbError>
│           ├── service/
│           │   ├── mod.rs
│           │   ├── projects.rs
│           │   ├── catalog.rs       # skills + tags
│           │   ├── tasks.rs
│           │   └── teams.rs
│           └── command.rs           # #[tauri::command] wrappers
```

**Responsibilities:** `db` gains row models (Serialize so commands can return them) and one repo module per entity family. `app` is the only place that knows about `tauri` and `serde`-on-errors; services are pure async functions over `&SqlitePool` and are what tests exercise. `domain::DomainError` is the shared domain error vocabulary.

---

## Task 1: `app` crate + error model + `DomainError` (TDD)

**Files:**
- Create: `crates/domain/src/error.rs`
- Modify: `crates/domain/src/lib.rs`
- Create: `crates/app/Cargo.toml`
- Create: `crates/app/src/lib.rs`
- Create: `crates/app/src/state.rs`
- Create: `crates/app/src/error.rs`
- Modify: `Cargo.toml` (workspace members)
- Modify: `crates/db/Cargo.toml` (serde)

- [ ] **Step 1: Add `DomainError` to domain (zero serde) — `crates/domain/src/error.rs`**

```rust
use thiserror::Error;

/// Domain errors. Domain crate has NO serde dependency (design §2.5/§6.4).
/// Variations beyond Phase 1 (SkillMismatch, Solver, InsufficientCapacity) are
/// defined now so later phases don't reshape the enum.
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid ratio {0}")]
    InvalidRatio(f64),
    #[error("invalid date window")]
    InvalidDateWindow,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("dependency cycle detected involving task {0}")]
    DependencyCycle(i64),
    #[error("insufficient capacity for resource {resource_id}: shortfall {shortfall_pd} PD")]
    InsufficientCapacity { resource_id: i64, shortfall_pd: f64 },
    #[error("skill mismatch on task {task_id}: missing skill {skill_id}")]
    SkillMismatch { task_id: i64, skill_id: i64 },
    #[error("solver error: {0}")]
    Solver(String),
}
```

- [ ] **Step 2: Export it — `crates/domain/src/lib.rs`**

```rust
pub mod calendar;
pub mod error;
pub mod types;
pub mod unit;
pub mod workload;

pub use calendar::Calendar;
pub use error::DomainError;
pub use types::{Allocation, DayFraction, Window};
pub use unit::UnitConfig;
pub use workload::{alloc_pd, capacity_pd, count_calendar_days, overlap, workload_pd, utilization, team_utilization};
```

- [ ] **Step 3: Add serde to `db` and create `app` crate — `crates/app/Cargo.toml`**

```toml
[package]
name = "app"
version = "0.1.0"
edition.workspace = true

[dependencies]
domain = { path = "../domain" }
db = { path = "../db" }
tokio = { workspace = true }
serde = { version = "1", features = ["derive"] }
thiserror = { workspace = true }
tauri = { version = "2", default-features = false, features = ["wry"] }

[dev-dependencies]
tokio = { workspace = true }
```

> `tauri` is pulled in only for the `#[tauri::command]` macro and `tauri::State`. We do **not** call `tauri::Builder`/`generate_context` here (that is Phase 1b), so no `tauri.conf.json` is needed and the crate compiles headlessly.

- [ ] **Step 4: Modify `crates/db/Cargo.toml` — add serde**

Under `[dependencies]` add:
```toml
serde = { version = "1", features = ["derive"] }
```

- [ ] **Step 5: Add serde + new models to `crates/db/src/models.rs`**

Replace the file contents with the Phase 0 models (now `Serialize`) plus new entities:

```rust
use chrono::NaiveDate;
use sqlx::FromRow;

fn _ensure_serde_in_scope() -> impl serde::Serialize { () }

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Resource {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub available_from: Option<NaiveDate>,
    pub available_to: Option<NaiveDate>,
    pub status: String,
    pub daily_capacity_pd: f64,
    pub daily_rate_pd: Option<f64>,
    pub max_parallel_tasks_per_day: Option<i64>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct AllocationRow {
    pub id: i64,
    pub resource_id: i64,
    pub task_id: i64,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub percent: f64,
    pub status: String,
    pub source: String,
    pub run_id: Option<i64>,
}

impl AllocationRow {
    pub fn to_domain(&self, project_id: i64) -> domain::Allocation {
        domain::Allocation {
            id: self.id,
            resource_id: self.resource_id,
            project_id,
            start: self.start_date,
            end: self.end_date,
            percent: self.percent,
        }
    }
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub priority: i64,
    pub budget_pd: f64,
    pub max_parallel_tasks_per_day: Option<i64>,
    pub status: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Task {
    pub id: i64,
    pub project_id: i64,
    pub parent_task_id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub estimate_pd: f64,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub is_long_term: i64,
    pub segment_kind: Option<String>,
    pub status: String,
    pub sort_order: i64,
}

/// Kanban-shaped task row: task + assigned resource name (first active allocation).
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct KanbanTask {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub status: String,
    pub sort_order: i64,
    pub estimate_pd: f64,
    pub assignee: Option<String>,
    pub skill_count: i64,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Skill {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Team {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct TeamMember {
    pub team_id: i64,
    pub resource_id: i64,
    pub role: Option<String>,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct TeamOverride {
    pub team_id: i64,
    pub pd_hours: Option<f64>,
    pub pm_workdays: Option<f64>,
    pub overload_threshold: Option<f64>,
    pub underload_threshold: Option<f64>,
    pub utilization_green: Option<f64>,
    pub utilization_yellow: Option<f64>,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct TaskSkillRequirement {
    pub task_id: i64,
    pub skill_id: i64,
    pub min_proficiency: i64,
    pub is_mandatory: i64,
    pub weight: f64,
}
```

- [ ] **Step 6: Create `crates/app/src/lib.rs`**

```rust
pub mod command;
pub mod error;
pub mod service;
pub mod state;

pub use state::AppState;
```

- [ ] **Step 7: Create `crates/app/src/state.rs`**

```rust
use sqlx::SqlitePool;

/// Shared Tauri state. Holds the single SQLite pool for the local app.
pub struct AppState {
    pub pool: SqlitePool,
}
```

- [ ] **Step 8: Create `crates/app/src/error.rs` (AppError + mappers)**

```rust
use serde::Serialize;

/// IPC-level error. Serializes to `{ "code": "...", "detail": "..." }` (design §6.4).
/// DomainError (no serde) is mapped here, never embedded via #[from].
#[derive(Debug, Serialize)]
pub struct AppError {
    pub code: &'static str,
    pub detail: String,
}

impl AppError {
    pub const fn validation(detail: String) -> Self { Self { code: "VALIDATION", detail } }
    pub const fn not_found(detail: String) -> Self { Self { code: "NOT_FOUND", detail } }
    pub const fn domain(detail: String) -> Self { Self { code: "DOMAIN", detail } }
    pub const fn db(detail: String) -> Self { Self { code: "DB", detail } }
    pub const fn internal(detail: String) -> Self { Self { code: "INTERNAL", detail } }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.detail)
    }
}
impl std::error::Error for AppError {}

/// Explicit DomainError -> AppError mapping (no #[from] embedding; design §6.4).
impl From<domain::DomainError> for AppError {
    fn from(e: domain::DomainError) -> Self {
        use domain::DomainError::*;
        match e {
            InvalidRatio(_) | InvalidDateWindow => AppError::validation(e.to_string()),
            NotFound(s) => AppError::not_found(s),
            DependencyCycle(_) | InsufficientCapacity { .. } | SkillMismatch { .. } | Solver(_) => {
                AppError::domain(e.to_string())
            }
        }
    }
}

impl From<db::DbError> for AppError {
    fn from(e: db::DbError) -> Self {
        match e {
            db::DbError::NotFound => AppError::not_found("entity".into()),
            other => AppError::db(other.to_string()),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self { AppError::db(e.to_string()) }
}
```

- [ ] **Step 9: Create stub `crates/app/src/service/mod.rs` and `crates/app/src/command.rs`**

`crates/app/src/service/mod.rs`:
```rust
pub mod catalog;
pub mod projects;
pub mod tasks;
pub mod teams;
```

`crates/app/src/command.rs` (stub, filled across tasks):
```rust
use crate::error::AppError;
use crate::state::AppState;

// Commands are added in Tasks 2–6. Each is #[tauri::command] delegating to a service.
fn _unused(_s: tauri::State<'_, AppState>) -> Result<(), AppError> { Ok(()) }
```

- [ ] **Step 10: Add `app` to the workspace — modify root `Cargo.toml`**

Change `members` to:
```toml
members = ["crates/domain", "crates/db", "crates/app"]
```

- [ ] **Step 11: Write the error-mapping test — `crates/app/tests/error_mapping.rs`**

```rust
use app::error::AppError;
use domain::DomainError;

#[test]
fn invalid_ratio_maps_to_validation() {
    let e: AppError = DomainError::InvalidRatio(-1.0).into();
    assert_eq!(e.code, "VALIDATION");
    assert!(e.detail.contains("invalid ratio"));
}

#[test]
fn dependency_cycle_maps_to_domain() {
    let e: AppError = DomainError::DependencyCycle(7).into();
    assert_eq!(e.code, "DOMAIN");
}

#[test]
fn not_found_maps_to_not_found() {
    let e: AppError = DomainError::NotFound("task 5".into()).into();
    assert_eq!(e.code, "NOT_FOUND");
    assert_eq!(e.detail, "task 5");
}

#[test]
fn app_error_serializes_to_code_detail() {
    let e = AppError::domain("boom".into());
    let json = serde_json::to_string(&e).unwrap();
    assert!(json.contains("\"code\":\"DOMAIN\""));
    assert!(json.contains("\"detail\":\"boom\""));
}
```

Add `serde_json` to `app` dev-dependencies: in `crates/app/Cargo.toml` under `[dev-dependencies]` add `serde_json = "1"`.

- [ ] **Step 12: Run test — verify PASS**

Run: `cargo test -p app --test error_mapping`
Expected: `4 passed`.

- [ ] **Step 13: Commit**

```bash
git add -A
git commit -m "feat(app): crate + AppError{code,detail} + DomainError mapping"
```

---

## Task 2: Projects — repo + service + command (TDD)

**Files:**
- Create: `crates/db/src/repo/projects.rs`
- Create: `crates/app/src/service/projects.rs`
- Modify: `crates/db/src/repo/mod.rs`
- Modify: `crates/app/src/command.rs`
- Create: `crates/app/tests/projects.rs`

- [ ] **Step 1: Write `crates/db/src/repo/projects.rs`**

```rust
use crate::error::DbError;
use crate::models::Project;
use sqlx::SqlitePool;

pub struct ProjectsRepo;

impl ProjectsRepo {
    pub async fn create(
        pool: &SqlitePool, name: &str, description: Option<&str>,
        start: Option<&str>, end: Option<&str>, priority: i64, budget_pd: f64,
    ) -> Result<i64, DbError> {
        let (id,): (i64,) = sqlx::query_as(
            "INSERT INTO projects (name, description, start_date, end_date, priority, budget_pd) \
             VALUES (?,?,?,?,?,?) RETURNING id")
            .bind(name).bind(description).bind(start).bind(end).bind(priority).bind(budget_pd)
            .fetch_one(pool).await?;
        Ok(id)
    }

    pub async fn list_active(pool: &SqlitePool) -> Result<Vec<Project>, DbError> {
        let rows = sqlx::query_as::<_, Project>(
            "SELECT id, name, description, start_date, end_date, priority, budget_pd, \
                    max_parallel_tasks_per_day, status \
             FROM projects WHERE deleted_at IS NULL ORDER BY priority, name")
            .fetch_all(pool).await?;
        Ok(rows)
    }

    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Project, DbError> {
        sqlx::query_as::<_, Project>(
            "SELECT id, name, description, start_date, end_date, priority, budget_pd, \
                    max_parallel_tasks_per_day, status \
             FROM projects WHERE id = ? AND deleted_at IS NULL")
            .bind(id).fetch_optional(pool).await?.ok_or(DbError::NotFound)
    }

    pub async fn set_status(pool: &SqlitePool, id: i64, status: &str) -> Result<(), DbError> {
        let n = sqlx::query("UPDATE projects SET status = ? WHERE id = ? AND deleted_at IS NULL")
            .bind(status).bind(id).execute(pool).await?.rows_affected();
        if n == 0 { return Err(DbError::NotFound); }
        Ok(())
    }

    pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
        let n = sqlx::query(
            "UPDATE projects SET deleted_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') \
             WHERE id = ? AND deleted_at IS NULL")
            .bind(id).execute(pool).await?.rows_affected();
        if n == 0 { return Err(DbError::NotFound); }
        Ok(())
    }
}
```

- [ ] **Step 2: Register repo — `crates/db/src/repo/mod.rs`**

```rust
pub mod allocations;
pub mod projects;
pub mod resources;
pub use allocations::AllocationsRepo;
pub use projects::ProjectsRepo;
pub use resources::ResourcesRepo;
```

- [ ] **Step 3: Write `crates/app/src/service/projects.rs`**

```rust
use crate::error::AppError;
use db::models::Project;
use db::ProjectsRepo;
use sqlx::SqlitePool;

pub struct ProjectsService;

impl ProjectsService {
    pub async fn create(
        pool: &SqlitePool, name: &str, description: Option<&str>,
        start: Option<&str>, end: Option<&str>, priority: i64, budget_pd: f64,
    ) -> Result<i64, AppError> {
        validate_priority(priority)?;
        if let (Some(s), Some(e)) = (start, end) {
            if e < s { return Err(domain::DomainError::InvalidDateWindow.into()); }
        }
        Ok(ProjectsRepo::create(pool, name, description, start, end, priority, budget_pd).await?)
    }

    pub async fn list(pool: &SqlitePool) -> Result<Vec<Project>, AppError> {
        Ok(ProjectsRepo::list_active(pool).await?)
    }

    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Project, AppError> {
        Ok(ProjectsRepo::get(pool, id).await?)
    }
}

fn validate_priority(p: i64) -> Result<(), AppError> {
    if !(1..=9).contains(&p) {
        return Err(domain::DomainError::InvalidRatio(p as f64).into());
    }
    Ok(())
}
```

- [ ] **Step 4: Add the command — modify `crates/app/src/command.rs`**

Replace the stub with:

```rust
use crate::error::AppError;
use crate::service::{catalog, projects, tasks, teams};
use crate::state::AppState;
use db::models::{Project, Skill, Tag, Task, Team, KanbanTask};

#[tauri::command]
pub async fn create_project(
    state: tauri::State<'_, AppState>,
    name: String, description: Option<String>,
    start: Option<String>, end: Option<String>,
    priority: i64, budget_pd: Option<f64>,
) -> Result<i64, AppError> {
    projects::ProjectsService::create(
        &state.pool, &name, description.as_deref(),
        start.as_deref(), end.as_deref(), priority, budget_pd.unwrap_or(0.0)).await
}

#[tauri::command]
pub async fn list_projects(state: tauri::State<'_, AppState>) -> Result<Vec<Project>, AppError> {
    projects::ProjectsService::list(&state.pool).await
}
```

(The remaining commands reference `catalog`, `tasks`, `teams`, `KanbanTask` — created in later tasks. Until then `cargo build -p app` will fail on unused imports; that's expected and resolves by Task 7.)

- [ ] **Step 5: Write the failing test — `crates/app/tests/projects.rs`**

```rust
use app::service::projects::ProjectsService;
use db::pool::connect;

#[tokio::test]
async fn create_list_get_project() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();

    let id = ProjectsService::create(&pool, "Atlas", Some("desc"), Some("2026-06-01"), Some("2026-07-01"), 3, 40.0).await.unwrap();
    let list = ProjectsService::list(&pool).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "Atlas");
    let got = ProjectsService::get(&pool, id).await.unwrap();
    assert_eq!(got.priority, 3);
    assert!((got.budget_pd - 40.0).abs() < 1e-9);
}

#[tokio::test]
async fn invalid_priority_rejected() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let err = ProjectsService::create(&pool, "X", None, None, None, 99, 0.0).await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}

#[tokio::test]
async fn bad_date_window_rejected() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let err = ProjectsService::create(&pool, "X", None, Some("2026-07-01"), Some("2026-06-01"), 5, 0.0).await.unwrap_err();
    assert_eq!(err.code, "VALIDATION"); // InvalidDateWindow -> VALIDATION
}
```

> Test migration path is `../db/migrations` relative to `crates/app/tests/`.

- [ ] **Step 6: Run test — verify PASS**

Run: `cargo test -p app --test projects`
Expected: `3 passed`.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(app): Projects repo+service+command"
```

---

## Task 3: Skills + Tags — repos + service + commands (TDD)

**Files:**
- Create: `crates/db/src/repo/skills.rs`
- Create: `crates/db/src/repo/tags.rs`
- Modify: `crates/db/src/repo/mod.rs`
- Create: `crates/app/src/service/catalog.rs`
- Create: `crates/app/tests/catalog.rs`

- [ ] **Step 1: Write `crates/db/src/repo/skills.rs`**

```rust
use crate::error::DbError;
use crate::models::Skill;
use sqlx::SqlitePool;

pub struct SkillsRepo;

impl SkillsRepo {
    /// Upsert by name; returns the skill id. Used for catalog management + LLM normalization.
    pub async fn ensure(pool: &SqlitePool, name: &str) -> Result<i64, DbError> {
        let (id,): (i64,) = sqlx::query_as(
            "INSERT INTO skills (name) VALUES (?) \
             ON CONFLICT(name) DO UPDATE SET name=excluded.name \
             RETURNING id")
            .bind(name).fetch_one(pool).await?;
        Ok(id)
    }

    pub async fn list(pool: &SqlitePool) -> Result<Vec<Skill>, DbError> {
        Ok(sqlx::query_as::<_, Skill>("SELECT id, name FROM skills ORDER BY name").fetch_all(pool).await?)
    }
}
```

- [ ] **Step 2: Write `crates/db/src/repo/tags.rs`**

```rust
use crate::error::DbError;
use crate::models::Tag;
use sqlx::SqlitePool;

pub struct TagsRepo;

impl TagsRepo {
    pub async fn ensure(pool: &SqlitePool, name: &str, color: Option<&str>) -> Result<i64, DbError> {
        let (id,): (i64,) = sqlx::query_as(
            "INSERT INTO tags (name, color) VALUES (?, ?) \
             ON CONFLICT(name) DO UPDATE SET color = COALESCE(excluded.color, tags.color) \
             RETURNING id")
            .bind(name).bind(color).fetch_one(pool).await?;
        Ok(id)
    }

    pub async fn list(pool: &SqlitePool) -> Result<Vec<Tag>, DbError> {
        Ok(sqlx::query_as::<_, Tag>("SELECT id, name, color FROM tags ORDER BY name").fetch_all(pool).await?)
    }
}
```

- [ ] **Step 3: Register repos — `crates/db/src/repo/mod.rs`**

```rust
pub mod allocations;
pub mod projects;
pub mod resources;
pub mod skills;
pub mod tags;
pub use allocations::AllocationsRepo;
pub use projects::ProjectsRepo;
pub use resources::ResourcesRepo;
pub use skills::SkillsRepo;
pub use tags::TagsRepo;
```

- [ ] **Step 4: Write `crates/app/src/service/catalog.rs`**

```rust
use crate::error::AppError;
use db::models::{Skill, Tag};
use db::{SkillsRepo, TagsRepo};
use sqlx::SqlitePool;

pub struct CatalogService;

impl CatalogService {
    pub async fn ensure_skill(pool: &SqlitePool, name: &str) -> Result<i64, AppError> {
        Ok(SkillsRepo::ensure(pool, name).await?)
    }
    pub async fn list_skills(pool: &SqlitePool) -> Result<Vec<Skill>, AppError> {
        Ok(SkillsRepo::list(pool).await?)
    }
    pub async fn ensure_tag(pool: &SqlitePool, name: &str, color: Option<&str>) -> Result<i64, AppError> {
        Ok(TagsRepo::ensure(pool, name, color).await?)
    }
    pub async fn list_tags(pool: &SqlitePool) -> Result<Vec<Tag>, AppError> {
        Ok(TagsRepo::list(pool).await?)
    }
}
```

- [ ] **Step 5: Add commands — append to `crates/app/src/command.rs`**

```rust
#[tauri::command]
pub async fn ensure_skill(state: tauri::State<'_, AppState>, name: String) -> Result<i64, AppError> {
    catalog::CatalogService::ensure_skill(&state.pool, &name).await
}
#[tauri::command]
pub async fn list_skills(state: tauri::State<'_, AppState>) -> Result<Vec<Skill>, AppError> {
    catalog::CatalogService::list_skills(&state.pool).await
}
#[tauri::command]
pub async fn ensure_tag(state: tauri::State<'_, AppState>, name: String, color: Option<String>) -> Result<i64, AppError> {
    catalog::CatalogService::ensure_tag(&state.pool, &name, color.as_deref()).await
}
#[tauri::command]
pub async fn list_tags(state: tauri::State<'_, AppState>) -> Result<Vec<Tag>, AppError> {
    catalog::CatalogService::list_tags(&state.pool).await
}
```

- [ ] **Step 6: Write the failing test — `crates/app/tests/catalog.rs`**

```rust
use app::service::catalog::CatalogService;
use db::pool::connect;

#[tokio::test]
async fn ensure_is_idempotent() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let id1 = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    let id2 = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    assert_eq!(id1, id2);
    assert_eq!(CatalogService::list_skills(&pool).await.unwrap().len(), 1);
}

#[tokio::test]
async fn tag_with_color() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let _ = CatalogService::ensure_tag(&pool, "urgent", Some("#f00")).await.unwrap();
    let tags = CatalogService::list_tags(&pool).await.unwrap();
    assert_eq!(tags[0].color.as_deref(), Some("#f00"));
}
```

- [ ] **Step 7: Run test — verify PASS**

Run: `cargo test -p app --test catalog`
Expected: `2 passed`.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat(app): Skills/Tags repo+service+command (idempotent ensure)"
```

---

## Task 4: Tasks — repo (with skill-reqs + tags in one tx) + service + command (TDD)

Tasks carry their skill requirements and tags. Creating a task writes the task row, its `task_skill_requirements`, and its `task_tags` in a single `with_write_tx` so partial writes can't leak.

**Files:**
- Create: `crates/db/src/repo/tasks.rs`
- Modify: `crates/db/src/repo/mod.rs`
- Create: `crates/app/src/service/tasks.rs`
- Create: `crates/app/tests/tasks.rs`

- [ ] **Step 1: Write `crates/db/src/repo/tasks.rs`**

```rust
use crate::error::DbError;
use crate::models::{KanbanTask, Task, TaskSkillRequirement};
use sqlx::SqlitePool;

pub struct TasksRepo;

/// Input for creating a task with its skill requirements and tags (design §3.3.11/13/14).
pub struct TaskCreate<'a> {
    pub project_id: i64,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub estimate_pd: f64,
    pub start: Option<&'a str>,
    pub end: Option<&'a str>,
    pub is_long_term: bool,
    pub sort_order: i64,
    pub skill_reqs: &'a [(i64 /*skill_id*/, i64 /*min_prof*/, bool /*mandatory*/, f64 /*weight*/)],
    pub tag_ids: &'a [i64],
}

impl TasksRepo {
    /// Atomic create: task + skill requirements + tags in one transaction.
    pub async fn create(pool: &SqlitePool, input: TaskCreate<'_>) -> Result<i64, DbError> {
        db::tx::with_write_tx(pool, |tx| Box::pin(async move {
            let (id,): (i64,) = sqlx::query_as(
                "INSERT INTO tasks (project_id, title, description, estimate_pd, start_date, end_date, \
                 is_long_term, sort_order) VALUES (?,?,?,?,?,?,?,?) RETURNING id")
                .bind(input.project_id).bind(input.title).bind(input.description)
                .bind(input.estimate_pd).bind(input.start).bind(input.end)
                .bind(input.is_long_term as i64).bind(input.sort_order)
                .fetch_one(&mut *tx).await?;
            for &(skill_id, min_prof, mandatory, weight) in input.skill_reqs {
                sqlx::query(
                    "INSERT INTO task_skill_requirements (task_id, skill_id, min_proficiency, is_mandatory, weight) \
                     VALUES (?,?,?,?,?)")
                    .bind(id).bind(skill_id).bind(min_prof).bind(mandatory as i64).bind(weight)
                    .execute(&mut *tx).await?;
            }
            for &tag_id in input.tag_ids {
                sqlx::query("INSERT INTO task_tags (task_id, tag_id) VALUES (?,?)")
                    .bind(id).bind(tag_id).execute(&mut *tx).await?;
            }
            Ok(id)
        })).await
    }

    pub async fn list_by_project(pool: &SqlitePool, project_id: i64) -> Result<Vec<Task>, DbError> {
        Ok(sqlx::query_as::<_, Task>(
            "SELECT id, project_id, parent_task_id, title, description, estimate_pd, start_date, end_date, \
                    is_long_term, segment_kind, status, sort_order \
             FROM tasks WHERE project_id = ? AND deleted_at IS NULL ORDER BY sort_order, id")
            .bind(project_id).fetch_all(pool).await?)
    }

    pub async fn set_status(pool: &SqlitePool, id: i64, status: &str) -> Result<(), DbError> {
        let n = sqlx::query("UPDATE tasks SET status = ? WHERE id = ? AND deleted_at IS NULL")
            .bind(status).bind(id).execute(pool).await?.rows_affected();
        if n == 0 { return Err(DbError::NotFound); }
        Ok(())
    }

    pub async fn list_skill_reqs(pool: &SqlitePool, task_id: i64) -> Result<Vec<TaskSkillRequirement>, DbError> {
        Ok(sqlx::query_as::<_, TaskSkillRequirement>(
            "SELECT task_id, skill_id, min_proficiency, is_mandatory, weight \
             FROM task_skill_requirements WHERE task_id = ?")
            .bind(task_id).fetch_all(pool).await?)
    }

    /// Kanban-shaped read: task + first assignee name + skill count (design §7 Kanban card).
    pub async fn list_kanban(pool: &SqlitePool, project_id: i64) -> Result<Vec<KanbanTask>, DbError> {
        Ok(sqlx::query_as::<_, KanbanTask>(
            "SELECT t.id, t.project_id, t.title, t.status, t.sort_order, t.estimate_pd, \
                    (SELECT r.name FROM allocations a JOIN resources r ON r.id = a.resource_id \
                     WHERE a.task_id = t.id AND a.deleted_at IS NULL LIMIT 1) AS assignee, \
                    (SELECT count(*) FROM task_skill_requirements sr WHERE sr.task_id = t.id) AS skill_count \
             FROM tasks t WHERE t.project_id = ? AND t.deleted_at IS NULL \
             ORDER BY t.sort_order, t.id")
            .bind(project_id).fetch_all(pool).await?)
    }
}
```

> `with_write_tx` (Phase 0) takes a `&mut Transaction` callback returning `Pin<Box<Future<Output = Result<T, DbError>>>>`; the closure uses `&mut *tx` (reborrow as `Executor`) and returns `Ok(id)` — `with_write_tx` owns commit/rollback.

- [ ] **Step 2: Register repo — `crates/db/src/repo/mod.rs`**

```rust
pub mod allocations;
pub mod projects;
pub mod resources;
pub mod skills;
pub mod tags;
pub mod tasks;
pub use allocations::AllocationsRepo;
pub use projects::ProjectsRepo;
pub use resources::ResourcesRepo;
pub use skills::SkillsRepo;
pub use tags::TagsRepo;
pub use tasks::{TaskCreate, TasksRepo};
```

- [ ] **Step 3: Write `crates/app/src/service/tasks.rs`**

```rust
use crate::error::AppError;
use db::models::{KanbanTask, Task, TaskSkillRequirement};
use db::repo::tasks::TaskCreate;
use db::TasksRepo;
use sqlx::SqlitePool;

pub struct TasksService;

impl TasksService {
    pub async fn create(
        pool: &SqlitePool,
        project_id: i64, title: &str, description: Option<&str>,
        estimate_pd: f64, start: Option<&str>, end: Option<&str>,
        is_long_term: bool, sort_order: i64,
        skill_reqs: &[(i64, i64, bool, f64)], tag_ids: &[i64],
    ) -> Result<i64, AppError> {
        if estimate_pd < 0.0 { return Err(domain::DomainError::InvalidRatio(estimate_pd).into()); }
        if let (Some(s), Some(e)) = (start, end) {
            if e < s { return Err(domain::DomainError::InvalidDateWindow.into()); }
        }
        for &(_, min_prof, _, _) in skill_reqs {
            if !(1..=5).contains(&min_prof) {
                return Err(domain::DomainError::InvalidRatio(min_prof as f64).into());
            }
        }
        let input = TaskCreate {
            project_id, title, description, estimate_pd, start, end, is_long_term, sort_order,
            skill_reqs, tag_ids,
        };
        Ok(TasksRepo::create(pool, input).await?)
    }

    pub async fn list_by_project(pool: &SqlitePool, project_id: i64) -> Result<Vec<Task>, AppError> {
        Ok(TasksRepo::list_by_project(pool, project_id).await?)
    }

    pub async fn set_status(pool: &SqlitePool, id: i64, status: &str) -> Result<(), AppError> {
        match status {
            "todo" | "in_progress" | "blocked" | "review" | "done" | "cancelled" => {}
            _ => return Err(domain::DomainError::InvalidRatio(0.0).into()),
        }
        Ok(TasksRepo::set_status(pool, id, status).await?)
    }

    pub async fn skill_reqs(pool: &SqlitePool, task_id: i64) -> Result<Vec<TaskSkillRequirement>, AppError> {
        Ok(TasksRepo::list_skill_reqs(pool, task_id).await?)
    }

    pub async fn kanban(pool: &SqlitePool, project_id: i64) -> Result<Vec<KanbanTask>, AppError> {
        Ok(TasksRepo::list_kanban(pool, project_id).await?)
    }
}
```

- [ ] **Step 4: Add commands — append to `crates/app/src/command.rs`**

```rust
#[tauri::command]
pub async fn create_task(
    state: tauri::State<'_, AppState>,
    project_id: i64, title: String, description: Option<String>,
    estimate_pd: f64, start: Option<String>, end: Option<String>,
    is_long_term: bool, sort_order: i64,
    skill_reqs: Vec<(i64, i64, bool, f64)>, tag_ids: Vec<i64>,
) -> Result<i64, AppError> {
    tasks::TasksService::create(
        &state.pool, project_id, &title, description.as_deref(), estimate_pd,
        start.as_deref(), end.as_deref(), is_long_term, sort_order, &skill_reqs, &tag_ids).await
}

#[tauri::command]
pub async fn set_task_status(state: tauri::State<'_, AppState>, id: i64, status: String) -> Result<(), AppError> {
    tasks::TasksService::set_status(&state.pool, id, &status).await
}

#[tauri::command]
pub async fn kanban_tasks(state: tauri::State<'_, AppState>, project_id: i64) -> Result<Vec<KanbanTask>, AppError> {
    tasks::TasksService::kanban(&state.pool, project_id).await
}
```

- [ ] **Step 5: Write the failing test — `crates/app/tests/tasks.rs`**

```rust
use app::service::tasks::TasksService;
use app::service::projects::ProjectsService;
use app::service::catalog::CatalogService;
use db::pool::connect;

async fn fresh() -> sqlx::SqlitePool {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn create_task_with_skills_and_tags_atomic() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    let urgent = CatalogService::ensure_tag(&pool, "urgent", None).await.unwrap();

    let tid = TasksService::create(
        &pool, pid, "Build API", None, 5.0, Some("2026-06-01"), Some("2026-06-10"),
        false, 0, &[(rust, 3, true, 1.0)], &[urgent]).await.unwrap();

    let reqs = TasksService::skill_reqs(&pool, tid).await.unwrap();
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].skill_id, rust);
    assert_eq!(reqs[0].min_proficiency, 3);

    let kb = TasksService::kanban(&pool, pid).await.unwrap();
    assert_eq!(kb.len(), 1);
    assert_eq!(kb[0].title, "Build API");
    assert_eq!(kb[0].skill_count, 1);
    assert_eq!(kb[0].assignee, None); // no allocation yet
}

#[tokio::test]
async fn invalid_proficiency_rejected() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let err = TasksService::create(
        &pool, pid, "T", None, 1.0, None, None, false, 0, &[(1, 9, true, 1.0)], &[]).await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}

#[tokio::test]
async fn set_status_transitions() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let tid = TasksService::create(&pool, pid, "T", None, 1.0, None, None, false, 0, &[], &[]).await.unwrap();
    TasksService::set_status(&pool, tid, "in_progress").await.unwrap();
    let kb = TasksService::kanban(&pool, pid).await.unwrap();
    assert_eq!(kb[0].status, "in_progress");
    let err = TasksService::set_status(&pool, tid, "bogus").await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}
```

- [ ] **Step 6: Run test — verify PASS**

Run: `cargo test -p app --test tasks`
Expected: `3 passed`.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(app): Tasks repo+service+command (atomic skills/tags, kanban read)"
```

---

## Task 5: Task dependencies + cycle guard (TDD)

Dependencies form a DAG; adding an edge that would create a cycle must fail with `DomainError::DependencyCycle` (mapped to `DOMAIN`). Cycle detection is a DFS over existing edges within the service (design §3.3.12 note: app-layer cycle check).

**Files:**
- Modify: `crates/db/src/repo/tasks.rs` (add `TaskDepsRepo` block)
- Modify: `crates/app/src/service/tasks.rs` (add `add_dependency`)
- Create: `crates/app/tests/deps.rs`

- [ ] **Step 1: Append `TaskDepsRepo` to `crates/db/src/repo/tasks.rs`**

```rust
pub struct TaskDepsRepo;

impl TaskDepsRepo {
    pub async fn add(pool: &SqlitePool, task_id: i64, predecessor_id: i64, lag_days: i64) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO task_dependencies (task_id, predecessor_id, lag_days) VALUES (?,?,?) \
             ON CONFLICT(task_id, predecessor_id) DO UPDATE SET lag_days = excluded.lag_days")
            .bind(task_id).bind(predecessor_id).bind(lag_days)
            .execute(pool).await?;
        Ok(())
    }

    /// All (task_id, predecessor_id) edges, for in-memory cycle detection.
    pub async fn all_edges(pool: &SqlitePool) -> Result<Vec<(i64, i64)>, DbError> {
        Ok(sqlx::query_as("SELECT task_id, predecessor_id FROM task_dependencies")
            .fetch_all(pool).await?)
    }

    /// Direct predecessors of a task (for the Kanban/Gantt dependency display).
    pub async fn predecessors(pool: &SqlitePool, task_id: i64) -> Result<Vec<i64>, DbError> {
        let rows: Vec<(i64,)> = sqlx::query_as(
            "SELECT predecessor_id FROM task_dependencies WHERE task_id = ?")
            .bind(task_id).fetch_all(pool).await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }
}
```

- [ ] **Step 2: Export `TaskDepsRepo` — `crates/db/src/repo/mod.rs`**

Change the tasks re-export line to:
```rust
pub use tasks::{TaskCreate, TaskDepsRepo, TasksRepo};
```

- [ ] **Step 3: Append dependency service to `crates/app/src/service/tasks.rs`**

```rust
use db::TaskDepsRepo;

impl TasksService {
    /// Add a dependency edge after checking it creates no cycle (design §3.3.12).
    pub async fn add_dependency(
        pool: &SqlitePool, task_id: i64, predecessor_id: i64, lag_days: i64,
    ) -> Result<(), AppError> {
        if task_id == predecessor_id {
            return Err(domain::DomainError::InvalidRatio(0.0).into()); // self-dep invalid
        }
        // Tentative new edge set: existing edges + the proposed one.
        let mut edges = TaskDepsRepo::all_edges(pool).await?;
        edges.push((task_id, predecessor_id));
        if has_cycle(&edges) {
            return Err(domain::DomainError::DependencyCycle(task_id).into());
        }
        TaskDepsRepo::add(pool, task_id, predecessor_id, lag_days).await?;
        Ok(())
    }
}

/// Edge direction: task depends on predecessor (task must come after predecessor).
/// A cycle in this graph means impossible ordering.
fn has_cycle(edges: &[(i64, i64)]) -> bool {
    use std::collections::{HashMap, HashSet};
    let mut adj: HashMap<i64, Vec<i64>> = HashMap::new();
    for &(t, p) in edges { adj.entry(p).or_default().push(t); } // p -> t
    let nodes: HashSet<i64> = edges.iter().flat_map(|(t, p)| [*t, *p]).collect();
    let mut white = nodes.clone();
    while let Some(&start) = white.iter().next() {
        let mut stack = vec![start];
        let mut on_path = HashSet::new();
        while let Some(&n) = stack.last() {
            if !white.contains(&n) { stack.pop(); continue; }
            white.remove(&n);
            on_path.insert(n);
            let neighbors = adj.get(&n).cloned().unwrap_or_default();
            let mut descended = false;
            for nb in neighbors {
                if on_path.contains(&nb) { return true; } // back edge -> cycle
                if white.contains(&nb) { stack.push(nb); descended = true; break; }
            }
            if !descended { on_path.remove(&n); stack.pop(); }
        }
    }
    false
}

#[cfg(test)]
mod cycle_tests {
    use super::has_cycle;
    #[test]
    fn acyclic_ok() { assert!(!has_cycle(&[(1, 2), (2, 3)])); }
    #[test]
    fn cycle_detected() { assert!(has_cycle(&[(1, 2), (2, 3), (3, 1)])); }
}
```

- [ ] **Step 4: Write the failing test — `crates/app/tests/deps.rs`**

```rust
use app::service::tasks::TasksService;
use app::service::projects::ProjectsService;
use db::pool::connect;

#[tokio::test]
async fn dependency_then_cycle_blocked() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let a = TasksService::create(&pool, pid, "A", None, 1.0, None, None, false, 0, &[], &[]).await.unwrap();
    let b = TasksService::create(&pool, pid, "B", None, 1.0, None, None, false, 1, &[], &[]).await.unwrap();
    let c = TasksService::create(&pool, pid, "C", None, 1.0, None, None, false, 2, &[], &[]).await.unwrap();

    // B depends on A; C depends on B  (A -> B -> C)
    TasksService::add_dependency(&pool, b, a, 0).await.unwrap();
    TasksService::add_dependency(&pool, c, b, 0).await.unwrap();

    // A depending on C would close the cycle A->B->C->A -> must be rejected
    let err = TasksService::add_dependency(&pool, a, c, 0).await.unwrap_err();
    assert_eq!(err.code, "DOMAIN");
    assert!(err.detail.contains("cycle"));
}

#[tokio::test]
async fn self_dependency_rejected() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let a = TasksService::create(&pool, pid, "A", None, 1.0, None, None, false, 0, &[], &[]).await.unwrap();
    let err = TasksService::add_dependency(&pool, a, a, 0).await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}
```

- [ ] **Step 5: Run test — verify PASS**

Run: `cargo test -p app --test deps`
Expected: `2 passed` (plus the 2 inline `cycle_tests`).

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(app): task dependencies with DAG cycle guard"
```

---

## Task 6: Teams + members + overrides — repo + service + command (TDD)

**Files:**
- Create: `crates/db/src/repo/teams.rs`
- Modify: `crates/db/src/repo/mod.rs`
- Modify: `crates/app/src/service/teams.rs` (create)
- Create: `crates/app/tests/teams.rs`

- [ ] **Step 1: Write `crates/db/src/repo/teams.rs`**

```rust
use crate::error::DbError;
use crate::models::{Team, TeamMember, TeamOverride};
use sqlx::SqlitePool;

pub struct TeamsRepo;

impl TeamsRepo {
    pub async fn create(pool: &SqlitePool, name: &str, description: Option<&str>) -> Result<i64, DbError> {
        let (id,): (i64,) = sqlx::query_as(
            "INSERT INTO teams (name, description) VALUES (?,?) RETURNING id")
            .bind(name).bind(description).fetch_one(pool).await?;
        Ok(id)
    }
    pub async fn list_active(pool: &SqlitePool) -> Result<Vec<Team>, DbError> {
        Ok(sqlx::query_as::<_, Team>(
            "SELECT id, name, description FROM teams WHERE deleted_at IS NULL ORDER BY name")
            .fetch_all(pool).await?)
    }
}

pub struct TeamMembersRepo;
impl TeamMembersRepo {
    pub async fn add(pool: &SqlitePool, team_id: i64, resource_id: i64, role: Option<&str>) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO team_members (team_id, resource_id, role) VALUES (?,?,?) \
             ON CONFLICT(team_id, resource_id) DO UPDATE SET role = excluded.role")
            .bind(team_id).bind(resource_id).bind(role).execute(pool).await?;
        Ok(())
    }
    pub async fn list_members(pool: &SqlitePool, team_id: i64) -> Result<Vec<TeamMember>, DbError> {
        Ok(sqlx::query_as::<_, TeamMember>(
            "SELECT team_id, resource_id, role FROM team_members WHERE team_id = ?")
            .bind(team_id).fetch_all(pool).await?)
    }
    /// The (first) team a resource belongs to, for effective-constant resolution (design §3.3.8a).
    pub async fn team_of_resource(pool: &SqlitePool, resource_id: i64) -> Result<Option<i64>, DbError> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT team_id FROM team_members WHERE resource_id = ? \
             ORDER BY (role = 'lead') DESC, joined_at DESC LIMIT 1")
            .bind(resource_id).fetch_optional(pool).await?;
        Ok(row.map(|r| r.0))
    }
}

pub struct TeamOverridesRepo;
impl TeamOverridesRepo {
    pub async fn upsert(pool: &SqlitePool, o: &TeamOverride) -> Result<(), DbError> {
        sqlx::query(
            "INSERT INTO team_overrides (team_id, pd_hours, pm_workdays, overload_threshold, \
             underload_threshold, utilization_green, utilization_yellow) \
             VALUES (?,?,?,?,?,?,?) \
             ON CONFLICT(team_id) DO UPDATE SET \
             pd_hours=excluded.pd_hours, pm_workdays=excluded.pm_workdays, \
             overload_threshold=excluded.overload_threshold, \
             underload_threshold=excluded.underload_threshold, \
             utilization_green=excluded.utilization_green, \
             utilization_yellow=excluded.utilization_yellow")
            .bind(o.team_id).bind(o.pd_hours).bind(o.pm_workdays)
            .bind(o.overload_threshold).bind(o.underload_threshold)
            .bind(o.utilization_green).bind(o.utilization_yellow)
            .execute(pool).await?;
        Ok(())
    }
    pub async fn get(pool: &SqlitePool, team_id: i64) -> Result<Option<TeamOverride>, DbError> {
        Ok(sqlx::query_as::<_, TeamOverride>(
            "SELECT team_id, pd_hours, pm_workdays, overload_threshold, underload_threshold, \
             utilization_green, utilization_yellow FROM team_overrides WHERE team_id = ?")
            .bind(team_id).fetch_optional(pool).await?)
    }
}
```

- [ ] **Step 2: Register repos — `crates/db/src/repo/mod.rs`**

```rust
pub mod allocations;
pub mod projects;
pub mod resources;
pub mod skills;
pub mod tags;
pub mod tasks;
pub mod teams;
pub use allocations::AllocationsRepo;
pub use projects::ProjectsRepo;
pub use resources::ResourcesRepo;
pub use skills::SkillsRepo;
pub use tags::TagsRepo;
pub use tasks::{TaskCreate, TaskDepsRepo, TasksRepo};
pub use teams::{TeamMembersRepo, TeamOverridesRepo, TeamsRepo};
```

- [ ] **Step 3: Create `crates/app/src/service/teams.rs`**

```rust
use crate::error::AppError;
use db::models::{Team, TeamMember, TeamOverride};
use db::{TeamMembersRepo, TeamOverridesRepo, TeamsRepo};
use sqlx::SqlitePool;

pub struct TeamsService;

impl TeamsService {
    pub async fn create(pool: &SqlitePool, name: &str, description: Option<&str>) -> Result<i64, AppError> {
        Ok(TeamsRepo::create(pool, name, description).await?)
    }
    pub async fn list(pool: &SqlitePool) -> Result<Vec<Team>, AppError> {
        Ok(TeamsRepo::list_active(pool).await?)
    }
    pub async fn add_member(pool: &SqlitePool, team_id: i64, resource_id: i64, role: Option<&str>) -> Result<(), AppError> {
        Ok(TeamMembersRepo::add(pool, team_id, resource_id, role).await?)
    }
    pub async fn members(pool: &SqlitePool, team_id: i64) -> Result<Vec<TeamMember>, AppError> {
        Ok(TeamMembersRepo::list_members(pool, team_id).await?)
    }
    pub async fn set_override(pool: &SqlitePool, o: TeamOverride) -> Result<(), AppError> {
        validate_override(&o)?;
        Ok(TeamOverridesRepo::upsert(pool, &o).await?)
    }
    pub async fn get_override(pool: &SqlitePool, team_id: i64) -> Result<Option<TeamOverride>, AppError> {
        Ok(TeamOverridesRepo::get(pool, team_id).await?)
    }
}

fn validate_override(o: &TeamOverride) -> Result<(), AppError> {
    if let Some(g) = o.utilization_green { if !(0.0..=1.0).contains(&g) { return Err(domain::DomainError::InvalidRatio(g).into()); } }
    if let Some(y) = o.utilization_yellow { if !(0.0..=1.0).contains(&y) { return Err(domain::DomainError::InvalidRatio(y).into()); } }
    if let Some(p) = o.pd_hours { if p <= 0.0 { return Err(domain::DomainError::InvalidRatio(p).into()); } }
    Ok(())
}
```

- [ ] **Step 4: Add commands — append to `crates/app/src/command.rs`**

```rust
#[tauri::command]
pub async fn create_team(state: tauri::State<'_, AppState>, name: String, description: Option<String>) -> Result<i64, AppError> {
    teams::TeamsService::create(&state.pool, &name, description.as_deref()).await
}
#[tauri::command]
pub async fn add_team_member(state: tauri::State<'_, AppState>, team_id: i64, resource_id: i64, role: Option<String>) -> Result<(), AppError> {
    teams::TeamsService::add_member(&state.pool, team_id, resource_id, role.as_deref()).await
}
#[tauri::command]
pub async fn set_team_override(state: tauri::State<'_, AppState>, o: TeamOverride) -> Result<(), AppError> {
    teams::TeamsService::set_override(&state.pool, o).await
}
```

- [ ] **Step 5: Write the failing test — `crates/app/tests/teams.rs`**

```rust
use app::service::teams::TeamsService;
use app::service::projects::ProjectsService;
use db::models::TeamOverride;
use db::pool::connect;
use db::ResourcesRepo;

#[tokio::test]
async fn team_members_and_override() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let _ = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap(); // satisfy FK-free env

    let rid = ResourcesRepo::create(&pool, "Alice", None).await.unwrap();
    let tid = TeamsService::create(&pool, "Platform", Some("core")).await.unwrap();
    TeamsService::add_member(&pool, tid, rid, Some("lead")).await.unwrap();
    assert_eq!(TeamsService::members(&pool, tid).await.unwrap().len(), 1);

    TeamsService::set_override(&pool, TeamOverride {
        team_id: tid, pd_hours: Some(8.0), pm_workdays: Some(20.0),
        overload_threshold: Some(1.1), underload_threshold: None,
        utilization_green: Some(0.7), utilization_yellow: Some(0.9),
    }).await.unwrap();
    let o = TeamsService::get_override(&pool, tid).await.unwrap().unwrap();
    assert!((o.utilization_green.unwrap() - 0.7).abs() < 1e-9);
}

#[tokio::test]
async fn bad_override_rejected() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let tid = TeamsService::create(&pool, "T", None).await.unwrap();
    let err = TeamsService::set_override(&pool, TeamOverride {
        team_id: tid, pd_hours: None, pm_workdays: None,
        overload_threshold: None, underload_threshold: None,
        utilization_green: Some(1.5), utilization_yellow: None,
    }).await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}
```

- [ ] **Step 6: Run test — verify PASS**

Run: `cargo test -p app --test teams`
Expected: `2 passed`.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(app): Teams/members/overrides repo+service+command"
```

---

## Task 7: Build-check commands + Kanban integration smoke test

Confirm every `#[tauri::command]` compiles and an end-to-end flow (project → team → resource → task with skills/tags → kanban read) works through the service layer.

**Files:**
- Create: `crates/app/tests/smoke_kanban.rs`

- [ ] **Step 1: Verify the full workspace builds (commands compile)**

Run: `cargo build --workspace`
Expected: clean build. If `command.rs` has unused-import warnings for `Skill`/`Tag`/`Team`/`Task`, that's fine — they're used by command return types; if truly unused, remove only the unused ones.

- [ ] **Step 2: Write the smoke test — `crates/app/tests/smoke_kanban.rs`**

```rust
use app::service::catalog::CatalogService;
use app::service::projects::ProjectsService;
use app::service::tasks::TasksService;
use app::service::teams::TeamsService;
use db::pool::connect;
use db::ResourcesRepo;
use db::AllocationsRepo;

#[tokio::test]
async fn end_to_end_kanban_with_assignee() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();

    let pid = ProjectsService::create(&pool, "Atlas", None, Some("2026-06-01"), Some("2026-06-30"), 2, 60.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    let hot = CatalogService::ensure_tag(&pool, "hot", Some("#f00")).await.unwrap();
    let rid = ResourcesRepo::create(&pool, "Alice", None).await.unwrap();
    let team = TeamsService::create(&pool, "Eng", None).await.unwrap();
    TeamsService::add_member(&pool, team, rid, Some("lead")).await.unwrap();

    let tid = TasksService::create(
        &pool, pid, "Implement core", None, 5.0, Some("2026-06-05"), Some("2026-06-15"),
        false, 0, &[(rust, 4, true, 1.0)], &[hot]).await.unwrap();

    // assign Alice (within task window & project window)
    AllocationsRepo::create(&pool, rid, tid, "2026-06-08", "2026-06-12", 0.5).await.unwrap();

    let kb = TasksService::kanban(&pool, pid).await.unwrap();
    assert_eq!(kb.len(), 1);
    assert_eq!(kb[0].assignee.as_deref(), Some("Alice"));
    assert_eq!(kb[0].skill_count, 1);
    assert_eq!(kb[0].status, "todo");
}
```

- [ ] **Step 3: Run the full suite — verify PASS**

Run: `cargo test --workspace`
Expected: all Phase 0 + Phase 1 tests pass (domain 17, db 11, app: error_mapping 4 + projects 3 + catalog 2 + tasks 3 + deps 2(+2 inline) + teams 2 + smoke 1).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "test(app): end-to-end kanban smoke (project/team/task/allocation/assignee)"
```

---

## Task 8: Encrypted database open (design §6.8 / confirmed #55)

DB encryption is default-on. Phase 0 added `connect_with_key(url, Some(passphrase))`; this task enables the SQLCipher build and adds an app-layer `open_encrypted` wrapper plus an end-to-end encryption test (correct key reads, wrong key fails). The first-run passphrase prompt + Argon2id raw-key hardening (§6.8) is frontend (Phase 1b); here we deliver the backend mechanism.

**Files:**
- Modify: `crates/db/Cargo.toml` (SQLCipher build feature)
- Create: `crates/app/src/crypto.rs`
- Modify: `crates/app/src/lib.rs`
- Create: `crates/app/tests/encryption.rs`

- [ ] **Step 1: Enable SQLCipher in `crates/db/Cargo.toml`**

Add a direct dependency so Cargo feature-unification switches sqlx's bundled SQLite to SQLCipher:

```toml
[dependencies]
domain = { path = "../domain" }
tokio = { workspace = true }
thiserror = { workspace = true }
libsqlite3-sys = { version = "0.30", features = ["bundled-sqlcipher-vendored-openssl"] }

[dependencies.sqlx]
version = "0.8"
default-features = false
features = ["runtime-tokio", "sqlite", "macros", "migrate", "chrono"]
```

> **Build caveat (verify at execution time):** the `libsqlite3-sys` version (`0.30` above) MUST equal the version `sqlx 0.8` resolves to, or feature unification won't merge cleanly. Run `cargo tree -i libsqlite3-sys` and pin the version shown. If feature unification does not pick up `bundled-sqlcipher-vendored-openssl`, the `PRAGMA key` will be a silent no-op and the Step 3 wrong-key test will FAIL — that's the signal to fix the version pin. OpenSSL is vendored (`vendored-openssl`), so no system OpenSSL is required; you do need a C/C++ toolchain on the build machine (already required for bundled SQLite).

- [ ] **Step 2: Create `crates/app/src/crypto.rs`**

```rust
use crate::error::AppError;
use crate::state::AppState;

/// Open (or create) the encrypted application database (design §6.8).
/// `passphrase` is obtained by the frontend first-run flow (Phase 1b); here we
/// only wire the mechanism. Migrations run on first open.
pub async fn open_encrypted(db_path: &str, passphrase: &str) -> Result<AppState, AppError> {
    let url = format!("sqlite://{}?mode=rwc", db_path);
    let pool = db::pool::connect_with_key(&url, Some(passphrase)).await?;
    sqlx::migrate!("../db/migrations").run(&pool).await?;
    Ok(AppState { pool })
}
```

- [ ] **Step 3: Register the module — `crates/app/src/lib.rs`**

```rust
pub mod command;
pub mod crypto;
pub mod error;
pub mod service;
pub mod state;

pub use state::AppState;
```

- [ ] **Step 4: Write the failing test — `crates/app/tests/encryption.rs`**

```rust
use db::pool::connect_with_key;

#[tokio::test]
async fn encrypted_roundtrip_and_wrong_key_fails() {
    let path = "/tmp/hrk_enc_test.db";
    let _ = std::fs::remove_file(path);
    let url = format!("sqlite://{}?mode=rwc", path);

    // create encrypted DB, migrate, write a row
    {
        let pool = connect_with_key(&url, Some("correct-horse-battery-staple")).await.unwrap();
        sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
        sqlx::query("INSERT INTO skills (name) VALUES ('Rust')").execute(&pool).await.unwrap();
    }

    // reopen with the CORRECT key -> reads the row
    {
        let pool = connect_with_key(&url, Some("correct-horse-battery-staple")).await.unwrap();
        let n: (i64,) = sqlx::query_as("SELECT count(*) FROM skills").fetch_one(&pool).await.unwrap();
        assert_eq!(n.0, 1);
    }

    // reopen with a WRONG key -> connecting succeeds but reading fails (SQLCipher decrypt error)
    {
        let pool = connect_with_key(&url, Some("wrong-key")).await.unwrap();
        let res: Result<(i64,), _> = sqlx::query_as("SELECT count(*) FROM skills").fetch_one(&pool).await;
        assert!(res.is_err(), "wrong passphrase must fail to decrypt/read");
    }

    let _ = std::fs::remove_file(path);
}
```

- [ ] **Step 5: Run test — verify PASS**

Run: `cargo test -p app --test encryption`
Expected: `1 passed`. If it fails with a successful wrong-key read, the SQLCipher feature did not unify — fix the `libsqlite3-sys` version pin (Step 1 caveat).

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(app): encrypted DB open via SQLCipher (default-on, §6.8)"
```

---

## Self-Review

**Spec coverage (design §6 backend + §3 CRUD entities):**
- §6.4 AppError{code,detail} + DomainError zero-serde + explicit `From` mapper → Task 1 ✓
- §3.3.10 projects CRUD → Task 2 ✓
- §3.3.3/§3.3.2 skills/tags (idempotent ensure) → Task 3 ✓
- §3.3.11/13/14 tasks with skill-reqs + tags (atomic tx) → Task 4 ✓
- §3.3.12 task dependencies + cycle guard → Task 5 ✓
- §3.3.7/8/8a teams + members + overrides → Task 6 ✓
- §6 service/command split (services tested, commands compile) → all tasks ✓
- Kanban-shaped read (assignee + skill count) → Tasks 4 & 7 ✓
- §6.8 encrypted DB open (SQLCipher, default-on, confirmed #55) → Task 8 ✓

**Deferred (explicitly out of scope, not placeholders):**
- `#[tauri::command]` registration in `tauri::Builder` + window + `tauri.conf.json` → Phase 1b (frontend).
- Calendar-table repos (`holiday`/`time_off`/`work_week_template`), `resource_project_rates`, `workload_cache` → Phase 2 (allocations/workload UI).
- `effective_*` team-override resolver (design §3.3.8a) — `TeamMembersRepo::team_of_resource` provides the lookup; the full effective-constants resolver is wired in Phase 2 where workload is consumed.
- DTO layer distinct from row models — Phase 1 returns Serialize-deriving row models directly; a dedicated DTO refactor is optional and noted.

**Placeholder scan:** none. Every code step contains complete code; every test asserts concrete values and codes.

**Type consistency:**
- `AppError{code, detail}` used uniformly; `.code` assertions match constructors (`VALIDATION`/`DOMAIN`/`NOT_FOUND`).
- `DomainError` variants referenced in services (`InvalidRatio`, `InvalidDateWindow`, `DependencyCycle`) all exist in Task 1's enum.
- `TaskCreate<'a>` fields match `TasksService::create` arguments; `KanbanTask` columns match the `SELECT` alias order (`assignee`, `skill_count`).
- `with_write_tx` closure returns `Ok(id)` (T = i64) matching Phase 0's borrowed-`&mut Transaction` contract.
- Repo method names (`ProjectsRepo::create/list_active/get/set_status/soft_delete`, `TasksRepo::create/list_by_project/set_status/list_kanban`, `TaskDepsRepo::add/all_edges`, `TeamsRepo`/`TeamMembersRepo`/`TeamOverridesRepo`) are consistent across tasks.

**Known impl-time items carried forward (from design, not blockers):**
- `TaskCreate` closure uses `Box::pin` + `&mut *tx` to satisfy Phase 0's borrowed `with_write_tx` signature (verified against Phase 0 Task 8).
- Calendar/resolver + workload-cache integration → Phase 2.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-06-27-hr-kanban-phase1-backend.md`. Two execution options:

1. **Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks.
2. **Inline Execution** — Execute tasks in this session in batches with checkpoints.

Which approach? (After Phase 0 + Phase 1 backend are green, the next plan is **Phase 1b: Frontend + Kanban**.)
