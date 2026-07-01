# HR Kanban — Phase 2: Backend (Workload Service + Calendar) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Compute real-time per-resource and per-team workload/capacity/utilization (and detect overload) by hydrating the DB calendar into the pure `domain::Calendar` and running the Phase 0 workload core over persisted allocations — exposed via Tauri commands for the Phase 2b Dashboard.

**Architecture:** A `db::calendar` module hydrates `work_week_template`/`holiday`/`time_off` rows into `domain::Calendar` (the in-memory struct from Phase 0 Task 3). An `app::WorkloadService` bridges persisted allocations (`AllocationsRepo::list_for_resource` → `AllocationRow::to_domain`) into `domain::Allocation` and calls the Phase 0 pure functions (`capacity_pd`, `workload_pd`, `utilization`, `team_utilization`). Effective overload thresholds resolve `team_overrides → settings → const default` (design §3.3.8a). Computation is on-demand per window (design §4.9: <5ms); `workload_cache` materialization is deferred.

**Tech Stack:** Rust, `sqlx`, `tokio`, `chrono`, `serde`, `tauri` (commands). No new heavy deps.

**Prerequisite:** Phase 0 + Phase 1 backend green. Specifically: `domain::{Calendar, DayFraction, Allocation, Window, capacity_pd, workload_pd, utilization, team_utilization}`, `db::{DbError, connect, with_write_tx, AllocationsRepo, ResourcesRepo, TeamMembersRepo, TeamOverridesRepo, ProjectsRepo, TasksRepo}`, `app::{AppError, AppState, DomainError mapping}`.

**Scope note:** Backend only. The Dashboard / allocation-editor / calendar-management **UI** is Phase 2b. Computation is on-demand (no `workload_cache`); long-term-task segmentation enforcement and `config_hash`/cache are Phase 3+.

**Reference design:** `docs/design/2026-06-27-kanban-design.md` (§4 workload core, §3.3.8a/9 calendar, §3.6 queries, §7 Dashboard data needs).

---

## File Structure

```
kanban/
├── crates/db/
│   ├── migrations/
│   │   └── 0002_settings_thresholds.sql   # NEW
│   └── src/
│       ├── models.rs                       # MOD: add WeekTemplate/Holiday/TimeOff/Settings rows
│       └── repo/
│           ├── mod.rs                      # MOD: declare calendar + settings
│           ├── calendar.rs                 # NEW: repos + hydrate()
│           └── settings.rs                 # NEW: SettingsRepo
└── crates/app/src/
    ├── service/
    │   ├── mod.rs                          # MOD: add workload
    │   └── workload.rs                     # NEW: WorkloadService + summary DTOs
    ├── command.rs                          # MOD: add workload + calendar commands
    └── tests/
        ├── workload.rs                     # NEW
        └── calendar.rs                     # NEW
```

**Responsibilities:** `db::calendar` owns calendar CRUD + the one-way hydration to `domain::Calendar` (pure data). `app::WorkloadService` is the only place that combines a hydrated `Calendar` + bridged allocations + the domain math to produce utilization. Threshold resolution is a small helper reading `team_overrides`/`settings`.

---

## Task 1: Calendar repos + hydrate (TDD, golden)

**Files:**
- Create: `crates/db/src/repo/calendar.rs`
- Modify: `crates/db/src/repo/mod.rs`
- Modify: `crates/db/src/models.rs`
- Create: `crates/db/tests/calendar.rs`

- [ ] **Step 1: Add calendar row models — append to `crates/db/src/models.rs`**

```rust
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct WeekTemplate {
    pub id: i64,
    pub scope: String,
    pub project_id: Option<i64>,
    pub mon: i64, pub tue: i64, pub wed: i64, pub thu: i64, pub fri: i64, pub sat: i64, pub sun: i64,
    pub mon_frac: f64, pub tue_frac: f64, pub wed_frac: f64, pub thu_frac: f64,
    pub fri_frac: f64, pub sat_frac: f64, pub sun_frac: f64,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Holiday {
    pub id: i64,
    pub project_id: Option<i64>,
    pub day: String,
    pub fraction: f64,
    pub name: Option<String>,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct TimeOff {
    pub id: i64,
    pub resource_id: i64,
    pub day: String,
    pub fraction: f64,
    pub reason: Option<String>,
}
```

- [ ] **Step 2: Write `crates/db/src/repo/calendar.rs`** (repos + hydrate)

