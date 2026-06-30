use crate::error::AppError;
use crate::service::thresholds::{band, effective_thresholds_map};
use chrono::NaiveDate;
use db::models::DayOccupancy;
use db::SettingsRepo;
use domain::{capacity_pd, workload_pd, Window};
use sqlx::SqlitePool;

pub struct CalendarOccupancyService;

impl CalendarOccupancyService {
    /// Per-day per-resource workload/capacity/utilization across [start, end]
    /// (for the calendar occupancy grid). Skips non-working days (zero capacity).
    /// Reuses Phase 2 `hydrate()` + Phase 0 per-day math, so per-day values are
    /// consistent with the window aggregates in `WorkloadService`. Each day's color
    /// band uses the resource's EFFECTIVE (per-team) thresholds, batched once (no N+1).
    pub async fn range(
        pool: &SqlitePool, start: &str, end: &str,
    ) -> Result<Vec<DayOccupancy>, AppError> {
        let cal = db::repo::calendar::hydrate(pool).await?;
        let s = NaiveDate::parse_from_str(start, "%Y-%m-%d").map_err(|_| domain::DomainError::InvalidDateWindow)?;
        let e = NaiveDate::parse_from_str(end, "%Y-%m-%d").map_err(|_| domain::DomainError::InvalidDateWindow)?;
        if e < s { return Err(domain::DomainError::InvalidDateWindow.into()); }

        let resources = db::ResourcesRepo::list_active(pool).await?;
        let ids: Vec<i64> = resources.iter().map(|r| r.id).collect();
        let thr_map = effective_thresholds_map(pool, &ids).await?;
        let global = SettingsRepo::thresholds(pool).await?;

        let mut out = Vec::new();
        for r in resources {
            let thr = thr_map.get(&r.id).unwrap_or(&global);
            // Load this resource's allocations once (shared across all days in range).
            let rows = db::AllocationsRepo::list_for_resource(pool, r.id, start, end).await?;
            let allocs: Vec<domain::Allocation> = rows.iter().map(|row| row.to_domain()).collect();
            let mut d = s;
            while d <= e {
                let w = Window { start: d, end: d };
                let cap = capacity_pd(&cal, 0, r.id, r.daily_capacity_pd, w); // 0 ⇒ global calendar
                if cap > 0.0 {
                    let wl = workload_pd(&cal, &allocs, r.id, w);
                    let util = wl / cap;
                    out.push(DayOccupancy {
                        date: d, resource_id: r.id, resource_name: r.name.clone(),
                        workload_pd: wl, capacity_pd: cap,
                        utilization: util,
                        status: band(util, thr).to_string(),
                    });
                }
                d = d.succ_opt().unwrap();
            }
        }
        Ok(out)
    }
}
