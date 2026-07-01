# HR Kanban — Phase 0: Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the tested Rust foundation (workspace, full SQLite schema + migrations, connection/transaction layer, and the pure workload/capacity calculation engine) that every later phase depends on.

**Architecture:** A Cargo workspace with two crates: `domain` (pure types + the workload/capacity core, zero I/O, zero serde on errors) and `db` (sqlx persistence: migrations, pool, `with_write_tx`, entity models, repositories). The workload engine is a set of pure functions over an in-memory `Calendar`, verified by golden numeric tests derived from design §4.10. No Tauri/UI/AI yet — everything is verifiable via `cargo test`.

**Tech Stack:** Rust (edition 2021), `sqlx` (SQLite, `bundled`, runtime queries + `migrate!`), `libsqlite3-sys` (bundled static SQLite), `chrono` (dates), `tokio` (async runtime, used by sqlx), `thiserror` (domain errors). Schema + workload formulas are taken verbatim from the finalized design doc `docs/design/2026-06-27-kanban-design.md` (§3.3 DDL, §4.9 workload core).

**Scope note:** This plan covers Phase 0 only. Phase 1 (Tauri shell + CRUD commands + Kanban UI), Phase 2 (allocations UI + Dashboard), Phase 3 (Gantt + calendar), Phase 4 (AI engine), Phase 5 (reports), Phase 6 (polish) are separate plans. Repositories for entities beyond `resources`/`allocations` (teams, projects, tasks, skills, tags, calendar tables) are also deferred to Phase 1 — this plan establishes the schema and the repo *pattern* with the two most critical repos.

**Reference design:** `docs/design/2026-06-27-kanban-design.md`

---

## File Structure

```
kanban/
├── Cargo.toml                  # workspace root
├── rust-toolchain.toml         # pin toolchain
├── .gitignore
├── crates/
│   ├── domain/                 # pure, no I/O, no serde on errors
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # re-exports
│   │       ├── unit.rs         # UnitConfig (PD↔hours↔PM)
│   │       ├── calendar.rs     # Calendar + day_factor (L1∧L2∧L3)
│   │       ├── workload.rs     # capacity_pd / alloc_pd / workload_pd / utilization
│   │       ├── types.rs        # Allocation, Window, DayFraction
│   │       └── error.rs        # DomainError (thiserror, NO serde)
│   └── db/                     # sqlx persistence
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── error.rs        # DbError
│       │   ├── pool.rs         # connect + PRAGMAs
│       │   ├── tx.rs           # with_write_tx (BEGIN IMMEDIATE + busy_timeout + retry)
│       │   ├── models.rs       # entity structs (FromRow)
│       │   └── repo/
│       │       ├── mod.rs
│       │       ├── resources.rs
│       │       └── allocations.rs
│       └── migrations/
│           └── 0001_init.sql   # full schema from design §3.3
└── docs/                       # (already exists: design + this plan)
```

**Responsibilities:** `domain` holds the workload math and domain types — it must compile and test with no database. `db` owns the SQLite schema, connection health, transaction discipline, and repositories. `db` depends on `domain`; `domain` depends on nothing in this workspace.

---

## Task 1: Workspace & toolchain

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `.gitignore`

- [ ] **Step 1: Initialize git repo**

```bash
cd /Users/yeheng/workspaces/Github/kanban
git init
git config user.email "dev@example.com" 2>/dev/null || true
git config user.name "HR Kanban" 2>/dev/null || true
```

- [ ] **Step 2: Create `.gitignore`**

```gitignore
/target
**/*.rs.bk
*.db
*.db-shm
*.db-wal
.env
.DS_Store
```

- [ ] **Step 3: Create `rust-toolchain.toml`**

```toml
[toolchain]
channel = "1.82"
components = ["rustfmt", "clippy"]
profile = "minimal"
```

- [ ] **Step 4: Create workspace `Cargo.toml`**

```toml
[workspace]
resolver = "2"
members = ["crates/domain", "crates/db"]

[workspace.package]
edition = "2021"
license = "MIT OR Apache-2.0"

[workspace.dependencies]
chrono = { version = "0.4", features = ["clock"], default-features = false }
thiserror = "1"
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 5: Verify workspace resolves (empty crates will error — that's expected next)**

```bash
mkdir -p crates/domain/src crates/db/src
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "chore: init cargo workspace skeleton"
```

---

## Task 2: `domain` crate — `UnitConfig` (TDD)

**Files:**
- Create: `crates/domain/Cargo.toml`
- Create: `crates/domain/src/lib.rs`
- Create: `crates/domain/src/unit.rs`
- Test: `crates/domain/src/unit.rs` (inline `#[cfg(test)]`)

- [ ] **Step 1: Create `crates/domain/Cargo.toml`**

```toml
[package]
name = "domain"
version = "0.1.0"
edition.workspace = true

[dependencies]
chrono = { workspace = true }
thiserror = { workspace = true }
```

- [ ] **Step 2: Write the failing test in `crates/domain/src/unit.rs`**

```rust
use chrono::NaiveDate;

/// PD↔hours↔PM conversion constants (design §4.1). Configurable; defaults 1 PD = 8h, 1 PM = 20 PD.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitConfig {
    pub hours_per_pd: f64,   // default 8.0
    pub pd_per_pm: f64,      // default 20.0
}

impl UnitConfig {
    pub const DEFAULT: Self = Self { hours_per_pd: 8.0, pd_per_pm: 20.0 };

    pub fn pd_to_hours(self, pd: f64) -> f64 { pd * self.hours_per_pd }
    pub fn hours_to_pd(self, h: f64) -> f64 { h / self.hours_per_pd }
    pub fn pd_to_pm(self, pd: f64) -> f64 { pd / self.pd_per_pm }
    pub fn pm_to_pd(self, pm: f64) -> f64 { pm * self.pd_per_pm }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pd_hours_roundtrip() {
        let u = UnitConfig::DEFAULT;
        assert!((u.pd_to_hours(1.0) - 8.0).abs() < 1e-9);
        assert!((u.hours_to_pd(8.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn pm_pd_roundtrip() {
        let u = UnitConfig::DEFAULT;
        assert!((u.pm_to_pd(1.0) - 20.0).abs() < 1e-9);
        assert!((u.pd_to_pm(20.0) - 1.0).abs() < 1e-9);
    }
}
```

- [ ] **Step 3: Create `crates/domain/src/lib.rs`**

```rust
pub mod unit;
pub use unit::UnitConfig;
```

- [ ] **Step 4: Run test — verify PASS**

Run: `cargo test -p domain unit`
Expected: `running 2 tests ... 2 passed`.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(domain): UnitConfig PD/PM/hours conversion"
```

---

## Task 3: `domain` crate — `Calendar` + `day_factor` (TDD, golden)

Implements the L1∧L2∧L3 calendar factor (design §3.3.9 / §4.2): base = work-week fraction for the weekday (project template overrides global); a full holiday/leave zeroes the day; a half (fraction 0.5) halves it. Holidays are looked up project-level-first then global; holiday takes precedence over `time_off` per the design's sequential pseudocode.

**Files:**
- Create: `crates/domain/src/types.rs`
- Create: `crates/domain/src/calendar.rs`
- Modify: `crates/domain/src/lib.rs`
- Modify: `crates/domain/Cargo.toml` (add `std::collections` via alloc — already in std)

- [ ] **Step 1: Create `crates/domain/src/types.rs`**

```rust
use chrono::NaiveDate;

/// Per-weekday capacity fraction (from work_week_template). 0.0 = non-working day.
/// Index: 0=Mon .. 6=Sun (chrono `.weekday().num_days_from_monday()`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DayFraction {
    pub days: [f64; 7],
}

impl DayFraction {
    /// Standard Mon–Fri full-time week.
    pub const MON_FRI: Self = Self { days: [1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0] };

    pub fn at(&self, weekday_idx: u32) -> f64 {
        self.days[weekday_idx as usize]
    }
}

/// Closed date window `[start, end]` inclusive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Window {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

