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

/// Allocation row with its task's `project_id` already joined in. The project_id
/// lives on the row (not passed to `to_domain`) so the bridge is zero-argument and
/// callers don't have to track the join tuple themselves.
#[derive(Debug, Clone, FromRow)]
pub struct AllocationRow {
    pub id: i64,
    pub resource_id: i64,
    pub task_id: i64,
    pub project_id: i64,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub percent: f64,
    pub status: String,
    pub source: String,
    pub run_id: Option<i64>,
}

impl AllocationRow {
    /// Bridge to the pure `domain::Allocation` consumed by the workload engine.
    pub fn to_domain(&self) -> domain::Allocation {
        domain::Allocation {
            id: self.id,
            resource_id: self.resource_id,
            project_id: self.project_id,
            start: self.start_date,
            end: self.end_date,
            percent: self.percent,
        }
    }
}