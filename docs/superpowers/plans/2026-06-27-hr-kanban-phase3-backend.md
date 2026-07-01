# HR Kanban — Phase 3: Backend (Gantt + Cross-project + Calendar Occupancy) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Provide the read-side data the Phase 3b Gantt and Calendar views need: **Gantt bars** by project and by resource (cross-project), **dependency edges** for arrows, and **per-day per-resource occupancy** for the calendar view.

**Architecture:** A `GanttRepo` adds two join queries (project view + cross-project resource view) returning a flat `GanttBar` list the frontend lays out. `TaskDepsRepo::for_project` returns dependency edges within a project. A `CalendarOccupancyService` reuses the Phase 2 `hydrate()` + Phase 0 per-day workload math to produce daily occupancy. All read-only; on-demand (no cache).

**Tech Stack:** Rust, `sqlx`, `chrono`, `serde`, `tauri` commands. No new deps.

**Prerequisite:** Phase 0/1/2 backend green. Uses `db::calendar::hydrate`, `db::AllocationsRepo::list_for_resource`, `domain::{capacity_pd, workload_pd, Window}`, `AllocationRow::to_domain`.

**Scope note:** Backend read queries + commands only. The Gantt UI (bars, drag-resize, dependency arrows, virtualization) and the Calendar occupancy grid are **Phase 3b** (frontend). Allocation mutation (resize/reassign via drag) reuses `create_allocation`/`set_task_status` plus a future `update_allocation` — the resize command is noted for 3b.

**Reference design:** `docs/design/2026-06-27-kanban-design.md` (§7 Gantt resource/project views, §3.6 queries, §4.6 aggregation).

---

## File Structure

```
kanban/
├── crates/db/src/
│   ├── models.rs            # MOD: add GanttBar, DepEdge, DayOccupancy
│   └── repo/
│       ├── mod.rs           # MOD
│       ├── gantt.rs         # NEW: GanttRepo
│       └── tasks.rs         # MOD: TaskDepsRepo::for_project
└── crates/app/src/
    ├── service/
    │   ├── mod.rs           # MOD: add occupancy
    │   └── occupancy.rs     # NEW: CalendarOccupancyService
    ├── command.rs           # MOD: gantt/occupancy/deps commands
    └── tests/
        ├── gantt.rs         # NEW
        └── occupancy.rs     # NEW
```

---

## Task 1: GanttRepo — project view + cross-project resource view (TDD)

**Files:**
- Modify: `crates/db/src/models.rs`
- Create: `crates/db/src/repo/gantt.rs`
- Modify: `crates/db/src/repo/mod.rs`
- Create: `crates/db/tests/gantt.rs`

- [ ] **Step 1: Models — append to `crates/db/src/models.rs`**

```rust
/// One allocation bar for a Gantt view (joined with resource/task/project names).
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct GanttBar {
    pub allocation_id: i64,
    pub resource_id: i64,
    pub resource_name: String,
    pub task_id: i64,
    pub task_title: String,
    pub project_id: i64,
    pub project_name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub percent: f64,
    pub status: String,
    pub source: String,
}

/// A dependency edge for drawing Gantt arrows.
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct DepEdge {
    pub task_id: i64,
    pub predecessor_id: i64,
    pub lag_days: i64,
    pub dep_type: String,
}

/// Per-day per-resource occupancy (calendar view).
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct DayOccupancy {
    pub date: NaiveDate,
    pub resource_id: i64,
    pub resource_name: String,
    pub workload_pd: f64,
    pub capacity_pd: f64,
    pub utilization: f64,
}
```

- [ ] **Step 2: `crates/db/src/repo/gantt.rs`**