```rust
use crate::error::DbError;
use crate::models::{Holiday, TimeOff, WeekTemplate};
use chrono::NaiveDate;
use sqlx::SqlitePool;

// ---- Work-week template ----
pub struct WeekTemplateRepo;
impl WeekTemplateRepo {
    /// Upsert the global template (design §3.3.9a; idx_wwt_global enforces one global row).
    pub async fn upsert_global(
        pool: &SqlitePool, week: [f64; 7],
    ) -> Result<(), DbError> {
        db::tx::with_write_tx(pool, |tx| Box::pin(async move {
            sqlx::query(
                "INSERT INTO work_week_template (scope, mon,tue,wed,thu,fri,sat,sun,
                   mon_frac,tue_frac,wed_frac,thu_frac,fri_frac,sat_frac,sun_frac)
                 VALUES ('global', ?,?,?,?,?,?,?,  ?,?,?,?,?,?,?)
                 ON CONFLICT DO UPDATE SET
                   mon=excluded.mon, tue=excluded.tue, wed=excluded.wed, thu=excluded.thu,
                   fri=excluded.fri, sat=excluded.sat, sun=excluded.sun,
                   mon_frac=excluded.mon_frac, tue_frac=excluded.tue_frac, wed_frac=excluded.wed_frac,
                   thu_frac=excluded.thu_frac, fri_frac=excluded.fri_frac, sat_frac=excluded.sat_frac,
                   sun_frac=excluded.sun_frac")
                .bind(week[0] > 0).bind(week[1] > 0).bind(week[2] > 0).bind(week[3] > 0)
                .bind(week[4] > 0).bind(week[5] > 0).bind(week[6] > 0)
                .bind(week[0]).bind(week[1]).bind(week[2]).bind(week[3])
                .bind(week[4]).bind(week[5]).bind(week[6])
                .execute(&mut **tx).await?;
            Ok(())
        })).await
    }

    pub async fn list(pool: &SqlitePool) -> Result<Vec<WeekTemplate>, DbError> {
        Ok(sqlx::query_as::<_, WeekTemplate>(
            "SELECT id, scope, project_id, mon,tue,wed,thu,fri,sat,sun,
                    mon_frac,tue_frac,wed_frac,thu_frac,fri_frac,sat_frac,sun_frac \
             FROM work_week_template")
            .fetch_all(pool).await?)
    }
}

// ---- Holidays ----
pub struct HolidayRepo;
impl HolidayRepo {
    pub async fn add(pool: &SqlitePool, project_id: Option<i64>, day: &str, fraction: f64, name: Option<&str>) -> Result<i64, DbError> {
        let (id,): (i64,) = sqlx::query_as(
            "INSERT INTO holiday (project_id, day, fraction, name) VALUES (?,?,?,?) RETURNING id")
            .bind(project_id).bind(day).bind(fraction).bind(name).fetch_one(pool).await?;
        Ok(id)
    }
    pub async fn list(pool: &SqlitePool) -> Result<Vec<Holiday>, DbError> {
        Ok(sqlx::query_as::<_, Holiday>("SELECT id, project_id, day, fraction, name FROM holiday ORDER BY day")
            .fetch_all(pool).await?)
    }
}

// ---- Time off ----
pub struct TimeOffRepo;
impl TimeOffRepo {
    pub async fn add(pool: &SqlitePool, resource_id: i64, day: &str, fraction: f64, reason: Option<&str>) -> Result<i64, DbError> {
        let (id,): (i64,) = sqlx::query_as(
            "INSERT INTO time_off (resource_id, day, fraction, reason) VALUES (?,?,?,?) RETURNING id")
            .bind(resource_id).bind(day).bind(fraction).bind(reason).fetch_one(pool).await?;
        Ok(id)
    }
    pub async fn list_for_resource(pool: &SqlitePool, resource_id: i64) -> Result<Vec<TimeOff>, DbError> {
        Ok(sqlx::query_as::<_, TimeOff>(
            "SELECT id, resource_id, day, fraction, reason FROM time_off WHERE resource_id = ? ORDER BY day")
            .bind(resource_id).fetch_all(pool).await?)
    }
}

// ---- Hydration: DB rows -> pure domain::Calendar ----

fn frac_of(bit: i64, f: f64) -> f64 { if bit == 0 { 0.0 } else { f } }

/// Load all calendar rows into a `domain::Calendar` (design §4.9 authoritative input).
pub async fn hydrate(pool: &SqlitePool) -> Result<domain::Calendar, DbError> {
    let mut cal = domain::Calendar::default();
    for w in WeekTemplateRepo::list(pool).await? {
        let days = [
            frac_of(w.mon, w.mon_frac), frac_of(w.tue, w.tue_frac), frac_of(w.wed, w.wed_frac),
            frac_of(w.thu, w.thu_frac), frac_of(w.fri, w.fri_frac), frac_of(w.sat, w.sat_frac),
            frac_of(w.sun, w.sun_frac),
        ];
        let df = domain::DayFraction { days };
        match (w.scope.as_str(), w.project_id) {
            ("global", _) => cal.global_week = Some(df),
            ("project", Some(pid)) => { cal.project_weeks.insert(pid, df); }
            _ => {}
        }
    }
    for h in HolidayRepo::list(pool).await? {
        if let Ok(d) = NaiveDate::parse_from_str(&h.day, "%Y-%m-%d") {
            match h.project_id {
                Some(pid) => { cal.holidays_project.insert((pid, d), h.fraction); }
                None => { cal.holidays_global.insert(d, h.fraction); }
            }
        }
    }
    // time_off: hydrate all (small in MVP); a window-scoped query is a later optimization.
    let rows: Vec<(i64, String, f64)> = sqlx::query_as("SELECT resource_id, day, fraction FROM time_off")
        .fetch_all(pool).await?;
    for (rid, day, frac) in rows {
        if let Ok(d) = NaiveDate::parse_from_str(&day, "%Y-%m-%d") {
            cal.time_off.insert((rid, d), frac);
        }
    }
    Ok(cal)
}
```