/// A resource allocation (in-memory form; design §4.9). Carries project_id so that
/// day_factor resolves project-scoped calendars when summing across projects.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Allocation {
    pub id: i64,
    pub resource_id: i64,
    pub project_id: i64,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub percent: f64, // (0.0, 1.0]
}
```

- [ ] **Step 2: Write the failing test in `crates/domain/src/calendar.rs`**

```rust
use crate::types::DayFraction;
use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;

/// In-memory calendar: L1 work-week templates (global + per-project), L2 holidays,
/// L3 time-off. Pure data; hydrated from DB in a later layer.
#[derive(Debug, Clone, Default)]
pub struct Calendar {
    pub global_week: Option<DayFraction>,
    pub project_weeks: HashMap<i64, DayFraction>,
    /// (project_id, day) -> off-fraction; 1.0 = full-day holiday, 0.5 = half.
    pub holidays_project: HashMap<(i64, NaiveDate), f64>,
    /// day -> off-fraction for global holidays (project_id = NULL).
    pub holidays_global: HashMap<NaiveDate, f64>,
    /// (resource_id, day) -> off-fraction; 1.0 = full-day leave.
    pub time_off: HashMap<(i64, NaiveDate), f64>,
}

impl Calendar {
    pub fn global(week: DayFraction) -> Self {
        Self { global_week: Some(week), ..Default::default() }
    }

    fn week(&self, project_id: i64) -> DayFraction {
        self.project_weeks.get(&project_id).copied().unwrap_or_else(|| {
            self.global_week.unwrap_or(DayFraction::MON_FRI)
        })
    }

    fn holiday_off(&self, project_id: i64, d: NaiveDate) -> Option<f64> {
        self.holidays_project
            .get(&(project_id, d))
            .copied()
            .or_else(|| self.holidays_global.get(&d).copied())
    }

