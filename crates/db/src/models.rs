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

/// Allocation joined with resource name + task title (allocation editor / Gantt later).
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct AllocationView {
    pub id: i64,
    pub resource_id: i64,
    pub resource_name: String,
    pub task_id: i64,
    pub task_title: String,
    pub project_id: i64,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub percent: f64,
    pub status: String,
    pub source: String,
}

/// One allocation bar for a Gantt view (joined with resource/task/project names).
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct GanttBar {
    pub allocation_id: i64,
    pub resource_id: i64,
    pub resource_name: String,
    pub task_id: i64,
    pub task_title: String,
    pub project_id: i64,
    pub project_name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub percent: f64,
    pub status: String,
    pub source: String,
}

/// A dependency edge for drawing Gantt arrows.
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct DepEdge {
    pub task_id: i64,
    pub predecessor_id: i64,
    pub lag_days: i64,
    pub dep_type: String,
}

/// Per-day per-resource occupancy (calendar view).
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct DayOccupancy {
    pub date: NaiveDate,
    pub resource_id: i64,
    pub resource_name: String,
    pub workload_pd: f64,
    pub capacity_pd: f64,
    pub utilization: f64,
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

#[derive(Debug, Clone, FromRow, serde::Serialize, serde::Deserialize)]
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

/// Work-week template row (design §3.3.9a). `mon..sun` are 0/1 on/off bits;
/// `*_frac` are the per-day capacity fractions in (0, 1.0]. Off-day fracs are
/// ignored by hydration (`frac_of` zeroes them) so they are stored as 1.0 to
/// satisfy the schema CHECK (`*_frac > 0`).
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct WeekTemplate {
    pub id: i64,
    pub scope: String,
    pub project_id: Option<i64>,
    pub mon: i64, pub tue: i64, pub wed: i64, pub thu: i64, pub fri: i64, pub sat: i64, pub sun: i64,
    pub mon_frac: f64, pub tue_frac: f64, pub wed_frac: f64, pub thu_frac: f64,
    pub fri_frac: f64, pub sat_frac: f64, pub sun_frac: f64,
}

/// Holiday row (design §3.3.9b). `project_id = NULL` ⇒ global holiday.
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct Holiday {
    pub id: i64,
    pub project_id: Option<i64>,
    pub day: String,
    pub fraction: f64,
    pub name: Option<String>,
}

/// Per-resource time-off row (design §3.3.9c).
#[derive(Debug, Clone, FromRow, serde::Serialize)]
pub struct TimeOff {
    pub id: i64,
    pub resource_id: i64,
    pub day: String,
    pub fraction: f64,
    pub reason: Option<String>,
}