> `hydrate` is the single bridge from DB calendar state to the pure `domain::Calendar` that the Phase 0 math runs on. `with_write_tx` is used for the upsert (busy-retry); the closure uses `&mut **tx` per the Phase 0 borrowed pattern.

- [ ] **Step 3: Register the module — `crates/db/src/repo/mod.rs`**

```rust
pub mod allocations;
pub mod calendar;
pub mod projects;
pub mod resources;
pub mod skills;
pub mod tags;
pub mod tasks;
pub mod teams;
pub use allocations::AllocationsRepo;
pub use calendar::{HolidayRepo, TimeOffRepo, WeekTemplateRepo};
pub use projects::ProjectsRepo;
pub use resources::ResourcesRepo;
pub use skills::SkillsRepo;
pub use tags::TagsRepo;
pub use tasks::{TaskCreate, TaskDepsRepo, TasksRepo};
pub use teams::{TeamMembersRepo, TeamOverridesRepo, TeamsRepo};
```

- [ ] **Step 4: Write the failing test — `crates/db/tests/calendar.rs`**

```rust
use db::pool::connect;
use db::repo::calendar::{hydrate, HolidayRepo, TimeOffRepo, WeekTemplateRepo};
use domain::{capacity_pd, Calendar, DayFraction, Window};
use chrono::NaiveDate;

fn d(s: &str) -> NaiveDate { NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap() }
fn win(s: &str, e: &str) -> Window { Window { start: d(s), end: d(e) } }

async fn fresh() -> sqlx::SqlitePool {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn hydrate_reflects_db_calendar() {
    let pool = fresh().await;
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO projects (id,name) VALUES (1,'P')").execute(&pool).await.unwrap();

    // global week Mon-Fri (already seeded by 0001, but re-assert via upsert)
    WeekTemplateRepo::upsert_global(&pool, [1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0]).await.unwrap();
    // global full-day holiday on Wed 2026-07-01
    HolidayRepo::add(&pool, None, "2026-07-01", 1.0, Some("Holiday")).await.unwrap();
    // Alice half-day time-off Thu 2026-07-02
    TimeOffRepo::add(&pool, 1, "2026-07-02", 0.5, Some("leave")).await.unwrap();

    let cal = hydrate(&pool).await.unwrap();
    // capacity Mon..Fri with Wed holiday + Thu half: 1+1+0+0.5+1 = 3.5
    let cap = capacity_pd(&cal, 1, 1, win("2026-06-29", "2026-07-03"));
    assert!((cap - 3.5).abs() < 1e-9);
}

#[tokio::test]
async fn hydrate_empty_returns_default_calendar() {
    let pool = fresh().await;
    let cal = hydrate(&pool).await.unwrap();
    // no global week inserted yet beyond seed; seed gives Mon-Fri -> capacity 5 for a workweek
    let cap = capacity_pd(&cal, 1, 1, win("2026-06-29", "2026-07-03"));
    assert!((cap - 5.0).abs() < 1e-9);
}
```

- [ ] **Step 5: Run test — verify PASS**

