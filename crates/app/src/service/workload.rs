use crate::error::AppError;
use crate::service::thresholds::effective_overload;
use chrono::NaiveDate;
use db::repo::calendar::hydrate;
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
        let cal = hydrate(pool).await?;
        let w = parse_window(start, end)?;
        let rows = AllocationsRepo::list_for_resource(pool, resource_id, start, end).await?;
        let allocs: Vec<domain::Allocation> = rows.iter().map(|r| r.to_domain()).collect();
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
