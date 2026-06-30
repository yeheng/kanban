use crate::error::AppError;
use crate::service::thresholds::{effective_overload, effective_overload_cached};
use chrono::NaiveDate;
use db::repo::calendar::hydrate;
use db::{AllocationsRepo, ResourcesRepo, SettingsRepo};
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
    ///
    /// Denominator policy (cross-project resource summaries, design §4.9): the capacity
    /// denominator uses the GLOBAL calendar (`project_id = 0` ⇒ no project overrides) by
    /// design — a resource's total availability is not project-specific. The workload
    /// numerator sums the resource's allocations across ALL projects, with each
    /// allocation rated against its OWN project's calendar (via `alloc_pd`). This means a
    /// project-level work-week or holiday override affects the *numerator* (that project's
    /// allocation PD) but not the denominator, so utilization reflects the override.
    /// `CalendarOccupancyService` follows the same global-denominator policy (see
    /// `occupancy.rs`), keeping dashboard/calendar/occupancy consistent.
    pub async fn resource_summary(
        pool: &SqlitePool,
        resource_id: i64,
        start: &str,
        end: &str,
    ) -> Result<ResourceSummary, AppError> {
        let cal = hydrate(pool).await?;
        Self::resource_summary_with_cal(pool, &cal, resource_id, start, end).await
    }

    /// Like `resource_summary` but reuses a pre-hydrated calendar to avoid the 3-query
    /// `hydrate()` call per resource when computing many summaries in one request.
    pub async fn resource_summary_with_cal(
        pool: &SqlitePool,
        cal: &domain::Calendar,
        resource_id: i64,
        start: &str,
        end: &str,
    ) -> Result<ResourceSummary, AppError> {
        let resource = ResourcesRepo::get(pool, resource_id).await?;
        let w = parse_window(start, end)?;
        let rows = AllocationsRepo::list_for_resource(pool, resource_id, start, end).await?;
        let allocs: Vec<domain::Allocation> = rows.iter().map(|r| r.to_domain()).collect();
        let threshold = effective_overload(pool, resource_id).await?;
        Ok(Self::summarize(
            cal,
            &allocs,
            resource_id,
            resource.daily_capacity_pd,
            w,
            threshold,
        ))
    }

    /// All resources whose utilization exceeds their effective threshold (Dashboard alert list).
    /// The calendar and global thresholds are hydrated ONCE (not per resource) to avoid
    /// redundant DB queries at scale (design §4.9 <5ms target).
    pub async fn overloads(
        pool: &SqlitePool,
        start: &str,
        end: &str,
    ) -> Result<Vec<ResourceSummary>, AppError> {
        let cal = hydrate(pool).await?;
        let global_threshold = SettingsRepo::thresholds(pool).await?.overload;
        let w = parse_window(start, end)?;
        let mut out = Vec::new();
        for r in db::ResourcesRepo::list_active(pool).await? {
            let rows = AllocationsRepo::list_for_resource(pool, r.id, start, end).await?;
            let allocs: Vec<domain::Allocation> = rows.iter().map(|row| row.to_domain()).collect();
            let threshold = effective_overload_cached(pool, r.id, global_threshold).await?;
            let s = Self::summarize(&cal, &allocs, r.id, r.daily_capacity_pd, w, threshold);
            if s.overloaded {
                out.push(s);
            }
        }
        Ok(out)
    }

    /// Pure aggregation shared by `resource_summary` and `overloads`: no DB access, so it
    /// can be called in a loop without re-hydrating the calendar.
    fn summarize(
        cal: &domain::Calendar,
        allocs: &[domain::Allocation],
        resource_id: i64,
        daily_capacity_pd: f64,
        w: Window,
        threshold: f64,
    ) -> ResourceSummary {
        let cap = capacity_pd(cal, 0, resource_id, daily_capacity_pd, w); // 0 ⇒ global calendar
        let wl = workload_pd(cal, allocs, resource_id, w);
        let util = if cap > 0.0 { wl / cap } else { 0.0 };
        ResourceSummary {
            resource_id,
            capacity_pd: cap,
            workload_pd: wl,
            utilization: util,
            overloaded: util > threshold,
        }
    }
}