Run: `cargo test -p db --test calendar`
Expected: `2 passed`. (The seeded global Mon–Fri template from migration 0001 makes the empty-hydrate case yield 5.0 PD for a Mon–Fri week.)

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(db): calendar repos + hydrate() to domain::Calendar"
```

---

## Task 2: Settings thresholds (migration 0002) + SettingsRepo + effective resolver (TDD)

**Files:**
- Create: `crates/db/migrations/0002_settings_thresholds.sql`
- Create: `crates/db/src/repo/settings.rs`
- Modify: `crates/db/src/repo/mod.rs`
- Create: `crates/app/src/service/thresholds.rs` (effective resolver)
- Modify: `crates/app/src/service/mod.rs`
- Create: `crates/app/tests/thresholds.rs`

- [ ] **Step 1: Migration `crates/db/migrations/0002_settings_thresholds.sql`**

```sql
-- 0002: global utilization thresholds (team-level overrides live in team_overrides).
ALTER TABLE settings ADD COLUMN overload_threshold  REAL CHECK (overload_threshold  IS NULL OR overload_threshold  > 0);
ALTER TABLE settings ADD COLUMN underload_threshold REAL CHECK (underload_threshold IS NULL OR underload_threshold >= 0);
ALTER TABLE settings ADD COLUMN utilization_green   REAL CHECK (utilization_green   IS NULL OR (utilization_green   >= 0 AND utilization_green   <= 1.0));
ALTER TABLE settings ADD COLUMN utilization_yellow  REAL CHECK (utilization_yellow  IS NULL OR (utilization_yellow  >= 0 AND utilization_yellow  <= 1.0));
UPDATE settings SET overload_threshold = 1.10, underload_threshold = 0.50, utilization_green = 0.70, utilization_yellow = 1.00 WHERE id = 1;
```

- [ ] **Step 2: `crates/db/src/repo/settings.rs`**

```rust
use crate::error::DbError;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Copy)]
pub struct Thresholds {
    pub overload: f64,
    pub underload: f64,
    pub green: f64,
    pub yellow: f64,
}

pub struct SettingsRepo;
impl SettingsRepo {
    pub async fn thresholds(pool: &SqlitePool) -> Result<Thresholds, DbError> {
        let (overload, underload, green, yellow): (Option<f64>, Option<f64>, Option<f64>, Option<f64>) =
            sqlx::query_as(
                "SELECT overload_threshold, underload_threshold, utilization_green, utilization_yellow \
                 FROM settings WHERE id = 1")
            .fetch_one(pool).await?;
        Ok(Thresholds {
            overload: overload.unwrap_or(1.10),
            underload: underload.unwrap_or(0.50),
            green: green.unwrap_or(0.70),
            yellow: yellow.unwrap_or(1.00),
        })
    }
}
```

- [ ] **Step 3: Register — `crates/db/src/repo/mod.rs`** (add)

```rust
pub mod settings;
pub use settings::{SettingsRepo, Thresholds};
```

- [ ] **Step 4: Effective resolver — `crates/app/src/service/thresholds.rs`**

```rust
use crate::error::AppError;
use db::{SettingsRepo, TeamMembersRepo, TeamOverridesRepo};
use sqlx::SqlitePool;

/// Effective overload threshold for a resource:
/// team_overrides.overload_threshold (if the resource's team has one) → settings → 1.10.
/// (design §3.3.8a; confirmed #3: thresholds configurable per team/role.)
pub async fn effective_overload(pool: &SqlitePool, resource_id: i64) -> Result<f64, AppError> {
    if let Some(team_id) = TeamMembersRepo::team_of_resource(pool, resource_id).await? {
        if let Some(o) = TeamOverridesRepo::get(pool, team_id).await? {
            if let Some(t) = o.overload_threshold { return Ok(t); }
        }
    }
    Ok(SettingsRepo::thresholds(pool).await?.overload)
}
```

- [ ] **Step 5: Register — `crates/app/src/service/mod.rs`**

```rust
pub mod catalog;
pub mod projects;
pub mod tasks;
pub mod teams;
pub mod thresholds;
pub mod workload;
```

(`workload` is created in Task 3; create a stub `pub mod workload;` file `crates/app/src/service/workload.rs` with a placeholder `pub struct WorkloadService;` so this compiles, replaced in Task 3.)

- [ ] **Step 6: Write the failing test — `crates/app/tests/thresholds.rs`**

```rust
use app::service::thresholds::effective_overload;
use app::service::teams::TeamsService;
use db::models::TeamOverride;
use db::pool::connect;
use db::ResourcesRepo;

#[tokio::test]
async fn resolves_team_override_then_settings_default() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();

    // resource without a team -> settings default 1.10
    let rid = ResourcesRepo::create(&pool, "Alice", None).await.unwrap();
    assert!((effective_overload(&pool, rid).await.unwrap() - 1.10).abs() < 1e-9);

    // put Alice in a team with override 1.30 -> wins
    let tid = TeamsService::create(&pool, "Eng", None).await.unwrap();
    TeamsService::add_member(&pool, tid, rid, Some("lead")).await.unwrap();
    TeamsService::set_override(&pool, TeamOverride {
        team_id: tid, pd_hours: None, pm_workdays: None,
        overload_threshold: Some(1.30), underload_threshold: None,
        utilization_green: None, utilization_yellow: None,
    }).await.unwrap();
    assert!((effective_overload(&pool, rid).await.unwrap() - 1.30).abs() < 1e-9);
}
```

- [ ] **Step 7: Run test — verify PASS**

Run: `cargo test -p app --test thresholds`
Expected: `1 passed`.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat(db,app): settings thresholds (mig 0002) + effective overload resolver"
```

