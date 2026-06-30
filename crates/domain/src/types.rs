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
    pub daily_capacity_pd: f64,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub percent: f64, // (0.0, 1.0]
}