fn parse_window(start: &str, end: &str) -> Result<Window, AppError> {
    let s = NaiveDate::parse_from_str(start, "%Y-%m-%d")
        .map_err(|_| domain::DomainError::InvalidDateWindow)?;
    let e = NaiveDate::parse_from_str(end, "%Y-%m-%d")
        .map_err(|_| domain::DomainError::InvalidDateWindow)?;
    if e < s {
        return Err(domain::DomainError::InvalidDateWindow.into());
    }
    Ok(Window { start: s, end: e })
}

use db::{ProjectsRepo, TeamMembersRepo, TeamsRepo};

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
        pool: &SqlitePool,
        team_id: i64,
        start: &str,
        end: &str,
    ) -> Result<TeamSummary, AppError> {
        TeamsRepo::get(pool, team_id).await?;
        let cal = hydrate(pool).await?;
        let global_threshold = SettingsRepo::thresholds(pool).await?.overload;
        let w = parse_window(start, end)?;
        let members = TeamMembersRepo::list_members(pool, team_id).await?;
        let ids: Vec<i64> = members.iter().map(|m| m.resource_id).collect();

        let mut total_wl = 0.0;
        let mut total_cap = 0.0;
        let mut overloaded = Vec::new();
        for &rid in &ids {
            let resource = ResourcesRepo::get(pool, rid).await?;
            let rows = AllocationsRepo::list_for_resource(pool, rid, start, end).await?;
            let allocs: Vec<domain::Allocation> = rows.iter().map(|r| r.to_domain()).collect();
            let cap = capacity_pd(&cal, 0, rid, resource.daily_capacity_pd, w);
            let wl = workload_pd(&cal, &allocs, rid, w);
            total_wl += wl;
            total_cap += cap;
            let util = if cap > 0.0 { wl / cap } else { 0.0 };
            if util > effective_overload_cached(pool, rid, global_threshold).await? {
                overloaded.push(rid);
            }
        }
        let util = if total_cap > 0.0 {
            total_wl / total_cap
        } else {
            0.0
        };
        Ok(TeamSummary {
            team_id,
            capacity_pd: total_cap,
            workload_pd: total_wl,
            utilization: util,
            overloaded_members: overloaded,
        })
    }

    /// Project burn: allocated PD vs budget PD (design §8 R3).
    ///
    /// Allocated PD is computed dynamically with the Phase 0 pure `alloc_pd` over each
    /// allocation's FULL span (capacity from the project's calendar), NOT read from the
    /// `allocations.allocated_pd` cache column — that column defaults to 0 and is never
    /// populated by the current write path, so SUM(allocated_pd) would always be 0.
    /// A windowed burn (clipped to a reporting window) is a Phase 5 report concern.
    pub async fn project_burn(pool: &SqlitePool, project_id: i64) -> Result<ProjectBurn, AppError> {
        let project = ProjectsRepo::get(pool, project_id).await?;
        let cal = hydrate(pool).await?;
        // All active allocations on this project's tasks, with their full spans.
        let rows: Vec<(i64, i64, f64, NaiveDate, NaiveDate, f64)> = sqlx::query_as(
            "SELECT a.resource_id, a.task_id, r.daily_capacity_pd, a.start_date, a.end_date, a.percent \
             FROM allocations a \
             JOIN tasks t ON t.id = a.task_id \
             JOIN resources r ON r.id = a.resource_id \
             WHERE t.project_id = ? AND a.deleted_at IS NULL AND t.deleted_at IS NULL",
        )
        .bind(project_id)
        .fetch_all(pool)
        .await?;
        let mut allocated = 0.0;
        for (resource_id, _task_id, daily_capacity_pd, start, end, percent) in rows {
            let a = domain::Allocation {
                id: 0,
                resource_id,
                project_id,
                daily_capacity_pd,
                start,
                end,
                percent,
            };
            // Full-span window: overlap() with the same window is the whole span.
            allocated += domain::alloc_pd(&cal, &a, Window { start, end });
        }
        let usage = if project.budget_pd > 0.0 {
            allocated / project.budget_pd
        } else {
            0.0
        };
        Ok(ProjectBurn {
            project_id,
            budget_pd: project.budget_pd,
            allocated_pd: allocated,
            usage,
        })
    }
}