    /// Day factor ∈ [0,1] for one resource on one day under one project (design §3.3.9).
    pub fn day_factor(&self, project_id: i64, resource_id: i64, d: NaiveDate) -> f64 {
        let base = self.week(project_id).at(d.weekday().num_days_from_monday());
        if base == 0.0 {
            return 0.0;
        }
        if let Some(off) = self.holiday_off(project_id, d) {
            return base * (1.0 - off); // holiday wins over time_off (design pseudocode)
        }
        if let Some(off) = self.time_off.get(&(resource_id, d)).copied() {
            return base * (1.0 - off);
        }
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate { NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap() }
    // Week: Mon 2026-06-29 .. Sun 2026-07-05
    const MON: &str = "2026-06-29";
    const WED: &str = "2026-07-01"; // weekday Wed

    #[test]
    fn workday_full_factor() {
        let cal = Calendar::global(DayFraction::MON_FRI);
        assert!((cal.day_factor(1, 1, d(MON)) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn weekend_zero() {
        let cal = Calendar::global(DayFraction::MON_FRI);
        assert!((cal.day_factor(1, 1, d("2026-06-28"))).abs() < 1e-9); // Sun
    }

    #[test]
    fn full_holiday_zeroes_day() {
        let mut cal = Calendar::global(DayFraction::MON_FRI);
        cal.holidays_global.insert(d(WED), 1.0); // full-day holiday
        assert!((cal.day_factor(1, 1, d(WED))).abs() < 1e-9);
    }

    #[test]
    fn half_holiday_halves_day() {
        let mut cal = Calendar::global(DayFraction::MON_FRI);
        cal.holidays_global.insert(d(WED), 0.5); // half-day holiday
        assert!((cal.day_factor(1, 1, d(WED)) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn project_holiday_overrides_global() {
        let mut cal = Calendar::global(DayFraction::MON_FRI);
        cal.holidays_global.insert(d(WED), 1.0);     // global full holiday
        cal.holidays_project.insert((7, d(WED)), 0.5); // project 7 half -> wins
        assert!((cal.day_factor(7, 1, d(WED)) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn time_off_halves_when_no_holiday() {
        let mut cal = Calendar::global(DayFraction::MON_FRI);
        cal.time_off.insert((1, d(WED)), 0.5); // half-day leave
        assert!((cal.day_factor(1, 1, d(WED)) - 0.5).abs() < 1e-9);
    }
}
```

- [ ] **Step 3: Update `crates/domain/src/lib.rs`**

```rust
pub mod calendar;
pub mod types;
pub mod unit;

pub use calendar::Calendar;
pub use types::{Allocation, DayFraction, Window};
pub use unit::UnitConfig;
```

- [ ] **Step 4: Run test — verify PASS**

Run: `cargo test -p domain calendar`
Expected: `running 6 tests ... 6 passed`.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat(domain): Calendar day_factor (workweek/holiday/time-off)"
```

---

## Task 4: `domain` crate — `capacity_pd` / `alloc_pd` (TDD)

Implements design §4.9: raw capacity (sum of `day_factor`, 1 working day = 1 PD), calendar-day overlap of two closed windows, and single-allocation PD over a window (`days × percent × avg_day_factor`).

**Files:**
- Create: `crates/domain/src/workload.rs`
- Modify: `crates/domain/src/lib.rs`

- [ ] **Step 1: Write the failing test + impl in `crates/domain/src/workload.rs`**

```rust
use crate::calendar::Calendar;
use crate::types::{Allocation, Window};
use chrono::{Datelike, Days, NaiveDate};

/// Calendar-day count between two inclusive dates (>=1 when a<=b), else 0.
pub fn count_calendar_days(a: NaiveDate, b: NaiveDate) -> i64 {
    if b < a { 0 } else { (b - a).num_days() + 1 }
}

/// Inclusive overlap of two closed windows, or None if disjoint.
pub fn overlap(a: Window, b: Window) -> Option<(NaiveDate, NaiveDate)> {
    let s = a.start.max(b.start);
    let e = a.end.min(b.end);
    if e < s { None } else { Some((s, e)) }
}

/// Mean day-factor across a [start,end] range (capacity/workload share this basis).
fn avg_day_factor(cal: &Calendar, project_id: i64, resource_id: i64,
                  start: NaiveDate, end: NaiveDate) -> f64 {
    let n = count_calendar_days(start, end);
    if n == 0 { return 0.0; }
    let mut sum = 0.0;
    let mut d = start;
    while d <= end {
        sum += cal.day_factor(project_id, resource_id, d);
        d = d.checked_add_days(Days::new(1)).unwrap();
    }
    sum / (n as f64)
}

/// Raw capacity (no percent) in PD for a window (design §4.3). 1 working day = 1 PD.
pub fn capacity_pd(cal: &Calendar, project_id: i64, resource_id: i64, w: Window) -> f64 {
    let mut sum = 0.0;
    let mut d = w.start;
    while d <= w.end {
        sum += cal.day_factor(project_id, resource_id, d);
        d = d.checked_add_days(Days::new(1)).unwrap();
    }
    sum
}

/// Single allocation's PD within a query window (design §4.4).
pub fn alloc_pd(cal: &Calendar, a: &Allocation, w: Window) -> f64 {
    match overlap(Window { start: a.start, end: a.end }, w) {
        None => 0.0,
        Some((os, oe)) => {
            let days = count_calendar_days(os, oe) as f64;
            let avg = avg_day_factor(cal, a.project_id, a.resource_id, os, oe);
            days * a.percent * avg
        }
    }
}

// bring Datelike into scope for .num_days()
use chrono::TimeDelta as _;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DayFraction;

    fn d(s: &str) -> NaiveDate { NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap() }
    fn win(s: &str, e: &str) -> Window { Window { start: d(s), end: d(e) } }
    fn cal() -> Calendar { Calendar::global(DayFraction::MON_FRI) }
    const MON: &str = "2026-06-29";
    const FRI: &str = "2026-07-03";
    const WED: &str = "2026-07-01";

    #[test]
    fn capacity_five_workdays_is_5_pd() {
        let c = cal();
        assert!((capacity_pd(&c, 1, 1, win(MON, FRI)) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn capacity_skips_holiday() {
        let mut c = cal();
        c.holidays_global.insert(d(WED), 1.0); // full holiday Wed
        assert!((capacity_pd(&c, 1, 1, win(MON, FRI)) - 4.0).abs() < 1e-9);
    }

    #[test]
    fn alloc_pd_half_over_full_week() {
        let c = cal();
        let a = Allocation { id: 1, resource_id: 1, project_id: 1, start: d(MON), end: d(FRI), percent: 0.5 };
        // 5 days * 0.5 * avg_day_factor(1.0) = 2.5
        assert!((alloc_pd(&c, &a, win(MON, FRI)) - 2.5).abs() < 1e-9);
    }

    #[test]
    fn alloc_pd_with_half_holiday() {
        let mut c = cal();
        c.holidays_global.insert(d(WED), 0.5); // half holiday Wed -> day_factor 0.5
        let a = Allocation { id: 1, resource_id: 1, project_id: 1, start: d(MON), end: d(FRI), percent: 0.5 };
        // capacity = 4.5; avg_day_factor over 5 days = 4.5/5 = 0.9; alloc = 5*0.5*0.9 = 2.25
        assert!((alloc_pd(&c, &a, win(MON, FRI)) - 2.25).abs() < 1e-9);
    }

    #[test]
    fn alloc_pd_disjoint_window_zero() {
        let c = cal();
        let a = Allocation { id: 1, resource_id: 1, project_id: 1, start: d(MON), end: d(FRI), percent: 1.0 };
        assert!((alloc_pd(&c, &a, win("2026-08-01", "2026-08-05"))).abs() < 1e-9);
    }
}
```

> **Note on `TimeDelta`:** `chrono` exposes `(b - a).num_days()` via the `Datelike`/`Sub` impls; the `use chrono::TimeDelta as _;` line only ensures the trait path compiles across recent chrono versions. If the compiler flags it as unused, drop that line — the `.num_days()` call on `NaiveDate - NaiveDate` works without it.

- [ ] **Step 2: Update `crates/domain/src/lib.rs`**

```rust
pub mod calendar;
pub mod types;
pub mod unit;
pub mod workload;

pub use calendar::Calendar;
pub use types::{Allocation, DayFraction, Window};
pub use unit::UnitConfig;
pub use workload::{alloc_pd, capacity_pd, count_calendar_days, overlap};
```

- [ ] **Step 3: Run test — verify PASS**

Run: `cargo test -p domain workload`
Expected: `running 5 tests ... 5 passed`.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(domain): capacity_pd & alloc_pd with overlap"
```

---

## Task 5: `domain` crate — `workload_pd` / `utilization` (TDD, golden)

Implements cross-project workload summation, utilization (workload / capacity), and team aggregation (design §4.9). The golden test mirrors design §4.10: a resource split across two allocations in one week.

**Files:**
- Modify: `crates/domain/src/workload.rs`

- [ ] **Step 1: Append failing tests + impl to `crates/domain/src/workload.rs`**

Add after `alloc_pd` (inside the same file, outside the test module):

```rust
/// Total workload of one resource across ALL its allocations in a window (design §4.9).
pub fn workload_pd(cal: &Calendar, allocs: &[Allocation], resource_id: i64, w: Window) -> f64 {
    allocs.iter()
        .filter(|a| a.resource_id == resource_id)
        .map(|a| alloc_pd(cal, a, w))
        .sum()
}

/// Utilization for a (project, resource) in a window; >1.0 = overload.
/// Capacity uses that project's calendar; workload sums the resource's allocations
/// regardless of project (cross-project load). 0 capacity -> 0 utilization.
pub fn utilization(cal: &Calendar, allocs: &[Allocation],
                   project_id: i64, resource_id: i64, w: Window) -> f64 {
    let cap = capacity_pd(cal, project_id, resource_id, w);
    if cap <= 0.0 { return 0.0; }
    workload_pd(cal, allocs, resource_id, w) / cap
}

/// Team utilization = Σ workload / Σ capacity over members (design §4.9).
pub fn team_utilization(cal: &Calendar, allocs: &[Allocation],
                        members: &[i64], project_id: i64, w: Window) -> f64 {
    let (wl, cap) = members.iter().fold((0.0_f64, 0.0_f64), |(wl, cap), &r| {
        (wl + workload_pd(cal, allocs, r, w), cap + capacity_pd(cal, project_id, r, w))
    });
    if cap <= 0.0 { 0.0 } else { wl / cap }
}
```

Add these tests inside the existing `#[cfg(test)] mod tests`:

```rust
    #[test]
    fn workload_sums_across_projects() {
        let c = cal();
        // Alice 50% on project 1, 30% on project 2, both over the full Mon–Fri week.
        let a1 = Allocation { id: 1, resource_id: 1, project_id: 1, start: d(MON), end: d(FRI), percent: 0.5 };
        let a2 = Allocation { id: 2, resource_id: 1, project_id: 2, start: d(MON), end: d(FRI), percent: 0.3 };
        let allocs = [a1, a2];
        // 5*0.5*1.0 + 5*0.3*1.0 = 2.5 + 1.5 = 4.0 PD
        assert!((workload_pd(&c, &allocs, 1, win(MON, FRI)) - 4.0).abs() < 1e-9);
    }

    #[test]
    fn utilization_half_loaded() {
        let c = cal();
        let a = Allocation { id: 1, resource_id: 1, project_id: 1, start: d(MON), end: d(FRI), percent: 0.5 };
        // workload 2.5 / capacity 5.0 = 0.5
        assert!((utilization(&c, &[a], 1, 1, win(MON, FRI)) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn utilization_detects_overload() {
        let c = cal();
        // Two full-time allocations on the same resource, same week -> 200% load.
        let a1 = Allocation { id: 1, resource_id: 1, project_id: 1, start: d(MON), end: d(FRI), percent: 1.0 };
        let a2 = Allocation { id: 2, resource_id: 1, project_id: 2, start: d(MON), end: d(FRI), percent: 1.0 };
        assert!(utilization(&c, &[a1, a2], 1, 1, win(MON, FRI)) > 1.0);
    }

    #[test]
    fn team_utilization_aggregates_members() {
        let c = cal();
        let a = Allocation { id: 1, resource_id: 1, project_id: 1, start: d(MON), end: d(FRI), percent: 0.5 };
        // 2 members, only one loaded at 50% -> team = 2.5 / 10.0 = 0.25
        assert!((team_utilization(&c, &[a], &[1, 2], 1, win(MON, FRI)) - 0.25).abs() < 1e-9);
    }
```

- [ ] **Step 2: Update `crates/domain/src/lib.rs` exports**

```rust
pub use workload::{alloc_pd, capacity_pd, count_calendar_days, overlap, workload_pd, utilization, team_utilization};
```

- [ ] **Step 3: Run test — verify PASS**

Run: `cargo test -p domain`
Expected: all domain tests pass (unit + calendar + workload = 17 tests).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(domain): workload_pd, utilization, team_utilization"
```

---

## Task 6: `db` crate — full schema migration (TDD)

Loads the complete schema from design §3.3 as a sqlx migration and asserts it applies cleanly and all core tables exist.

**Files:**
- Create: `crates/db/Cargo.toml`
- Create: `crates/db/src/lib.rs`
- Create: `crates/db/migrations/0001_init.sql`
- Create: `crates/db/tests/migration.rs`

- [ ] **Step 1: Create `crates/db/Cargo.toml`**

```toml
[package]
name = "db"
version = "0.1.0"
edition.workspace = true

[dependencies]
domain = { path = "../domain" }
tokio = { workspace = true }
thiserror = { workspace = true }

[dependencies.sqlx]
version = "0.8"
default-features = false
features = ["runtime-tokio", "sqlite", "macros", "migrate", "chrono"]

[dev-dependencies]
tokio = { workspace = true }
```

- [ ] **Step 2: Create `crates/db/src/lib.rs`**

```rust
pub mod error;
pub mod pool;
pub mod tx;
pub mod models;
pub mod repo;

pub use sqlx::SqlitePool;
```

- [ ] **Step 3: Create the migration `crates/db/migrations/0001_init.sql`**

This is the verbatim schema from design §3.3 (settings, tags, skills, resources, resource_skills, resource_tags, teams, team_members, team_overrides, work_week_template, holiday, time_off, projects, tasks, task_dependencies, task_skill_requirements, task_tags, allocations, allocation triggers, ai_optimization_runs, resource_project_rates) plus seed rows for the single `settings` row and the global work-week template.

```sql
-- 0001_init.sql — full schema (design §3.3)

CREATE TABLE settings (
    id                  INTEGER PRIMARY KEY CHECK (id = 1),
    default_unit        TEXT    NOT NULL DEFAULT 'PD'  CHECK (default_unit IN ('PD','PM')),
    pd_hours            REAL    NOT NULL DEFAULT 8.0    CHECK (pd_hours > 0),
    pm_workdays         REAL    NOT NULL DEFAULT 20.0   CHECK (pm_workdays > 0),
    ai_provider         TEXT    NOT NULL DEFAULT 'ollama',
    ai_base_url         TEXT,
    ai_api_key_enc      TEXT,
    secret_store        TEXT    NOT NULL DEFAULT 'keychain' CHECK (secret_store IN ('keychain','encrypted_file')),
    ai_chat_model       TEXT    NOT NULL DEFAULT 'qwen2.5:7b',
    ai_embed_model      TEXT    NOT NULL DEFAULT 'nomic-embed-text',
    ai_embed_dim        INTEGER NOT NULL DEFAULT 768,
    solver_backend      TEXT    NOT NULL DEFAULT 'good_lp' CHECK (solver_backend IN ('good_lp','greedy','hungarian')),
    solver_timeout_ms   INTEGER NOT NULL DEFAULT 5000,
    locale              TEXT    NOT NULL DEFAULT 'zh-CN',
    updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
INSERT INTO settings (id) VALUES (1);

CREATE TABLE tags (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,
    color       TEXT,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_tags_name ON tags(name);

CREATE TABLE skills (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_skills_name ON skills(name);

CREATE TABLE resources (
    id                          INTEGER PRIMARY KEY AUTOINCREMENT,
    name                        TEXT    NOT NULL,
    email                       TEXT,
    available_from              TEXT,
    available_to                TEXT,
    status                      TEXT    NOT NULL DEFAULT 'active' CHECK (status IN ('active','inactive','archived')),
    daily_capacity_pd           REAL    NOT NULL DEFAULT 1.0 CHECK (daily_capacity_pd > 0),
    daily_rate_pd               REAL,
    max_parallel_tasks_per_day  INTEGER,
    metadata                    TEXT,
    created_at                  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at                  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    deleted_at                  TEXT
);
CREATE INDEX idx_resources_status ON resources(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_name ON resources(name);

CREATE TABLE resource_skills (
    resource_id     INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    skill_id        INTEGER NOT NULL REFERENCES skills(id)    ON DELETE CASCADE,
    proficiency     INTEGER NOT NULL CHECK (proficiency BETWEEN 1 AND 5),
    evidence        TEXT,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    PRIMARY KEY (resource_id, skill_id)
);
CREATE INDEX idx_resource_skills_skill ON resource_skills(skill_id);

CREATE TABLE resource_tags (
    resource_id INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    tag_id      INTEGER NOT NULL REFERENCES tags(id)      ON DELETE CASCADE,
    PRIMARY KEY (resource_id, tag_id)
);
CREATE INDEX idx_resource_tags_tag ON resource_tags(tag_id);

CREATE TABLE teams (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    description TEXT,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    deleted_at  TEXT
);
CREATE UNIQUE INDEX idx_teams_name_active ON teams(name) WHERE deleted_at IS NULL;

CREATE TABLE team_members (
    team_id     INTEGER NOT NULL REFERENCES teams(id)     ON DELETE CASCADE,
    resource_id INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    role        TEXT,
    joined_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    PRIMARY KEY (team_id, resource_id)
);
CREATE INDEX idx_team_members_resource ON team_members(resource_id);

CREATE TABLE team_overrides (
    team_id             INTEGER PRIMARY KEY REFERENCES teams(id) ON DELETE CASCADE,
    pd_hours            REAL    CHECK (pd_hours IS NULL OR pd_hours > 0),
    pm_workdays         REAL    CHECK (pm_workdays IS NULL OR pm_workdays > 0),
    overload_threshold  REAL    CHECK (overload_threshold IS NULL OR overload_threshold > 0),
    underload_threshold REAL    CHECK (underload_threshold IS NULL OR underload_threshold >= 0),
    utilization_green   REAL    CHECK (utilization_green IS NULL OR (utilization_green >= 0 AND utilization_green <= 1.0)),
    utilization_yellow  REAL    CHECK (utilization_yellow IS NULL OR (utilization_yellow >= 0 AND utilization_yellow <= 1.0)),
    updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);

-- NOTE: work_week_template references projects(id); projects is created below.
-- SQLite allows forward FK references, so ordering is fine.

CREATE TABLE work_week_template (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    scope       TEXT    NOT NULL CHECK (scope IN ('global','project')),
    project_id  INTEGER REFERENCES projects(id) ON DELETE CASCADE,
    mon         INTEGER NOT NULL DEFAULT 1 CHECK (mon IN (0,1)),
    tue         INTEGER NOT NULL DEFAULT 1 CHECK (tue IN (0,1)),
    wed         INTEGER NOT NULL DEFAULT 1 CHECK (wed IN (0,1)),
    thu         INTEGER NOT NULL DEFAULT 1 CHECK (thu IN (0,1)),
    fri         INTEGER NOT NULL DEFAULT 1 CHECK (fri IN (0,1)),
    sat         INTEGER NOT NULL DEFAULT 0 CHECK (sat IN (0,1)),
    sun         INTEGER NOT NULL DEFAULT 0 CHECK (sun IN (0,1)),
    mon_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (mon_frac > 0 AND mon_frac <= 1.0),
    tue_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (tue_frac > 0 AND tue_frac <= 1.0),
    wed_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (wed_frac > 0 AND wed_frac <= 1.0),
    thu_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (thu_frac > 0 AND thu_frac <= 1.0),
    fri_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (fri_frac > 0 AND fri_frac <= 1.0),
    sat_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (sat_frac > 0 AND sat_frac <= 1.0),
    sun_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (sun_frac > 0 AND sun_frac <= 1.0),
    CHECK ((scope = 'global' AND project_id IS NULL) OR (scope = 'project' AND project_id IS NOT NULL))
);
CREATE UNIQUE INDEX idx_wwt_global ON work_week_template((1)) WHERE scope='global';
CREATE UNIQUE INDEX idx_wwt_project ON work_week_template(project_id) WHERE scope='project';
INSERT OR IGNORE INTO work_week_template (scope, mon,tue,wed,thu,fri,sat,sun) VALUES ('global', 1,1,1,1,1,0,0);

CREATE TABLE holiday (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  INTEGER REFERENCES projects(id) ON DELETE CASCADE,
    day         TEXT    NOT NULL,
    fraction    REAL    NOT NULL DEFAULT 1.0 CHECK (fraction > 0 AND fraction <= 1.0),
    name        TEXT,
    CHECK (length(day) = 10)
);
CREATE INDEX idx_holiday_day ON holiday(day);
CREATE INDEX idx_holiday_project_day ON holiday(project_id, day);

CREATE TABLE time_off (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_id INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    day         TEXT    NOT NULL,
    fraction    REAL    NOT NULL DEFAULT 1.0 CHECK (fraction > 0 AND fraction <= 1.0),
    reason      TEXT,
    note        TEXT,
    CHECK (length(day) = 10)
);
CREATE INDEX idx_time_off_res_day ON time_off(resource_id, day);

CREATE TABLE projects (
    id                          INTEGER PRIMARY KEY AUTOINCREMENT,
    name                        TEXT    NOT NULL,
    description                 TEXT,
    start_date                  TEXT,
    end_date                    TEXT,
    priority                    INTEGER NOT NULL DEFAULT 5 CHECK (priority BETWEEN 1 AND 9),
    budget_pd                   REAL    NOT NULL DEFAULT 0 CHECK (budget_pd >= 0),
    max_parallel_tasks_per_day  INTEGER,
    status                      TEXT    NOT NULL DEFAULT 'planning' CHECK (status IN ('planning','active','on_hold','done','cancelled')),
    metadata                    TEXT,
    created_at                  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at                  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    deleted_at                  TEXT,
    CHECK (end_date IS NULL OR start_date IS NULL OR end_date >= start_date)
);
CREATE INDEX idx_projects_status ON projects(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_projects_dates ON projects(start_date, end_date) WHERE deleted_at IS NULL;

CREATE TABLE tasks (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id      INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    parent_task_id  INTEGER REFERENCES tasks(id) ON DELETE CASCADE,
    title           TEXT    NOT NULL,
    description     TEXT,
    estimate_pd     REAL    NOT NULL DEFAULT 0 CHECK (estimate_pd >= 0),
    start_date      TEXT,
    end_date        TEXT,
    is_long_term    INTEGER NOT NULL DEFAULT 0 CHECK (is_long_term IN (0,1)),
    segment_kind    TEXT    CHECK (segment_kind IN ('milestone','phase','segment') OR segment_kind IS NULL),
    status          TEXT    NOT NULL DEFAULT 'todo' CHECK (status IN ('todo','in_progress','blocked','review','done','cancelled')),
    sort_order      INTEGER NOT NULL DEFAULT 0,
    metadata        TEXT,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    deleted_at      TEXT,
    CHECK (end_date IS NULL OR start_date IS NULL OR end_date >= start_date)
);
CREATE INDEX idx_tasks_project ON tasks(project_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_parent  ON tasks(parent_task_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_status  ON tasks(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_dates   ON tasks(start_date, end_date) WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_longterm ON tasks(is_long_term) WHERE is_long_term = 1 AND deleted_at IS NULL;

CREATE TABLE task_dependencies (
    task_id         INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    predecessor_id  INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    lag_days        INTEGER NOT NULL DEFAULT 0,
    dep_type        TEXT    NOT NULL DEFAULT 'FS' CHECK (dep_type IN ('FS','FF','SS','SF')),
    PRIMARY KEY (task_id, predecessor_id),
    CHECK (task_id <> predecessor_id)
);
CREATE INDEX idx_deps_predecessor ON task_dependencies(predecessor_id);

CREATE TABLE task_skill_requirements (
    task_id             INTEGER NOT NULL REFERENCES tasks(id)  ON DELETE CASCADE,
    skill_id            INTEGER NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    min_proficiency     INTEGER NOT NULL CHECK (min_proficiency BETWEEN 1 AND 5),
    is_mandatory        INTEGER NOT NULL DEFAULT 1 CHECK (is_mandatory IN (0,1)),
    weight              REAL    NOT NULL DEFAULT 1.0 CHECK (weight >= 0),
    PRIMARY KEY (task_id, skill_id)
);
CREATE INDEX idx_task_req_skill ON task_skill_requirements(skill_id);

CREATE TABLE task_tags (
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    tag_id  INTEGER NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (task_id, tag_id)
);
CREATE INDEX idx_task_tags_tag ON task_tags(tag_id);

CREATE TABLE ai_optimization_runs (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    seed                INTEGER NOT NULL,
    objective           TEXT    NOT NULL DEFAULT 'balanced' CHECK (objective IN ('balanced','min_makespan','max_utilization','fairness','skill_fit')),
    scope               TEXT    NOT NULL DEFAULT 'full' CHECK (scope IN ('full','incremental')),
    scope_project_ids   TEXT,
    scope_from          TEXT,
    scope_to            TEXT,
    config_json         TEXT NOT NULL,
    constraints_json    TEXT NOT NULL,
    weights_json        TEXT NOT NULL,
    input_snapshot_json TEXT NOT NULL,
    output_plan_json    TEXT,
    score_overall       REAL,
    score_skill_fit     REAL,
    score_utilization   REAL,
    score_fairness      REAL,
    explanation_md      TEXT,
    provider            TEXT NOT NULL,
    chat_model          TEXT NOT NULL,
    embed_model         TEXT,
    solver_backend      TEXT NOT NULL,
    solver_status       TEXT NOT NULL CHECK (solver_status IN ('optimal','feasible','infeasible','timeout','error')),
    status              TEXT    NOT NULL DEFAULT 'proposed' CHECK (status IN ('proposed','accepted','rejected')),
    applied             INTEGER NOT NULL DEFAULT 0 CHECK (applied IN (0,1)),
    started_at          TEXT NOT NULL,
    finished_at         TEXT,
    duration_ms         INTEGER,
    error_msg           TEXT,
    created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_runs_applied ON ai_optimization_runs(applied, created_at);
CREATE INDEX idx_runs_scope ON ai_optimization_runs(scope_from, scope_to);
CREATE INDEX idx_runs_status ON ai_optimization_runs(status, created_at);

CREATE TABLE allocations (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_id     INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    task_id         INTEGER NOT NULL REFERENCES tasks(id)     ON DELETE CASCADE,
    start_date      TEXT    NOT NULL,
    end_date        TEXT    NOT NULL,
    percent         REAL    NOT NULL CHECK (percent > 0 AND percent <= 1.0),
    allocated_pd    REAL    NOT NULL DEFAULT 0 CHECK (allocated_pd >= 0),
    status          TEXT    NOT NULL DEFAULT 'planned' CHECK (status IN ('planned','committed','in_progress','done','cancelled','locked')),
    source          TEXT    NOT NULL DEFAULT 'manual' CHECK (source IN ('manual','ai')),
    run_id          INTEGER REFERENCES ai_optimization_runs(id) ON DELETE SET NULL,
    note            TEXT,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    deleted_at      TEXT,
    CHECK (end_date >= start_date),
    CHECK (source <> 'ai' OR run_id IS NOT NULL)
);
CREATE INDEX idx_alloc_resource_date ON allocations(resource_id, start_date, end_date) WHERE deleted_at IS NULL;
CREATE INDEX idx_alloc_task ON allocations(task_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_alloc_run ON allocations(run_id) WHERE run_id IS NOT NULL;
CREATE INDEX idx_alloc_status ON allocations(status) WHERE deleted_at IS NULL;

CREATE TRIGGER trg_allocation_validate_insert
AFTER INSERT ON allocations
BEGIN
    SELECT RAISE(ABORT, 'allocation out of task window')
    FROM tasks t
    WHERE t.id = NEW.task_id
      AND t.start_date IS NOT NULL AND t.end_date IS NOT NULL
      AND (NEW.start_date < t.start_date OR NEW.end_date > t.end_date);
    SELECT RAISE(ABORT, 'allocation out of resource availability')
    FROM resources r
    WHERE r.id = NEW.resource_id
      AND r.available_from IS NOT NULL AND r.available_to IS NOT NULL
      AND (NEW.start_date < r.available_from OR NEW.end_date > r.available_to);
    SELECT RAISE(ABORT, 'allocation.percent invalid')
    WHERE NEW.percent <= 0 OR NEW.percent > 1.0;
END;

CREATE TRIGGER trg_allocation_validate_update
AFTER UPDATE OF start_date, end_date, resource_id, task_id, percent ON allocations
BEGIN
    SELECT RAISE(ABORT, 'allocation out of task window')
    FROM tasks t
    WHERE t.id = NEW.task_id
      AND t.start_date IS NOT NULL AND t.end_date IS NOT NULL
      AND (NEW.start_date < t.start_date OR NEW.end_date > t.end_date);
    SELECT RAISE(ABORT, 'allocation out of resource availability')
    FROM resources r
    WHERE r.id = NEW.resource_id
      AND r.available_from IS NOT NULL AND r.available_to IS NOT NULL
      AND (NEW.start_date < r.available_from OR NEW.end_date > r.available_to);
    SELECT RAISE(ABORT, 'allocation.percent invalid')
    WHERE NEW.percent <= 0 OR NEW.percent > 1.0;
END;

CREATE TABLE resource_project_rates (
    resource_id     INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    project_id      INTEGER NOT NULL REFERENCES projects(id)  ON DELETE CASCADE,
    daily_rate_pd   REAL    NOT NULL CHECK (daily_rate_pd > 0),
    valid_from      TEXT,
    valid_to        TEXT,
    note            TEXT,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    PRIMARY KEY (resource_id, project_id, valid_from),
    CHECK (valid_to IS NULL OR valid_from IS NULL OR valid_to >= valid_from)
);
CREATE INDEX idx_rpr_project ON resource_project_rates(project_id);
CREATE INDEX idx_rpr_res_date ON resource_project_rates(resource_id, valid_from, valid_to);
```

> The migration embeds the seed `settings` row and the global `work_week_template` (Mon–Fri) so the DB is usable immediately after migrate.

- [ ] **Step 4: Write the migration test `crates/db/tests/migration.rs`**

```rust
use db::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
async fn migration_creates_all_tables() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let expected = [
        "settings","tags","skills","resources","resource_skills","resource_tags",
        "teams","team_members","team_overrides","work_week_template","holiday","time_off",
        "projects","tasks","task_dependencies","task_skill_requirements","task_tags",
        "allocations","ai_optimization_runs","resource_project_rates",
    ];
    for tbl in expected {
        let exists: (i64,) = sqlx::query_as(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?"
        )
        .bind(tbl)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(exists.0, 1, "table {} missing after migration", tbl);
    }

    // seeded single settings row + global work-week template
    let n_settings: (i64,) = sqlx::query_as("SELECT count(*) FROM settings").fetch_one(&pool).await.unwrap();
    assert_eq!(n_settings.0, 1);
    let n_week: (i64,) = sqlx::query_as("SELECT count(*) FROM work_week_template WHERE scope='global'").fetch_one(&pool).await.unwrap();
    assert_eq!(n_week.0, 1);
}

#[tokio::test]
async fn global_workweek_unique_constraint() {
    let pool = SqlitePoolOptions::new().connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    // inserting a second global row must fail (idx_wwt_global)
    let res = sqlx::query(
        "INSERT INTO work_week_template (scope) VALUES ('global')"
    ).execute(&pool).await;
    assert!(res.is_err(), "second global work_week_template should be rejected");
}
```

- [ ] **Step 5: Create stub modules so the crate compiles**

`crates/db/src/error.rs`:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("not found")]
    NotFound,
}
```

`crates/db/src/pool.rs`:
```rust
use crate::error::DbError;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

/// Connect to a SQLite file (or `:memory:`), unencrypted (design §3.1 PRAGMAs).
/// WAL is skipped for in-memory DBs (unsupported). Delegates to `connect_with_key`
/// with `None`.
pub async fn connect(url: &str) -> Result<SqlitePool, DbError> {
    connect_with_key(url, None).await
}

/// Connect with optional SQLCipher encryption. When `key` is `Some`, `PRAGMA key`
/// is issued **first** (SQLCipher requires the key before any other op) — honoring
/// decision #55 (encryption default-on). The SQLCipher build is enabled via the
/// `db` Cargo.toml (see Phase 1 Task 8); without it, `PRAGMA key` is a harmless no-op
/// on plain SQLite, so headless `:memory:` tests (which pass `None`) are unaffected.
pub async fn connect_with_key(url: &str, key: Option<&str>) -> Result<SqlitePool, DbError> {
    let mut opts = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(std::time::Duration::from_millis(5000));
    if let Some(k) = key {
        opts = opts.pragma("key", k); // SQLCipher key — MUST be the first pragma
    }
    opts = opts
        .pragma("synchronous", "NORMAL")
        .pragma("temp_store", "MEMORY");
    if !url.contains(":memory:") {
        opts = opts.pragma("journal_mode", "WAL").pragma("mmap_size", "268435456");
    }
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(opts)
        .await?;
    Ok(pool)
}
```

`crates/db/src/tx.rs` (stub — implemented in Task 8):
```rust
use crate::error::DbError;
use sqlx::SqlitePool;

/// Placeholder; replaced in Task 8.
pub async fn with_write_tx<F, T>(_pool: &SqlitePool, _f: F) -> Result<T, DbError>
where F: for<'c> FnOnce(&mut sqlx::Transaction<'c, sqlx::Sqlite>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, DbError>> + Send + 'c>> + Send, T: Send {
    Err(DbError::NotFound)
}
```

`crates/db/src/models.rs`:
```rust
// Entity structs added in Task 10.
```

`crates/db/src/repo/mod.rs`:
```rust
// Repos added in Task 10/11.
```

- [ ] **Step 6: Run test — verify PASS**

Run: `cargo test -p db --test migration`
Expected: `2 passed`.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(db): full schema migration 0001_init + seed"
```

---

## Task 7: `db` crate — connection pool + PRAGMAs (TDD)

**Files:**
- Modify: `crates/db/src/pool.rs` (already created in Task 6)
- Create: `crates/db/tests/pool.rs`

- [ ] **Step 1: Write the failing test `crates/db/tests/pool.rs`**

```rust
use db::pool::connect;

#[tokio::test]
async fn connect_sets_pragmas_and_foreign_keys() {
    let pool = connect("sqlite::memory:").await.unwrap();
    // foreign_keys ON
    let fk: (i64,) = sqlx::query_as("PRAGMA foreign_keys").fetch_one(&pool).await.unwrap();
    assert_eq!(fk.0, 1);
    // synchronous NORMAL (= 1)
    let syn: (i64,) = sqlx::query_as("PRAGMA synchronous").fetch_one(&pool).await.unwrap();
    assert_eq!(syn.0, 1);
    // migrations still runnable on the pool
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
}
```

- [ ] **Step 2: Run test — verify PASS**

Run: `cargo test -p db --test pool`
Expected: `1 passed`.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat(db): connect() with foreign_keys/synchronous/busy_timeout PRAGMAs"
```

---

## Task 8: `db` crate — `with_write_tx` (TDD)

Implements the design's unified write-transaction helper (design §6.5 / §3.7): `BEGIN IMMEDIATE` semantics, `busy_timeout`, and application-level `SQLITE_BUSY` backoff retry. sqlite `sqlx` does not expose `BEGIN IMMEDIATE` via `pool.begin()` directly, so we set the option on the connection (already `busy_timeout`) and retry the whole transaction on `SQLITE_BUSY`.

**Files:**
- Modify: `crates/db/src/tx.rs`
- Create: `crates/db/tests/tx.rs`

- [ ] **Step 1: Replace `crates/db/src/tx.rs`**

```rust
use crate::error::DbError;
use sqlx::SqlitePool;
use std::pin::Pin;

/// Run `f` inside a write transaction with busy-retry (design §6.5).
///
/// Uses the canonical **borrowed-transaction callback** pattern so it compiles on
/// stable Rust: `f` receives `&mut Transaction` and returns a pinned boxed future
/// yielding `Result<T, DbError>`. The transaction is NOT returned — `with_write_tx`
/// owns it and commits on `Ok`, drops (rolls back) on `Err`. On `SQLITE_BUSY`
/// (single writer in WAL) the whole tx is retried with backoff up to 3 times.
pub async fn with_write_tx<F, T>(pool: &SqlitePool, f: F) -> Result<T, DbError>
where
    F: for<'c> FnOnce(
            &'c mut sqlx::Transaction<'c, sqlx::Sqlite>,
        ) -> Pin<Box<dyn std::future::Future<Output = Result<T, DbError>> + Send + 'c>>,
    T: Send,
{
    const BACKOFF_MS: [u64; 3] = [50, 100, 200];
    let mut last: Option<DbError> = None;
    for &ms in BACKOFF_MS.iter() {
        let mut tx = pool.begin().await?;
        match f(&mut tx).await {
            Ok(val) => {
                tx.commit().await?;
                return Ok(val);
            }
            Err(DbError::Sqlx(sqlx::Error::Database(e))) if e.is_busy() => {
                last = Some(DbError::Sqlx(sqlx::Error::Database(e)));
                tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
            }
            Err(e) => {
                return Err(e); // tx drops here -> rollback
            }
        }
    }
    Err(last.unwrap_or(DbError::NotFound))
}
```

> **Why borrowed, not owned:** stable Rust cannot easily express an owned-`Transaction` async closure (the `(Transaction, T)` hand-back fights the borrow checker across the boxed future). The `for<'c> FnOnce(&'c mut Transaction<'c,_>) -> Pin<Box<Future + 'c>>` shape is the standard sqlx pattern. Callers use `&mut *tx` (reborrow as `Executor`) and return `Ok(value)`.

- [ ] **Step 2: Write the failing test `crates/db/tests/tx.rs`**

```rust
use db::error::DbError;
use db::pool::connect;
use db::tx::with_write_tx;

#[tokio::test]
async fn tx_commits_on_success() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let id = with_write_tx(&pool, |tx| Box::pin(async move {
        sqlx::query("INSERT INTO skills (name) VALUES (?)")
            .bind("Rust")
            .execute(&mut *tx)
            .await?;
        let row: (i64,) = sqlx::query_as("SELECT count(*) FROM skills").fetch_one(&mut *tx).await?;
        Ok(row.0)
    })).await.unwrap();
    assert_eq!(id, 1);

    // visible after commit on a fresh connection
    let after: (i64,) = sqlx::query_as("SELECT count(*) FROM skills").fetch_one(&pool).await.unwrap();
    assert_eq!(after.0, 1);
}

#[tokio::test]
async fn tx_rolls_back_on_error() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let res = with_write_tx(&pool, |tx| Box::pin(async move {
        sqlx::query("INSERT INTO skills (name) VALUES (?)").bind("Rust").execute(&mut *tx).await?;
        // force a failure -> closure returns Err -> with_write_tx drops tx (rollback)
        sqlx::query("INSERT INTO no_such_table VALUES (1)").execute(&mut *tx).await?;
        Ok(())
    })).await;
    assert!(res.is_err());

    let count: (i64,) = sqlx::query_as("SELECT count(*) FROM skills").fetch_one(&pool).await.unwrap();
    assert_eq!(count, 0, "rollback should leave table empty");
}
```

- [ ] **Step 3: Run test — verify PASS**

Run: `cargo test -p db --test tx`
Expected: `2 passed`.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(db): with_write_tx with busy retry + rollback"
```

---

## Task 9: `db` crate — allocation trigger behavior (TDD)

Verifies the schema-level `trg_allocation_validate_insert` rejects allocations outside the task window (design §3.3.15a).

**Files:**
- Create: `crates/db/tests/trigger.rs`

- [ ] **Step 1: Write the failing test `crates/db/tests/trigger.rs`**

```rust
use db::pool::connect;

async fn setup() -> sqlx::SqlitePool {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    sqlx::query("INSERT INTO projects (id,name,start_date,end_date) VALUES (1,'P','2026-06-01','2026-06-30')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-10','2026-06-20')")
        .execute(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn in_window_allocation_ok() {
    let pool = setup().await;
    let res = sqlx::query(
        "INSERT INTO allocations (resource_id,task_id,start_date,end_date,percent) VALUES (1,10,'2026-06-12','2026-06-18',0.5)"
    ).execute(&pool).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn out_of_task_window_rejected() {
    let pool = setup().await;
    let res = sqlx::query(
        "INSERT INTO allocations (resource_id,task_id,start_date,end_date,percent) VALUES (1,10,'2026-06-05','2026-06-18',0.5)"
    ).execute(&pool).await;
    assert!(res.is_err(), "allocation starting before task window must be aborted by trigger");
}

#[tokio::test]
async fn out_of_resource_availability_rejected() {
    let pool = setup().await;
    sqlx::query("UPDATE resources SET available_from='2026-06-15', available_to='2026-06-25' WHERE id=1")
        .execute(&pool).await.unwrap();
    let res = sqlx::query(
        "INSERT INTO allocations (resource_id,task_id,start_date,end_date,percent) VALUES (1,10,'2026-06-10','2026-06-20',0.5)"
    ).execute(&pool).await;
    assert!(res.is_err(), "allocation outside resource availability must be aborted");
}
```

- [ ] **Step 2: Run test — verify PASS**

Run: `cargo test -p db --test trigger`
Expected: `3 passed`.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "test(db): allocation time-window trigger enforcement"
```

---

## Task 10: `db` crate — entity models + `ResourcesRepo` (TDD)

Establishes the repository pattern with the `resources` entity (design §3.3.4): soft-delete, `FromRow` model, create/list/get/update.

**Files:**
- Modify: `crates/db/src/models.rs`
- Create: `crates/db/src/repo/resources.rs`
- Modify: `crates/db/src/repo/mod.rs`
- Create: `crates/db/tests/resources_repo.rs`

- [ ] **Step 1: Write `crates/db/src/models.rs`**

```rust
use chrono::NaiveDate;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
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
```

- [ ] **Step 2: Write `crates/db/src/repo/resources.rs`**

```rust
use crate::error::DbError;
use crate::models::Resource;
use sqlx::SqlitePool;

pub struct ResourcesRepo;

impl ResourcesRepo {
    pub async fn create(pool: &SqlitePool, name: &str, email: Option<&str>) -> Result<i64, DbError> {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO resources (name, email) VALUES (?, ?) RETURNING id"
        )
        .bind(name)
        .bind(email)
        .fetch_one(pool)
        .await?;
        Ok(row.0)
    }

    pub async fn list_active(pool: &SqlitePool) -> Result<Vec<Resource>, DbError> {
        let rows = sqlx::query_as::<_, Resource>(
            "SELECT id, name, email, available_from, available_to, status, \
             daily_capacity_pd, daily_rate_pd, max_parallel_tasks_per_day, metadata \
             FROM resources WHERE deleted_at IS NULL ORDER BY name"
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn get(pool: &SqlitePool, id: i64) -> Result<Resource, DbError> {
        sqlx::query_as::<_, Resource>(
            "SELECT id, name, email, available_from, available_to, status, \
             daily_capacity_pd, daily_rate_pd, max_parallel_tasks_per_day, metadata \
             FROM resources WHERE id = ? AND deleted_at IS NULL"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(DbError::NotFound)
    }

    pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<(), DbError> {
        let n = sqlx::query(
            "UPDATE resources SET deleted_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id = ? AND deleted_at IS NULL"
        )
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
        if n == 0 { return Err(DbError::NotFound); }
        Ok(())
    }
}
```

- [ ] **Step 3: Update `crates/db/src/repo/mod.rs`**

```rust
pub mod resources;
pub use resources::ResourcesRepo;
```

- [ ] **Step 4: Write the failing test `crates/db/tests/resources_repo.rs`**

```rust
use db::pool::connect;
use db::repo::ResourcesRepo;

#[tokio::test]
async fn create_list_get_softdelete() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let id = ResourcesRepo::create(&pool, "Alice", Some("a@x.com")).await.unwrap();
    assert!(id > 0);

    let list = ResourcesRepo::list_active(&pool).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "Alice");

    let got = ResourcesRepo::get(&pool, id).await.unwrap();
    assert_eq!(got.email.as_deref(), Some("a@x.com"));
    assert!((got.daily_capacity_pd - 1.0).abs() < 1e-9); // default 1.0

    ResourcesRepo::soft_delete(&pool, id).await.unwrap();
    assert!(ResourcesRepo::get(&pool, id).await.is_err());
    let after = ResourcesRepo::list_active(&pool).await.unwrap();
    assert_eq!(after.len(), 0);
}
```

- [ ] **Step 5: Run test — verify PASS**

Run: `cargo test -p db --test resources_repo`
Expected: `1 passed`.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(db): ResourcesRepo (create/list/get/soft_delete)"
```

---

## Task 11: `db` crate — `AllocationsRepo` + workload hydration bridge (TDD)

The allocations repo writes/reads allocations and, crucially, bridges DB rows into `domain::Allocation` so the workload engine (Tasks 3–5) can compute on real persisted data. This is the seam that connects persistence to the pure math.

**Files:**
- Modify: `crates/db/src/models.rs` (add `AllocationRow`)
- Create: `crates/db/src/repo/allocations.rs`
- Modify: `crates/db/src/repo/mod.rs`
- Create: `crates/db/tests/allocations_repo.rs`

- [ ] **Step 1: Append to `crates/db/src/models.rs`**

```rust
#[derive(Debug, Clone, FromRow)]
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
    /// Resolve task -> project, then bridge to the pure `domain::Allocation` used by
    /// the workload engine. `project_id` must be looked up by the caller/repo.
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
```

- [ ] **Step 2: Write `crates/db/src/repo/allocations.rs`**

```rust
use crate::error::DbError;
use crate::models::AllocationRow;
use sqlx::SqlitePool;

pub struct AllocationsRepo;

impl AllocationsRepo {
    /// Insert an allocation. Caller guarantees the (task,resource,window) validity —
    /// the DB trigger (trg_allocation_validate_insert) is the schema-level backstop.
    pub async fn create(
        pool: &SqlitePool,
        resource_id: i64,
        task_id: i64,
        start: &str,
        end: &str,
        percent: f64,
    ) -> Result<i64, DbError> {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO allocations (resource_id, task_id, start_date, end_date, percent) \
             VALUES (?, ?, ?, ?, ?) RETURNING id"
        )
        .bind(resource_id)
        .bind(task_id)
        .bind(start)
        .bind(end)
        .bind(percent)
        .fetch_one(pool)
        .await?;
        Ok(row.0)
    }

    /// All active allocations for a resource overlapping [start, end], joined to the
    /// task's project so they can be bridged to `domain::Allocation`.
    pub async fn list_for_resource(
        pool: &SqlitePool,
        resource_id: i64,
        start: &str,
        end: &str,
    ) -> Result<Vec<(AllocationRow, i64)>, DbError> {
        let rows: Vec<(AllocationRow, i64)> = sqlx::query_as(
            "SELECT a.id, a.resource_id, a.task_id, a.start_date, a.end_date, a.percent, \
                    a.status, a.source, a.run_id, t.project_id AS project_id \
             FROM allocations a JOIN tasks t ON t.id = a.task_id \
             WHERE a.resource_id = ? AND a.deleted_at IS NULL \
               AND a.start_date <= ? AND a.end_date >= ? \
             ORDER BY a.start_date"
        )
        .bind(resource_id)
        .bind(end)   // a.start_date <= window.end
        .bind(start) // a.end_date   >= window.start  => overlap
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}
```

> The query selects `t.project_id` as the 10th column; `sqlx::query_as` maps the first 9 columns into `AllocationRow` via `FromRow` and the 10th (`project_id: i64`) into the tuple's second element. Column order in `SELECT` must match: the 9 `AllocationRow` fields in struct order, then `project_id`.

- [ ] **Step 3: Update `crates/db/src/repo/mod.rs`**

```rust
pub mod allocations;
pub mod resources;
pub use allocations::AllocationsRepo;
pub use resources::ResourcesRepo;
```

- [ ] **Step 4: Write the failing test `crates/db/tests/allocations_repo.rs`**

```rust
use db::pool::connect;
use db::repo::AllocationsRepo;
use domain::{workload_pd, Calendar, DayFraction, Window};
use chrono::NaiveDate;

fn d(s: &str) -> NaiveDate { NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap() }

#[tokio::test]
async fn persist_then_compute_workload() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    // project 1 (Jun), resource 1, task 10 in project 1 (Jun 8-19)
    sqlx::query("INSERT INTO projects (id,name,start_date,end_date) VALUES (1,'P','2026-06-01','2026-06-30')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-08','2026-06-19')").execute(&pool).await.unwrap();

    // 50% allocation Mon 2026-06-08 .. Fri 2026-06-12 (5 workdays)
    let aid = AllocationsRepo::create(&pool, 1, 10, "2026-06-08", "2026-06-12", 0.5).await.unwrap();
    assert!(aid > 0);

    let rows = AllocationsRepo::list_for_resource(&pool, 1, "2026-06-08", "2026-06-12").await.unwrap();
    assert_eq!(rows.len(), 1);
    let (row, project_id) = &rows[0];
    assert_eq!(*project_id, 1);

    // bridge to domain and compute workload for that week
    let cal = Calendar::global(DayFraction::MON_FRI);
    let allocs = vec![row.to_domain(*project_id)];
    let wl = workload_pd(&cal, &allocs, 1, Window { start: d("2026-06-08"), end: d("2026-06-12") });
    assert!((wl - 2.5).abs() < 1e-9); // 5 * 0.5 * 1.0
}

#[tokio::test]
async fn out_of_window_insert_rejected() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    sqlx::query("INSERT INTO projects (id,name) VALUES (1,'P')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tasks (id,project_id,title,start_date,end_date) VALUES (10,1,'T','2026-06-10','2026-06-20')").execute(&pool).await.unwrap();
    let res = AllocationsRepo::create(&pool, 1, 10, "2026-06-01", "2026-06-15", 0.5).await;
    assert!(res.is_err());
}
```

- [ ] **Step 5: Run the full suite — verify PASS**

Run: `cargo test --workspace`
Expected: all tests pass across `domain` (17) and `db` (migration 2 + pool 1 + tx 2 + trigger 3 + resources 1 + allocations 2 = 11).

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(db): AllocationsRepo + domain bridge; foundation complete"
```

---

## Self-Review

**Spec coverage (design §3 / §4 / §6 foundation):**
- §3.1 PRAGMAs (WAL, foreign_keys, busy_timeout, synchronous, mmap) → Task 7 ✓
- §3.3 full DDL (all 20 tables + triggers + indexes) → Task 6 ✓
- §3.3.9 calendar three-table model + day_factor L1∧L2∧L3 → Task 3 ✓
- §3.3.15a allocation validate trigger → Task 9 ✓
- §4.1 UnitConfig → Task 2 ✓
- §4.3–4.5 capacity_pd / alloc_pd / workload_pd / utilization / team_utilization → Tasks 4–5 ✓
- §4.9 pure Rust core → Tasks 3–5 ✓
- §4.10 numeric golden → Tasks 4–5 (2.5 PD, 0.5 utilization, overload detection) ✓
- §6.5 db::with_write_tx (BEGIN/busy_timeout/retry) → Task 8 ✓
- §3.7 soft-delete + single settings row + global work-week seed → Tasks 6, 10 ✓
- DomainError NOT deriving serde (design §2.5/§6.4) → `domain` has no serde dep ✓

**Deferred to Phase 1 (explicitly out of this plan's scope, not placeholders):** repos for teams, projects, tasks, skills, tags, calendar tables, rates; the `effective_*` team-override resolver; `workload_cache` materialization (design §4.7); Tauri shell + commands; UI. Each gets its own task in the Phase 1 plan.

**Placeholder scan:** none. Every code step contains complete code; every test asserts concrete values.

**Type consistency:** `domain::Allocation` fields (`id, resource_id, project_id, start, end, percent`) are used identically in Tasks 3, 4, 5, 11. `AllocationRow::to_domain(project_id)` matches. `Window { start, end }` consistent throughout. `ResourcesRepo`/`AllocationsRepo` method names match across tasks.

**Known impl-time decisions carried forward (from design, not blockers):**
- `config_hash` normalization (design §4.7.2) — deferred with `workload_cache` to Phase 2.
- `team_overrides` effective resolver — deferred to Phase 1 with the teams repo.
- `DomainError → AppError` IPC mapping — deferred to the Tauri layer (Phase 1).

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-06-27-kanban-phase0-foundation.md`. Two execution options:

1. **Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.
2. **Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints.

Which approach?