```rust
use crate::error::DbError;
use crate::models::GanttBar;
use sqlx::SqlitePool;

pub struct GanttRepo;
impl GanttRepo {
    /// All allocation bars in a project (project Gantt view).
    pub async fn by_project(pool: &SqlitePool, project_id: i64) -> Result<Vec<GanttBar>, DbError> {
        Ok(sqlx::query_as::<_, GanttBar>(
            "SELECT a.id AS allocation_id, a.resource_id, r.name AS resource_name, \
                    a.task_id, t.title AS task_title, t.project_id, p.name AS project_name, \
                    a.start_date, a.end_date, a.percent, a.status, a.source \
             FROM allocations a \
             JOIN resources r ON r.id = a.resource_id \
             JOIN tasks t ON t.id = a.task_id \
             JOIN projects p ON p.id = t.project_id \
             WHERE t.project_id = ? AND a.deleted_at IS NULL \
             ORDER BY r.name, a.start_date")
            .bind(project_id).fetch_all(pool).await?)
    }

    /// A resource's allocation bars across ALL projects (cross-project resource Gantt view).
    pub async fn by_resource(pool: &SqlitePool, resource_id: i64) -> Result<Vec<GanttBar>, DbError> {
        Ok(sqlx::query_as::<_, GanttBar>(
            "SELECT a.id AS allocation_id, a.resource_id, r.name AS resource_name, \
                    a.task_id, t.title AS task_title, t.project_id, p.name AS project_name, \
                    a.start_date, a.end_date, a.percent, a.status, a.source \
             FROM allocations a \
             JOIN resources r ON r.id = a.resource_id \
             JOIN tasks t ON t.id = a.task_id \
             JOIN projects p ON p.id = t.project_id \
             WHERE a.resource_id = ? AND a.deleted_at IS NULL \
             ORDER BY a.start_date")
            .bind(resource_id).fetch_all(pool).await?)
    }
}
```

- [ ] **Step 3: Register — `crates/db/src/repo/mod.rs`**

```rust
pub mod gantt;
pub use gantt::GanttRepo;
```

- [ ] **Step 4: Test — `crates/db/tests/gantt.rs`**

```rust
use db::pool::connect;
use db::repo::gantt::GanttRepo;

#[tokio::test]
async fn by_project_and_cross_project_by_resource() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    sqlx::query("INSERT INTO projects (id,name) VALUES (1,'Atlas')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO projects (id,name) VALUES (2,'Borealis')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T1','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (11,2,'T2','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO allocations (resource_id,task_id,start_date,end_date,percent) VALUES (1,10,'2026-06-08','2026-06-12',0.5)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO allocations (resource_id,task_id,start_date,end_date,percent) VALUES (1,11,'2026-06-15','2026-06-19',0.3)").execute(&pool).await.unwrap();

    // project view: only Atlas's allocation
    let p1 = GanttRepo::by_project(&pool, 1).await.unwrap();
    assert_eq!(p1.len(), 1);
    assert_eq!(p1[0].project_name, "Atlas");
    assert_eq!(p1[0].resource_name, "Alice");

    // cross-project resource view: both, different projects
    let r1 = GanttRepo::by_resource(&pool, 1).await.unwrap();
    assert_eq!(r1.len(), 2);
    let names: Vec<_> = r1.iter().map(|b| b.project_name.as_str()).collect();
    assert!(names.contains(&"Atlas") && names.contains(&"Borealis"));
}
```

- [ ] **Step 5: Run — PASS**

Run: `cargo test -p db --test gantt`
Expected: `1 passed`.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(db): GanttRepo (project + cross-project resource views)"
```

---

## Task 2: Dependency edges per project (TDD)

**Files:**
- Modify: `crates/db/src/repo/tasks.rs` (extend `TaskDepsRepo`)
- Create: `crates/app/tests/deps_project.rs` (or extend an existing deps test)

- [ ] **Step 1: Add `for_project` to `TaskDepsRepo` in `crates/db/src/repo/tasks.rs`**

```rust
use crate::models::DepEdge;

impl TaskDepsRepo {
    /// Dependency edges among tasks of one project (for Gantt arrows).
    pub async fn for_project(pool: &SqlitePool, project_id: i64) -> Result<Vec<DepEdge>, DbError> {
        Ok(sqlx::query_as::<_, DepEdge>(
            "SELECT d.task_id, d.predecessor_id, d.lag_days, d.dep_type \
             FROM task_dependencies d JOIN tasks t ON t.id = d.task_id \
             WHERE t.project_id = ?")
            .bind(project_id).fetch_all(pool).await?)
    }
}
```

- [ ] **Step 2: Export `DepEdge` — `crates/db/src/repo/mod.rs`** (models already re-exported via `db::models`)

(No change needed; `DepEdge` is in `db::models`.)

- [ ] **Step 3: Test — append to `crates/app/tests/deps.rs`**

```rust
use db::repo::tasks::TaskDepsRepo;

