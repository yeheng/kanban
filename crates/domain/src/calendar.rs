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