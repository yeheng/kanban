use chrono::NaiveDate;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, serde::Serialize)]
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
/// lives on the row (not passed to `to_domain`) so the bridge is zero-argument.
#[derive(Debug, Clone, FromRow, serde::Serialize)]
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

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub priority: i64,
    pub budget_pd: f64,
    pub max_parallel_tasks_per_day: Option<i64>,
    pub status: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Task {
    pub id: i64,
    pub project_id: i64,
    pub parent_task_id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub estimate_pd: f64,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub is_long_term: i64,
    pub segment_kind: Option<String>,
    pub status: String,
    pub sort_order: i64,
}

/// Kanban-shaped task row: task + assigned resource name (first active allocation).
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct KanbanTask {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub status: String,
    pub sort_order: i64,
    pub estimate_pd: f64,
    pub assignee: Option<String>,
    pub skill_count: i64,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Skill {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Team {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct TeamMember {
    pub team_id: i64,
    pub resource_id: i64,
    pub role: Option<String>,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct TeamOverride {
    pub team_id: i64,
    pub pd_hours: Option<f64>,
    pub pm_workdays: Option<f64>,
    pub overload_threshold: Option<f64>,
    pub underload_threshold: Option<f64>,
    pub utilization_green: Option<f64>,
    pub utilization_yellow: Option<f64>,
}

#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct TaskSkillRequirement {
    pub task_id: i64,
    pub skill_id: i64,
    pub min_proficiency: i64,
    pub is_mandatory: i64,
    pub weight: f64,
}