---

## Task 3: WorkloadService — resource summary + overloads (TDD, §4.10 golden)

**Files:**
- Create: `crates/app/src/service/workload.rs` (replace stub)
- Create: `crates/app/tests/workload.rs`

- [ ] **Step 1: Write `crates/app/src/service/workload.rs`**

```rust
use crate::error::AppError;
use crate::service::thresholds::effective_overload;
use chrono::NaiveDate;
use db::AllocationsRepo;
use domain::{capacity_pd, workload_pd, Window};
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize)]
pub struct ResourceSummary {
    pub resource_id: i64,
    pub capacity_pd: f64,
    pub workload_pd: f64,
    pub utilization: f64,
    pub overloaded: bool,
}

pub struct WorkloadService;

impl WorkloadService {
    /// Per-resource utilization over a window (design §4.3–4.5).
    /// Capacity uses the GLOBAL calendar (project_id = 0 ⇒ no project overrides);
    /// workload sums the resource's allocations across all projects.
    pub async fn resource_summary(
        pool: &SqlitePool, resource_id: i64, start: &str, end: &str,
    ) -> Result<ResourceSummary, AppError> {
        let cal = db::calendar::hydrate(pool).await?;
        let w = parse_window(start, end)?;
        let rows = AllocationsRepo::list_for_resource(pool, resource_id, start, end).await?;
        let allocs: Vec<domain::Allocation> = rows.iter().map(|(r, pid)| r.to_domain(*pid)).collect();
        let cap = capacity_pd(&cal, 0, resource_id, w); // 0 ⇒ global calendar
        let wl = workload_pd(&cal, &allocs, resource_id, w);
        let util = if cap > 0.0 { wl / cap } else { 0.0 };
        let threshold = effective_overload(pool, resource_id).await?;
        Ok(ResourceSummary {
            resource_id, capacity_pd: cap, workload_pd: wl, utilization: util, overloaded: util > threshold,
        })
    }

    /// All resources whose utilization exceeds their effective threshold (Dashboard alert list).
    pub async fn overloads(pool: &SqlitePool, start: &str, end: &str) -> Result<Vec<ResourceSummary>, AppError> {
        let mut out = Vec::new();
        for r in db::ResourcesRepo::list_active(pool).await? {
            let s = Self::resource_summary(pool, r.id, start, end).await?;
            if s.overloaded { out.push(s); }
        }
        Ok(out)
    }
}

fn parse_window(start: &str, end: &str) -> Result<Window, AppError> {
    let s = NaiveDate::parse_from_str(start, "%Y-%m-%d").map_err(|_| domain::DomainError::InvalidDateWindow)?;
    let e = NaiveDate::parse_from_str(end, "%Y-%m-%d").map_err(|_| domain::DomainError::InvalidDateWindow)?;
    if e < s { return Err(domain::DomainError::InvalidDateWindow.into()); }
    Ok(Window { start: s, end: e })
}
```

- [ ] **Step 2: Write the failing test — `crates/app/tests/workload.rs`**

```rust
use app::service::workload::WorkloadService;
use db::pool::connect;
use db::{AllocationsRepo, ResourcesRepo};

async fn fresh() -> sqlx::SqlitePool {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    pool
}

const MON: &str = "2026-06-29"; const FRI: &str = "2026-07-03"; const WED: &str = "2026-07-01";

#[tokio::test]
async fn resource_summary_half_loaded_with_holiday() {
    let pool = fresh().await;
    sqlx::query("INSERT INTO projects (id,name,start_date,end_date) VALUES (1,'P','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    // global full-day holiday Wed
    sqlx::query("INSERT INTO holiday (project_id,day,fraction,name) VALUES (NULL,'2026-07-01',1.0,'H')").execute(&pool).await.unwrap();
    // Alice 50% Mon..Fri
    AllocationsRepo::create(&pool, 1, 10, MON, FRI, 0.5).await.unwrap();

    let s = WorkloadService::resource_summary(&pool, 1, MON, FRI).await.unwrap();
    // capacity = 4.0 (Wed=0); workload = 5*0.5*0.8 = 2.0; utilization = 0.5
    assert!((s.capacity_pd - 4.0).abs() < 1e-9);
    assert!((s.workload_pd - 2.0).abs() < 1e-9);
    assert!((s.utilization - 0.5).abs() < 1e-9);
    assert!(!s.overloaded); // 0.5 < 1.10
}

#[tokio::test]
async fn detects_overload_with_two_full_allocations() {
    let pool = fresh().await;
    sqlx::query("INSERT INTO projects (id,name) VALUES (1,'P')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO projects (id,name) VALUES (2,'Q')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (11,2,'U','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    AllocationsRepo::create(&pool, 1, 10, MON, FRI, 1.0).await.unwrap(); // 100% on P
    AllocationsRepo::create(&pool, 1, 11, MON, FRI, 1.0).await.unwrap(); // 100% on Q -> 200%

    let s = WorkloadService::resource_summary(&pool, 1, MON, FRI).await.unwrap();
    assert!(s.utilization > 1.0);
    assert!(s.overloaded);

    let ov = WorkloadService::overloads(&pool, MON, FRI).await.unwrap();
    assert_eq!(ov.len(), 1);
    assert_eq!(ov[0].resource_id, 1);
}

#[tokio::test]
async fn bad_window_rejected() {
    let pool = fresh().await;
    let err = WorkloadService::resource_summary(&pool, 1, "2026-07-05", "2026-06-01").await.unwrap_err();
    assert_eq!(err.code, "VALIDATION");
}
```

