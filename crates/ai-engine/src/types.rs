use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateResource {
    pub id: i64,
    pub name: String,
    /// skill_id -> proficiency (1..5)
    pub skills: HashMap<i64, i64>,
    pub tags: Vec<String>,
    pub daily_capacity_pd: f64,
    /// Availability window (design §3.3.2). When both are Some, the solver gates
    /// assignments to [available_from, available_to] to mirror the
    /// trg_allocation_validate_insert trigger (else apply() would ABORT).
    pub available_from: Option<NaiveDate>,
    pub available_to: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillReq {
    pub skill_id: i64,
    pub min_proficiency: i64,
    pub is_mandatory: bool,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateTask {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub estimate_pd: f64,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub priority: i64, // 1..9 (lower = higher priority)
    pub skill_reqs: Vec<SkillReq>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExistingAlloc {
    pub resource_id: i64,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCapacity {
    pub resource_id: i64,
    pub day: NaiveDate,
    /// Ratio-space capacity for that resource/day. 1.0 = full day, 0.0 = unavailable.
    pub factor: f64,
}

/// Multi-objective weights (design §1; default balanced 0.4/0.4/0.2; UI-tunable, confirmed #6).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveWeights {
    pub skill_fit: f64,
    pub balance: f64,
    pub budget: f64,
}
impl Default for ObjectiveWeights {
    fn default() -> Self {
        Self {
            skill_fit: 0.4,
            balance: 0.4,
            budget: 0.2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverConfig {
    pub backend: String,
    /// Wall-clock budget handed to the exact solver (HiGHS) in milliseconds. The solver is
    /// also wrapped in an outer `tokio::time::timeout` with this value (plus slack) by the
    /// app layer, so a runaway solve cannot hang the request. Default 5000 (matches the DB
    /// `settings.solver_timeout_ms` default, 0001_init.sql:16).
    pub timeout_ms: u64,
    /// If the count of feasible (resource, task) pairs exceeds this, the MILP backend is
    /// skipped and the greedy solver runs instead (variable-count guard against NP-hard
    /// blowup). Default 20000 (design §5.5.2 `milp_var_threshold`).
    pub milp_var_threshold: usize,
}
impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            backend: "greedy".into(),
            timeout_ms: 5000,
            milp_var_threshold: 20000,
        }
    }
}

/// Solver outcome class. Serializes to the lowercase token that the DB `ai_optimization_runs.
/// solver_status` CHECK constraint accepts: `optimal | feasible | infeasible | timeout | error`
/// (0001_init.sql:247). `GreedySolver` always returns `Feasible`; `MilpSolver` distinguishes
/// HiGHS's proven-optimum / time-limited-feasible / infeasible / timeout outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SolverStatus {
    #[default]
    Feasible,
    Optimal,
    Infeasible,
    Timeout,
    Error,
}
impl SolverStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            SolverStatus::Feasible => "feasible",
            SolverStatus::Optimal => "optimal",
            SolverStatus::Infeasible => "infeasible",
            SolverStatus::Timeout => "timeout",
            SolverStatus::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AllocationProblem {
    pub run_id: i64,
    pub resources: Vec<CandidateResource>,
    pub tasks: Vec<CandidateTask>,
    pub existing: Vec<ExistingAlloc>,
    pub daily_capacity: Vec<DailyCapacity>,
    pub budget_pd: Option<f64>,
    pub weights: ObjectiveWeights,
    pub config: SolverConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredAssignment {
    pub resource_id: i64,
    pub task_id: i64,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub percent: f64,
    pub score: f64,
    pub rationale: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SolutionMetrics {
    pub overall: f64,
    pub skill_fit: f64,
    /// % of candidate tasks the solver placed (shown as "排期覆盖" in the UI).
    pub scheduled_ratio: f64,
    /// Jain fairness index over per-resource total committed ratio-days, ×100.
    pub fairness: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Solution {
    pub run_id: i64,
    pub assignments: Vec<ScoredAssignment>,
    pub unscheduled: Vec<i64>, // task ids
    pub metrics: SolutionMetrics,
    /// Outcome class of the solver that produced this solution. Mirrors the DB
    /// `solver_status` enum (0001_init.sql:247). Defaults to `Feasible` for backward
    /// compatibility with the greedy path and `Solution::default()`.
    #[serde(default)]
    pub status: SolverStatus,
}

/// score[(resource_id, task_id)] -> 0..1 (filled by the Scorer, consumed by the Solver).
pub type ScoreMatrix = HashMap<(i64, i64), f64>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedPlan {
    pub solution: Solution,
    pub explanation_md: String,
}
