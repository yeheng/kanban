use crate::calendar::Calendar;
use crate::types::{Allocation, Window};
use chrono::{Days, NaiveDate};

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

/// Sum of day_factor across a [start,end] range (shared basis for capacity & alloc).
fn sum_day_factors(cal: &Calendar, project_id: i64, resource_id: i64,
                   start: NaiveDate, end: NaiveDate) -> f64 {
    let mut sum = 0.0;
    let mut d = start;
    while d <= end {
        sum += cal.day_factor(project_id, resource_id, d);
        d = d.checked_add_days(Days::new(1)).unwrap();
    }
    sum
}

/// Raw capacity (no percent) in PD for a window (design §4.3). 1 working day = 1 PD.
pub fn capacity_pd(cal: &Calendar, project_id: i64, resource_id: i64, w: Window) -> f64 {
    sum_day_factors(cal, project_id, resource_id, w.start, w.end)
}

/// Single allocation's PD within a query window (design §4.4).
pub fn alloc_pd(cal: &Calendar, a: &Allocation, w: Window) -> f64 {
    match overlap(Window { start: a.start, end: a.end }, w) {
        None => 0.0,
        Some((os, oe)) => sum_day_factors(cal, a.project_id, a.resource_id, os, oe) * a.percent,
    }
}

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
        // capacity = 4.5; alloc = 4.5 * 0.5 = 2.25
        assert!((alloc_pd(&c, &a, win(MON, FRI)) - 2.25).abs() < 1e-9);
    }

    #[test]
    fn alloc_pd_disjoint_window_zero() {
        let c = cal();
        let a = Allocation { id: 1, resource_id: 1, project_id: 1, start: d(MON), end: d(FRI), percent: 1.0 };
        assert!((alloc_pd(&c, &a, win("2026-08-01", "2026-08-05"))).abs() < 1e-9);
    }

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

    /// Design §4.10 flagship scenario: Alice split across two projects in one week,
    /// with a full-day holiday (Wed) and a half-day time-off (Thu). Pins the three
    /// core properties: (1) calendar deductions apply equally to capacity & workload;
    /// (2) cross-project allocations sum directly; (3) window clipping is correct
    /// (this is the case `SUM(allocated_pd)` gets wrong by 51%).
    #[test]
    fn design_4_10_flagship_two_projects_holiday_and_timeoff() {
        // Week Mon 2026-06-29 .. Sun 2026-07-05. Wed 07-01 = full holiday,
        // Thu 07-02 = Alice half-day leave.
        let mut c = cal();
        c.holidays_global.insert(d(WED), 1.0);          // Wed full-day holiday
        c.time_off.insert((1, d("2026-07-02")), 0.5);   // Thu half-day leave

        // Project A: 50% Mon–Fri. Project B: 60% Tue–Fri.
        let a = Allocation { id: 1, resource_id: 1, project_id: 1,
            start: d(MON), end: d(FRI), percent: 0.5 };
        let b = Allocation { id: 2, resource_id: 1, project_id: 2,
            start: d("2026-06-30"), end: d(FRI), percent: 0.6 }; // Tue–Fri
        let allocs = [a, b];

        // Capacity (project 1's calendar; same global week): day_factor sum
        // = 1.0(Mon)+1.0(Tue)+0(Wed)+0.5(Thu)+1.0(Fri) = 3.5 PD.
        let week = win(MON, "2026-07-05"); // Mon..Sun
        assert!((capacity_pd(&c, 1, 1, week) - 3.5).abs() < 1e-9,
            "§4.10 capacity: 3.5 PD");

        // Workload across both allocations:
        // Mon 0.5 | Tue 0.5+0.6=1.1 | Wed 0 | Thu 0.25+0.30=0.55 | Fri 0.5+0.6=1.1
        // = 3.25 PD.
        assert!((workload_pd(&c, &allocs, 1, week) - 3.25).abs() < 1e-9,
            "§4.10 workload: 3.25 PD");

        // Utilization = 3.25 / 3.5 = 0.9286 (92.9%, green — under 100%).
        assert!((utilization(&c, &allocs, 1, 1, week) - 0.9286).abs() < 1e-3,
            "§4.10 utilization: 0.9286");

        // Clipped-window sub-case (§4.10 Step 7): shrink to Mon..Thu.
        // Correct workload = 0.5+1.1+0+0.55 = 2.15 PD. A naive SUM(allocated_pd)
        // would still return 3.25 (full-range value, no overlap clipping) — 51% high.
        let mon_thu = win(MON, "2026-07-02");
        assert!((workload_pd(&c, &allocs, 1, mon_thu) - 2.15).abs() < 1e-9,
            "§4.10 clipped Mon..Thu workload: 2.15 PD (not the 3.25 a full-range sum gives)");
    }
}