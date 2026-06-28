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