- [ ] **Step 3: Run test — verify PASS**

Run: `cargo test -p app --test workload`
Expected: `3 passed`.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(app): WorkloadService resource summary + overload detection"
```

---

## Task 4: WorkloadService — team summary + project burn (TDD)

**Files:**
- Modify: `crates/app/src/service/workload.rs`
- Modify: `crates/app/tests/workload.rs`

- [ ] **Step 1: Append to `crates/app/src/service/workload.rs`**

```rust
use db::{ProjectsRepo, TeamMembersRepo};
use domain::team_utilization;

#[derive(Debug, Serialize)]
pub struct TeamSummary {
    pub team_id: i64,
    pub capacity_pd: f64,
    pub workload_pd: f64,
    pub utilization: f64,
    pub overloaded_members: Vec<i64>,
}

#[derive(Debug, Serialize)]
pub struct ProjectBurn {
    pub project_id: i64,
    pub budget_pd: f64,
    pub allocated_pd: f64,
    pub usage: f64, // allocated / budget (0 if budget 0)
}

impl WorkloadService {
    /// Team utilization = Σ workload / Σ capacity over members (design §4.9 team_utilization).
    /// Also lists members whose individual utilization exceeds their threshold.
    pub async fn team_summary(
        pool: &SqlitePool, team_id: i64, start: &str, end: &str,
    ) -> Result<TeamSummary, AppError> {
        let cal = db::calendar::hydrate(pool).await?;
        let w = parse_window(start, end)?;
        let members = TeamMembersRepo::list_members(pool, team_id).await?;
        let ids: Vec<i64> = members.iter().map(|m| m.resource_id).collect();

        let mut total_wl = 0.0; let mut total_cap = 0.0; let mut overloaded = Vec::new();
        for &rid in &ids {
            let rows = AllocationsRepo::list_for_resource(pool, rid, start, end).await?;
            let allocs: Vec<domain::Allocation> = rows.iter().map(|(r, pid)| r.to_domain(*pid)).collect();
            let cap = capacity_pd(&cal, 0, rid, w);
            let wl = workload_pd(&cal, &allocs, rid, w);
            total_wl += wl; total_cap += cap;
            let util = if cap > 0.0 { wl / cap } else { 0.0 };
            if util > effective_overload(pool, rid).await? { overloaded.push(rid); }
        }
        let util = if total_cap > 0.0 { total_wl / total_cap } else { 0.0 };
        Ok(TeamSummary { team_id, capacity_pd: total_cap, workload_pd: total_wl, utilization: util, overloaded_members: overloaded })
    }

    /// Simple project burn: sum of allocated_pd (full-span) vs budget_pd (design §8 R3).
    /// NOTE: allocated_pd is the full-span cached value (design §3.3.15 caveat); a windowed
    /// burn is a Phase 5 report concern. For the Dashboard health card this is sufficient.
    pub async fn project_burn(pool: &SqlitePool, project_id: i64) -> Result<ProjectBurn, AppError> {
        let project = ProjectsRepo::get(pool, project_id).await?;
        let (allocated,): (f64,) = sqlx::query_as(
            "SELECT COALESCE(SUM(a.allocated_pd),0) FROM allocations a \
             JOIN tasks t ON t.id = a.task_id WHERE t.project_id = ? AND a.deleted_at IS NULL")
            .bind(project_id).fetch_one(pool).await?;
        let usage = if project.budget_pd > 0.0 { allocated / project.budget_pd } else { 0.0 };
        Ok(ProjectBurn { project_id, budget_pd: project.budget_pd, allocated_pd: allocated, usage })
    }
}
```

- [ ] **Step 2: Append tests to `crates/app/tests/workload.rs`**

```rust
use app::service::projects::ProjectsService;
use app::service::teams::TeamsService;
use db::models::TeamOverride;

