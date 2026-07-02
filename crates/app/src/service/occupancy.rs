use crate::error::AppError;
use crate::service::thresholds::{band, effective_thresholds_map};
use chrono::NaiveDate;
use db::models::DayOccupancy;
use db::SettingsRepo;
use domain::{each_day, capacity_pd, workload_pd, Window};
use sqlx::SqlitePool;
use std::collections::HashMap;

pub struct CalendarOccupancyService;

impl CalendarOccupancyService {
    /// Per-day per-resource workload/capacity/utilization across [start, end]
    /// (for the calendar occupancy grid). Skips non-working days (zero capacity).
    /// Reuses Phase 2 `hydrate()` + Phase 0 per-day math, so per-day values are
    /// consistent with the window aggregates in `WorkloadService`. Each day's color
    /// band uses the resource's EFFECTIVE (per-team) thresholds, batched once (no N+1).
    /// Allocations are batch-loaded in one query (no N+1 per resource).
    #[tracing::instrument(skip(pool), fields(start = %start, end = %end))]
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

        // Batch-load all allocations in one query, grouped by resource_id.
        let all_rows = db::AllocationsRepo::list_for_resources(pool, &ids, start, end).await?;
        let mut allocs_by_res: HashMap<i64, Vec<domain::Allocation>> = HashMap::new();
        for row in all_rows {
            allocs_by_res.entry(row.resource_id).or_default().push(row.to_domain());
        }

        let mut out = Vec::new();
        for r in resources {
            let thr = thr_map.get(&r.id).unwrap_or(&global);
            let allocs = allocs_by_res.get(&r.id).map(|v| v.as_slice()).unwrap_or(&[]);
            for d in each_day(s, e) {
                let w = Window { start: d, end: d };
                let cap = capacity_pd(&cal, 0, r.id, r.daily_capacity_pd, w); // 0 ⇒ global calendar
                if cap > 0.0 {
                    let wl = workload_pd(&cal, allocs, r.id, w);
                    let util = wl / cap;
                    out.push(DayOccupancy {
                        date: d, resource_id: r.id, resource_name: r.name.clone(),
                        workload_pd: wl, capacity_pd: cap,
                        utilization: util,
                        status: band(util, thr).to_string(),
                    });
                }
            }
        }
        tracing::info!(resource_count = ids.len(), day_count = out.len(), "computed daily occupancy");
        Ok(out)
    }
}
