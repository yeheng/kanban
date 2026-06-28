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
}