#[tokio::test]
async fn team_summary_aggregates_members() {
    let pool = fresh().await;
    sqlx::query("INSERT INTO projects (id,name) VALUES (1,'P')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'A')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (2,'B')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    // A overloaded (100% on two tasks), B idle
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (11,1,'U','2026-06-01','2026-07-31')").execute(&pool).await.unwrap();
    AllocationsRepo::create(&pool, 1, 10, MON, FRI, 1.0).await.unwrap();
    AllocationsRepo::create(&pool, 1, 11, MON, FRI, 1.0).await.unwrap();

    let tid = TeamsService::create(&pool, "Eng", None).await.unwrap();
    TeamsService::add_member(&pool, tid, 1, None).await.unwrap();
    TeamsService::add_member(&pool, tid, 2, None).await.unwrap();

    let s = WorkloadService::team_summary(&pool, tid, MON, FRI).await.unwrap();
    // capacity = 5 (A) + 5 (B) = 10; workload = 10 (A) + 0 (B) = 10; util = 1.0
    assert!((s.capacity_pd - 10.0).abs() < 1e-9);
    assert!((s.workload_pd - 10.0).abs() < 1e-9);
    assert!((s.utilization - 1.0).abs() < 1e-9);
    assert_eq!(s.overloaded_members, vec![1]); // A > 1.10? util=2.0 > 1.10 yes; B=0 no
}

#[tokio::test]
async fn project_burn_ratio() {
    let pool = fresh().await;
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 40.0).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'A')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,?,'T','2026-06-01','2026-07-31')").bind(pid).execute(&pool).await.unwrap();
    // 100% for 5 workdays -> allocated_pd = 5 * 1.0 * 1.0 = 5 (full-span cached)
    AllocationsRepo::create(&pool, 1, 10, MON, FRI, 1.0).await.unwrap();
    let b = WorkloadService::project_burn(&pool, pid).await.unwrap();
    assert!((b.allocated_pd - 5.0).abs() < 1e-9);
    assert!((b.usage - (5.0 / 40.0)).abs() < 1e-9);
}
```

- [ ] **Step 3: Run test — verify PASS**

Run: `cargo test -p app --test workload`
Expected: `5 passed` (3 prior + team_summary + project_burn).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(app): WorkloadService team summary + project burn"
```

---

## Task 5: Commands + integration dashboard test

**Files:**
- Modify: `crates/app/src/command.rs`

- [ ] **Step 1: Add workload + calendar commands — append to `crates/app/src/command.rs`**

```rust
use crate::service::workload::{ProjectBurn, ResourceSummary, TeamSummary, WorkloadService};
use db::models::{Holiday, TimeOff, WeekTemplate};
use db::{HolidayRepo, TimeOffRepo, WeekTemplateRepo};

#[tauri::command]
pub async fn resource_summary(state: tauri::State<'_, AppState>, resource_id: i64, start: String, end: String) -> Result<ResourceSummary, AppError> {
    WorkloadService::resource_summary(&state.pool, resource_id, &start, &end).await
}
#[tauri::command]
pub async fn team_summary(state: tauri::State<'_, AppState>, team_id: i64, start: String, end: String) -> Result<TeamSummary, AppError> {
    WorkloadService::team_summary(&state.pool, team_id, &start, &end).await
}
#[tauri::command]
pub async fn overloads(state: tauri::State<'_, AppState>, start: String, end: String) -> Result<Vec<ResourceSummary>, AppError> {
    WorkloadService::overloads(&state.pool, &start, &end).await
}
#[tauri::command]
pub async fn project_burn(state: tauri::State<'_, AppState>, project_id: i64) -> Result<ProjectBurn, AppError> {
    WorkloadService::project_burn(&state.pool, project_id).await
}

// ---- calendar management ----
#[tauri::command]
pub async fn set_global_work_week(state: tauri::State<'_, AppState>, week: Vec<f64>) -> Result<(), AppError> {
    if week.len() != 7 { return Err(domain::DomainError::InvalidRatio(week.len() as f64).into()); }
    let arr: [f64; 7] = [week[0], week[1], week[2], week[3], week[4], week[5], week[6]];
    Ok(WeekTemplateRepo::upsert_global(&state.pool, arr).await?)
}
#[tauri::command]
pub async fn list_work_weeks(state: tauri::State<'_, AppState>) -> Result<Vec<WeekTemplate>, AppError> {
    Ok(WeekTemplateRepo::list(&state.pool).await?)
}
#[tauri::command]
pub async fn add_holiday(state: tauri::State<'_, AppState>, project_id: Option<i64>, day: String, fraction: Option<f64>, name: Option<String>) -> Result<i64, AppError> {
    Ok(HolidayRepo::add(&state.pool, project_id, &day, fraction.unwrap_or(1.0), name.as_deref()).await?)
}
#[tauri::command]
pub async fn list_holidays(state: tauri::State<'_, AppState>) -> Result<Vec<Holiday>, AppError> {
    Ok(HolidayRepo::list(&state.pool).await?)
}
#[tauri::command]
pub async fn add_time_off(state: tauri::State<'_, AppState>, resource_id: i64, day: String, fraction: Option<f64>, reason: Option<String>) -> Result<i64, AppError> {
    Ok(TimeOffRepo::add(&state.pool, resource_id, &day, fraction.unwrap_or(1.0), reason.as_deref()).await?)
}
```

