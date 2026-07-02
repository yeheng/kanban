pub mod calendar;
pub mod error;
pub mod types;
pub mod unit;
pub mod workload;

pub use calendar::Calendar;
pub use error::DomainError;
pub use types::{Allocation, DayFraction, Window};
pub use unit::UnitConfig;
pub use workload::{each_day, alloc_pd, capacity_pd, count_calendar_days, overlap, workload_pd, utilization, team_utilization};