#[tokio::test]
async fn deps_for_project_returns_edges() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let a = TasksService::create(&pool, pid, "A", None, 1.0, None, None, false, 0, &[], &[]).await.unwrap();
    let b = TasksService::create(&pool, pid, "B", None, 1.0, None, None, false, 1, &[], &[]).await.unwrap();
    TasksService::add_dependency(&pool, b, a, 0).await.unwrap();

    let edges = TaskDepsRepo::for_project(&pool, pid).await.unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].task_id, b);
    assert_eq!(edges[0].predecessor_id, a);
}
```

(Add `use db::repo::tasks::TaskDepsRepo;` to the imports if not present.)

- [ ] **Step 4: Run — PASS**

Run: `cargo test -p app --test deps`
Expected: prior tests + `deps_for_project_returns_edges` pass.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(db): TaskDepsRepo::for_project (Gantt dependency edges)"
```

---

## Task 3: Calendar occupancy service (TDD)

**Files:**
- Create: `crates/app/src/service/occupancy.rs`
- Modify: `crates/app/src/service/mod.rs`
- Create: `crates/app/tests/occupancy.rs`

- [ ] **Step 1: `crates/app/src/service/occupancy.rs`**

```rust
use crate::error::AppError;
use chrono::NaiveDate;
use db::models::DayOccupancy;
use domain::{capacity_pd, workload_pd, Window};
use sqlx::SqlitePool;

pub struct CalendarOccupancyService;

impl CalendarOccupancyService {
    /// Per-day per-resource workload/capacity/utilization across [start, end]
    /// (for the calendar occupancy grid). Skips days with zero capacity (non-working days).
    pub async fn range(
        pool: &SqlitePool, start: &str, end: &str,
    ) -> Result<Vec<DayOccupancy>, AppError> {
        let cal = db::calendar::hydrate(pool).await?;
        let s = NaiveDate::parse_from_str(start, "%Y-%m-%d").map_err(|_| domain::DomainError::InvalidDateWindow)?;
        let e = NaiveDate::parse_from_str(end, "%Y-%m-%d").map_err(|_| domain::DomainError::InvalidDateWindow)?;
        if e < s { return Err(domain::DomainError::InvalidDateWindow.into()); }

        let mut out = Vec::new();
        for r in db::ResourcesRepo::list_active(pool).await? {
            let rows = db::AllocationsRepo::list_for_resource(pool, r.id, start, end).await?;
            let allocs: Vec<domain::Allocation> = rows.iter().map(|(row, pid)| row.to_domain(*pid)).collect();
            let mut d = s;
            while d <= e {
                let w = Window { start: d, end: d };
                let cap = capacity_pd(&cal, 0, r.id, w);
                if cap > 0.0 {
                    let wl = workload_pd(&cal, &allocs, r.id, w);
                    out.push(DayOccupancy {
                        date: d, resource_id: r.id, resource_name: r.name.clone(),
                        workload_pd: wl, capacity_pd: cap,
                        utilization: wl / cap,
                    });
                }
                d = d.succ_opt().unwrap();
            }
        }
        Ok(out)
    }
}
```

- [ ] **Step 2: Register — `crates/app/src/service/mod.rs`**

```rust
pub mod occupancy;
```

- [ ] **Step 3: Test — `crates/app/tests/occupancy.rs`**

```rust
use app::service::occupancy::CalendarOccupancyService;
use db::pool::connect;
use db::AllocationsRepo;

#[tokio::test]
async fn daily_occupancy_for_half_loaded_week() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    sqlx::query("INSERT INTO projects (id,name) VALUES (1,'P')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    // Alice 50% Mon..Fri (2026-06-29..2026-07-03)
    AllocationsRepo::create(&pool, 1, 10, "2026-06-29", "2026-07-03", 0.5).await.unwrap();

    let occ = CalendarOccupancyService::range(&pool, "2026-06-29", "2026-07-05").await.unwrap();
    // 5 working days (Mon-Fri); each: workload 0.5, capacity 1.0, utilization 0.5
    assert_eq!(occ.len(), 5);
    for o in &occ {
        assert!((o.workload_pd - 0.5).abs() < 1e-9);
        assert!((o.capacity_pd - 1.0).abs() < 1e-9);
        assert!((o.utilization - 0.5).abs() < 1e-9);
    }
    // weekend days (cap 0) are skipped
    assert!(occ.iter().all(|o| o.date.weekday().num_days_from_monday() < 5));
}

use chrono::Datelike; // for .weekday()
```