- [ ] **Step 2: Register the new commands in `src-tauri/src/main.rs` `generate_handler!`**

Add to the `use app::command::{...}` import and the `generate_handler![...]` list:
```
resource_summary, team_summary, overloads, project_burn,
set_global_work_week, list_work_weeks, add_holiday, list_holidays, add_time_off,
```

- [ ] **Step 3: Build-check the whole workspace**

Run: `cargo build --workspace`
Expected: clean.

- [ ] **Step 4: Run the full suite — verify PASS**

Run: `cargo test --workspace`
Expected: all prior tests + Phase 2 (db calendar 2, app thresholds 1, app workload 5) pass.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(app): workload + calendar commands; Phase 2 backend complete"
```

---

## Self-Review

**Spec coverage (design §4 + roadmap Phase 2 backend):**
- §4.2 calendar three-table model → hydrate (Task 1) ✓
- §4.3 capacity / §4.4 workload / §4.5 utilization & overload → WorkloadService::resource_summary (Task 3) ✓
- §4.6 per-team aggregation → team_summary (Task 4) ✓
- §4.9 pure Rust core as authoritative source → services call domain fn (Tasks 3–4) ✓
- §4.10 golden numbers → workload tests (capacity 4.0 / workload 2.0 / util 0.5; overload 2.0) ✓
- §3.3.8a effective thresholds (team → settings → default) → thresholds resolver (Task 2) ✓
- §8 R3 project burn (budget vs allocated) → project_burn (Task 4) ✓
- §7 Dashboard data needs (resource/team utilization, overload list) → commands (Task 5) ✓

**Deferred (explicitly out of scope, not placeholders):**
- `workload_cache` materialization (§4.7) — on-demand compute is <5ms at MVP scale; cache is a refinement.
- Long-term-task segmentation enforcement, `config_hash` → Phase 3+.
- Windowed project burn (allocated_pd caveat, design §3.3.15) → Phase 5 reports.
- The Phase 2b UI (Dashboard, allocation editor, calendar management screens).

**Placeholder scan:** none. Every code step contains complete code; every test asserts concrete numeric values matching the §4.10 derivation.

**Type consistency:**
- `hydrate()` returns `domain::Calendar` (Phase 0 struct) — fields filled match `Calendar::{global_week, project_weeks, holidays_project, holidays_global, time_off}`.
- `AllocationsRepo::list_for_resource` → `(AllocationRow, project_id)` → `AllocationRow::to_domain(project_id)` → `domain::Allocation` (Phase 0 Task 11 seam) — used identically in Tasks 3 & 4.
- `capacity_pd(&cal, 0, rid, w)` uses project_id `0` ⇒ global calendar (no `project_weeks` entry for id 0 ⇒ `week()` returns `global_week`; `holiday_off(0,d)` ⇒ global) — documented convention.
- `Thresholds` / `ResourceSummary` / `TeamSummary` / `ProjectBurn` Serialize structs match command return types; `effective_overload` signature consistent across Tasks 2–4.
- `parse_window` maps bad input to `DomainError::InvalidDateWindow` → `AppError::Validation` (code `"VALIDATION"`), asserted.

**Known impl-time items (from design, not blockers):**
- `time_off` hydration loads ALL rows (fine at MVP scale ≤10 resources); a window-scoped query is a later optimization.
- Project burn uses the full-span `allocated_pd` cache (design caveat); windowed burn is a Phase 5 report task.
- `team_utilization` domain fn exists but `team_summary` recomputes per-member to also collect `overloaded_members`; the values match the domain fn (verified by the team test: util 1.0).

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-06-27-kanban-phase2-backend.md`. Two execution options:

1. **Subagent-Driven (recommended)** — fresh subagent per task, review between tasks.
2. **Inline Execution** — batch execution with checkpoints.

Which approach? (After Phase 2 backend, the next plan is **Phase 2b: Frontend** — Dashboard with utilization bars + overload alerts + project health, an allocation editor, and calendar-management screens consuming these commands.)