- [ ] **Step 4: Run — PASS**

Run: `cargo test -p app --test occupancy`
Expected: `1 passed`.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(app): CalendarOccupancyService (per-day per-resource)"
```

---

## Task 4: Commands + integration test

**Files:**
- Modify: `crates/app/src/command.rs`
- Modify: `src-tauri/src/main.rs` (register)

- [ ] **Step 1: Add commands — append to `crates/app/src/command.rs`**

```rust
use crate::service::occupancy::CalendarOccupancyService;
use db::models::{DayOccupancy, DepEdge, GanttBar};
use db::{GanttRepo, TaskDepsRepo};

#[tauri::command]
pub async fn gantt_project(state: tauri::State<'_, AppState>, project_id: i64) -> Result<Vec<GanttBar>, AppError> {
    Ok(GanttRepo::by_project(&state.pool, project_id).await?)
}
#[tauri::command]
pub async fn gantt_resource(state: tauri::State<'_, AppState>, resource_id: i64) -> Result<Vec<GanttBar>, AppError> {
    Ok(GanttRepo::by_resource(&state.pool, resource_id).await?)
}
#[tauri::command]
pub async fn dependencies_for_project(state: tauri::State<'_, AppState>, project_id: i64) -> Result<Vec<DepEdge>, AppError> {
    Ok(TaskDepsRepo::for_project(&state.pool, project_id).await?)
}
#[tauri::command]
pub async fn daily_occupancy(state: tauri::State<'_, AppState>, start: String, end: String) -> Result<Vec<DayOccupancy>, AppError> {
    CalendarOccupancyService::range(&state.pool, &start, &end).await
}
```

- [ ] **Step 2: Register in `src-tauri/src/main.rs`** — add to import + `generate_handler![...]`:

```
gantt_project, gantt_resource, dependencies_for_project, daily_occupancy,
```

- [ ] **Step 3: Build + full suite**

Run: `cargo build --workspace && cargo test --workspace`
Expected: clean build; all tests pass (incl. gantt 1, occupancy 1, deps +1).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(app): gantt/occupancy/deps commands; Phase 3 backend complete"
```

---

## Self-Review

**Spec coverage (design §7 Gantt/calendar + roadmap Phase 3 backend):**
- §7 Gantt project view → `GanttRepo::by_project` (Task 1) ✓
- §7 Gantt resource/cross-project view → `GanttRepo::by_resource` (Task 1) ✓
- §7 dependency arrows → `TaskDepsRepo::for_project` (Task 2) ✓
- §7 calendar daily occupancy → `CalendarOccupancyService::range` (Task 3) ✓
- Cross-project resource reuse (design §1) → resource view spans all projects (Task 1) ✓

**Deferred (not placeholders):** the Gantt/Calendar **UI** (Phase 3b); allocation drag-resize (`update_allocation` command — small add in 3b); `workload_cache`; long-term-task segmentation display; virtualization.

**Placeholder scan:** none — complete code; tests assert concrete values (bar counts, project names, per-day 0.5/1.0/0.5).

**Type consistency:**
- `GanttBar`/`DepEdge`/`DayOccupancy` SELECT alias order matches struct field order (`allocation_id`, `resource_name`, `task_title`, `project_name`, …).
- `CalendarOccupancyService` reuses Phase 2 `hydrate()` + Phase 0 `capacity_pd`/`workload_pd`/`Window` + `AllocationRow::to_domain` — same seam as `WorkloadService`, so per-day values are consistent with window aggregates.
- `capacity_pd(&cal, 0, r.id, day)` uses project_id `0` ⇒ global calendar (Phase 2 convention); weekends (cap 0) skipped.

**Known impl-time items:** occupancy is O(resources × days) on-demand — fine at MVP (≤10 res × ~30 days); a materialized table is a later optimization if the calendar view lags. Drag-resize needs an `update_allocation(resource,task,start,end,percent)` command (added in Phase 3b).

---

## Execution Handoff

Plan saved to `docs/superpowers/plans/2026-06-27-kanban-phase3-backend.md`. Options: **1. Subagent-Driven** (recommended) or **2. Inline Execution**. Which? (Next: **Phase 3b frontend** — Gantt bars + dependency arrows + calendar occupancy grid, consuming these commands.)
