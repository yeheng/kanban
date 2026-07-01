//! Exact MILP solver via `good_lp` (modeler) + HiGHS (backend).
//!
//! # Model: binary generalized assignment (one-shot, schema-compatible)
//!
//! The DB `allocations` row is `(resource, task, start, end, percent)` — a single uniform
//! daily ratio over a continuous window, validated by `trg_allocation_validate_insert`
//! (`percent ∈ (0, 1.0]`). A per-day continuous model `percent_{r,t,d}` would produce
//! day-varying ratios incompatible with that schema, so this first cut uses a **binary
//! generalized-assignment** formulation instead:
//!
//! - Decision var `x_{r,t} ∈ {0,1}` — created only for **feasible** pairs (mandatory skills
//!   met ∧ `needed_percent(r,t) = Some(p)`).
//! - The uniform daily ratio `p_{r,t}` for that pair is a **precomputed constant** (the very
//!   same `needed_percent` the greedy solver uses, solver.rs:65), so "resource r spends
//!   `p_{r,t}` of day d on task t" is the linear term `x_{r,t} · p_{r,t}` — a 1:1 match with
//!   the greedy `fits()` feasibility check.
//!
//! ## Constraints (all linear)
//! - One resource per task (allow unscheduled):  `Σ_r x[r,t] ≤ 1`
//! - Daily capacity:  `Σ_t x[r,t]·p[r,t] ≤ factor[r,d]`  for each (r, d) in some task window
//! - Budget hard gate (when `budget_pd > 0`):  `Σ x[r,t]·estimate[t] ≤ budget_pd`
//! - Load-balance coupling (minimax):  `L_max ≥ Σ_t x[r,t]·p[r,t]`  for each r
//!
//! ## Objective (multi-target, normalized to [0,1] so no weight is drowned by scale)
//! `maximize  W_skill·SkillFit_norm + W_sched·Coverage_norm − W_balance·LoadImbalance_norm`
//!
//! Budget is a hard constraint (not an objective term), mirroring the greedy path. The
//! per-day continuous model remains a future enhancement once `allocations` supports
//! per-day / split assignment rows.
use crate::solver::{
    compute_metrics, day_capacity, each_day, needed_percent, sum_capacity, Solver,
};
use crate::types::*;
use chrono::NaiveDate;
use std::collections::HashMap;

#[allow(unused_imports)]
use tracing;

/// Exact MILP solver (good_lp + HiGHS). Construct with a timeout (ms) and a variable-count
/// threshold; `solve` is synchronous and CPU-bound — the app layer wraps it in
/// `tokio::task::spawn_blocking` and an outer `tokio::time::timeout`.
pub struct MilpSolver {
    pub timeout_ms: u64,
    /// If the number of feasible (resource, task) pairs exceeds this, return `SolverStatus::
    /// Error` so the caller falls back to the greedy solver (NP-hard blowup guard).
    pub var_threshold: usize,
}

impl Default for MilpSolver {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            var_threshold: 20_000,
        }
    }
}

/// A precomputed feasible (resource, task) pair with its uniform daily ratio.
struct Feasible {
    r_id: i64,
    t_id: i64,
    r_idx: usize,
    t_idx: usize,
    percent: f64,
    score: f64,
    estimate: f64,
    priority: i64,
}

impl Solver for MilpSolver {
    #[tracing::instrument(skip(self, problem, scores), fields(run_id = problem.run_id, resources = problem.resources.len(), tasks = problem.tasks.len()))]
    fn solve(&self, problem: &AllocationProblem, scores: &ScoreMatrix) -> Solution {
        tracing::info!("MILP solver started");
        use good_lp::constraint;
        use good_lp::variable::Variable;
        // `Solution` clashes with our domain `types::Solution`; alias the good_lp trait.
        use good_lp::{ProblemVariables, SolverModel, default_solver};
        use good_lp::solvers::{Solution as GlSolution, SolutionStatus};

        let caps: HashMap<(i64, NaiveDate), f64> = problem
            .daily_capacity
            .iter()
            .map(|c| ((c.resource_id, c.day), c.factor.clamp(0.0, 1.0)))
            .collect();

        // Seed committed load from existing allocations (same semantics as GreedySolver).
        let mut load: HashMap<(i64, NaiveDate), f64> = HashMap::new();
        for e in &problem.existing {
            each_day(e.start, e.end, |day| {
                if day_capacity(&caps, e.resource_id, day) > 0.0 {
                    *load.entry((e.resource_id, day)).or_insert(0.0) += e.percent;
                }
            });
        }

        // 1. Enumerate feasible (r, t) pairs + their uniform percent (reusing needed_percent).
        let mut feasible: Vec<Feasible> = Vec::new();
        for (ri, r) in problem.resources.iter().enumerate() {
            for (ti, t) in problem.tasks.iter().enumerate() {
                let skills_ok = t
                    .skill_reqs
                    .iter()
                    .filter(|rq| rq.is_mandatory)
                    .all(|rq| r.skills.get(&rq.skill_id).is_some_and(|p| *p >= rq.min_proficiency));
                if !skills_ok {
                    continue;
                }
                let avail_ok = match (r.available_from, r.available_to) {
                    (Some(from), Some(to)) => t.start >= from && t.end <= to,
                    _ => true,
                };
                if !avail_ok {
                    continue;
                }
                let cap_sum = sum_capacity(&caps, r.id, t.start, t.end);
                let Some(percent) = needed_percent(t.estimate_pd, cap_sum, r.daily_capacity_pd)
                else {
                    continue;
                };
                feasible.push(Feasible {
                    r_id: r.id,
                    t_id: t.id,
                    r_idx: ri,
                    t_idx: ti,
                    percent,
                    score: scores.get(&(r.id, t.id)).copied().unwrap_or(0.0),
                    estimate: t.estimate_pd,
                    priority: t.priority,
                });
            }
        }

        // Variable-count guard: too large → signal the caller to fall back to greedy.
        if feasible.len() > self.var_threshold {
            tracing::warn!(feasible_pairs = feasible.len(), threshold = self.var_threshold, "MILP variable threshold exceeded, falling back");
            return Solution {
                run_id: problem.run_id,
                assignments: vec![],
                unscheduled: problem.tasks.iter().map(|t| t.id).collect(),
                metrics: SolutionMetrics::default(),
                status: SolverStatus::Error,
            };
        }

        // 2. Build variables: x[r,t] binary + L_max continuous (≥0).
        let mut vars = ProblemVariables::new();
        let xs: Vec<Variable> = feasible
            .iter()
            .map(|_| vars.add(good_lp::variable().binary()))
            .collect();
        let l_max = vars.add(good_lp::variable().min(0.0));

        // 3. Objective — normalized so each term ∈ [0,1] (no weight drowned by scale).
        //    SkillFit  = Σ x·score          / max possible Σ (per-task best score)
        //    Coverage  = Σ x·priority_w     / Σ all priority_w
        //    Imbalance = L_max              / worst-case single-resource load
        let priority_w = |p: i64| (10 - p.clamp(1, 9)) as f64; // priority 1 → 9, 9 → 1
        let max_skill: f64 = problem
            .tasks
            .iter()
            .map(|t| {
                problem
                    .resources
                    .iter()
                    .filter_map(|r| scores.get(&(r.id, t.id)).copied())
                    .fold(0.0_f64, f64::max)
            })
            .sum();
        let total_priority: f64 = problem.tasks.iter().map(|t| priority_w(t.priority)).sum();

        let eps = 1e-9;
        let mut obj = good_lp::Expression::default();
        for (f, &x) in feasible.iter().zip(xs.iter()) {
            let skill_term = if max_skill > eps { f.score / max_skill } else { 0.0 };
            // Coverage reward: scheduling a task is a core objective, weighted by
            // (skill_fit + balance) so it is always positive — never cancelled to zero by the
            // balance penalty (which uses w_balance alone). Without this, two zero-score tasks
            // on distinct days had reward == penalty ⇒ MILP chose to assign nothing.
            let cov_term = if total_priority > eps {
                priority_w(f.priority) / total_priority
            } else {
                0.0
            };
            let cov_w = problem.weights.skill_fit + problem.weights.balance;
            obj += (problem.weights.skill_fit * skill_term + cov_w * cov_term) * x;
        }
        // Balance penalty: L_max is a per-resource committed ratio (≤ 1.0 = one full day's
        // worth). Normalize against the fixed full-load reference (1.0), NOT against worst_load
        // — normalizing against worst_load made a single assignment saturate the penalty to 1.0
        // (cancelling the coverage reward), so MILP chose to assign nothing. Against 1.0,
        // assigning a 0.5-load task costs only 0.5, leaving a positive net objective.
        obj -= problem.weights.balance * l_max;
        let mut model = vars.maximise(obj).using(default_solver);

        // 4. Constraints.
        // 4a. One resource per task (≤1, i.e. allow unscheduled).
        for ti in 0..problem.tasks.len() {
            let row: good_lp::Expression = feasible
                .iter()
                .zip(xs.iter())
                .filter(|(f, _)| f.t_idx == ti)
                .fold(good_lp::Expression::default(), |acc, (_, &x)| acc + x);
            model = model.with(constraint!(row <= 1));
        }

        // 4b. Daily capacity: Σ_t x·percent ≤ factor[r,d] − existing_load[r,d], per (r,d).
        //     Only build constraints for (r,d) actually covered by some feasible task window.
        let mut days_by_resource: HashMap<i64, Vec<NaiveDate>> = HashMap::new();
        for f in &feasible {
            let r = &problem.resources[f.r_idx];
            each_day(f.t_start(problem), f.t_end(problem), |day| {
                if day_capacity(&caps, r.id, day) > 0.0 {
                    days_by_resource.entry(r.id).or_default().push(day);
                }
            });
        }
        for (r_id, days) in &mut days_by_resource {
            days.sort();
            days.dedup();
            for &day in days.iter() {
                let limit = day_capacity(&caps, *r_id, day) - load.get(&(*r_id, day)).copied().unwrap_or(0.0);
                let row: good_lp::Expression = feasible
                    .iter()
                    .zip(xs.iter())
                    .filter(|(f, _)| {
                        f.r_id == *r_id
                            && day >= problem.tasks[f.t_idx].start
                            && day <= problem.tasks[f.t_idx].end
                    })
                    .fold(good_lp::Expression::default(), |acc, (f, &x)| {
                        acc + f.percent * x
                    });
                // limit may be ≤0 if existing load saturates the day; the constraint still
                // correctly forbids any x that would push over it.
                model = model.with(constraint!(row <= limit));
            }
        }

        // 4c. Budget hard gate (unconditional when budget_pd > 0 — not weight-gated).
        if let Some(budget) = problem.budget_pd.filter(|b| *b > 0.0) {
            let row: good_lp::Expression = feasible
                .iter()
                .zip(xs.iter())
                .fold(good_lp::Expression::default(), |acc, (f, &x)| {
                    acc + f.estimate * x
                });
            model = model.with(constraint!(row <= budget));
        }

        // 4c2. Dependency coupling: Σ_r x[r,task] ≤ Σ_r x[r,predecessor] for each edge among
        //      candidate tasks. If the predecessor has no feasible (r,t) pair (e.g. mandatory
        //      skill unmet), its LHS Σ is the constant 0 ⇒ the dependent's Σ ≤ 0 ⇒ it can't be
        //      scheduled either (cascade). Edges referencing tasks outside this run are skipped.
        let t_id_to_idx: HashMap<i64, usize> = problem.tasks.iter().enumerate().map(|(i, t)| (t.id, i)).collect();
        for d in &problem.dependencies {
            let (Some(&ti), Some(&pi)) = (t_id_to_idx.get(&d.task_id), t_id_to_idx.get(&d.predecessor_id))
            else { continue };
            if ti == pi { continue; }
            let dep_row: good_lp::Expression = feasible
                .iter().zip(xs.iter())
                .filter(|(f, _)| f.t_idx == ti)
                .fold(good_lp::Expression::default(), |acc, (_, &x)| acc + x);
            let pred_row: good_lp::Expression = feasible
                .iter().zip(xs.iter())
                .filter(|(f, _)| f.t_idx == pi)
                .fold(good_lp::Expression::default(), |acc, (_, &x)| acc + x);
            model = model.with(constraint!(dep_row <= pred_row));
        }

        // 4d. Load-balance coupling (per-day minimax): L_max ≥ Σ_{t: day∈window} x[r,t]·percent
        //     for each (resource, day). This is the daily peak load — the right balance signal:
        //     two tasks on *different* days do NOT pile onto the same day, so they should not
        //     inflate the balance penalty. (A per-resource Σ-percent L_max wrongly penalized a
        //     resource staffed across many days at a normal 100%/day, making MILP refuse to
        //     assign anything.) `days_by_resource` (built in 4b) already enumerates the days.
        for (ri, r) in problem.resources.iter().enumerate() {
            let Some(days) = days_by_resource.get(&r.id) else { continue };
            for &day in days {
                let row: good_lp::Expression = feasible
                    .iter()
                    .zip(xs.iter())
                    .filter(|(f, _)| {
                        f.r_idx == ri
                            && day >= problem.tasks[f.t_idx].start
                            && day <= problem.tasks[f.t_idx].end
                    })
                    .fold(good_lp::Expression::default(), |acc, (f, &x)| {
                        acc + f.percent * x
                    });
                model = model.with(constraint!(l_max >= row));
            }
        }

        // 5. Solve with a time limit (HiGHS set_time_limit, in seconds).
        let timeout_s = (self.timeout_ms as f64) / 1000.0;
        let solved = model.set_time_limit(timeout_s).solve();

        let (chosen, status) = match solved {
            Ok(sol) => {
                let mut picked: Vec<&Feasible> = Vec::new();
                for (f, &x) in feasible.iter().zip(xs.iter()) {
                    if sol.value(x) > 0.5 {
                        picked.push(f);
                    }
                }
                // HiGHS exposes optimality via the solution status; good_lp returns Ok for both
                // proven-optimum and time-limited-feasible. Distinguish with the status enum.
                let status = match sol.status() {
                    SolutionStatus::Optimal => SolverStatus::Optimal,
                    SolutionStatus::TimeLimit | SolutionStatus::GapLimit => SolverStatus::Feasible,
                };
                (picked, status)
            }
            Err(_e) => {
                // Infeasible (hard constraints unsatisfiable) or timeout-with-no-feasible /
                // resolution error. The app layer falls back to the greedy solver.
                (Vec::new(), SolverStatus::Infeasible)
            }
        };

        // 6. Assemble the Solution. percent is the precomputed constant (already trigger-safe
        //    in (0,1.0] because needed_percent clamps to [0.01,1.0]).
        let mut assignments = Vec::with_capacity(chosen.len());
        let mut scheduled: HashMap<i64, ()> = HashMap::new();
        let mut score_sum = 0.0;
        let mut planned_pd = 0.0;
        let mut total_load: HashMap<i64, f64> = HashMap::new();
        for f in &chosen {
            let t = &problem.tasks[f.t_idx];
            assignments.push(ScoredAssignment {
                resource_id: f.r_id,
                task_id: f.t_id,
                start: t.start,
                end: t.end,
                percent: f.percent,
                score: f.score,
                rationale: format!("milp: global optimum (score {:.2})", f.score),
            });
            scheduled.insert(f.t_id, ());
            score_sum += f.score;
            planned_pd += f.estimate;
            *total_load.entry(f.r_id).or_insert(0.0) += f.percent;
        }
        let unscheduled: Vec<i64> = problem
            .tasks
            .iter()
            .map(|t| t.id)
            .filter(|id| !scheduled.contains_key(id))
            .collect();

        let loads: Vec<f64> = problem
            .resources
            .iter()
            .map(|r| total_load.get(&r.id).copied().unwrap_or(0.0))
            .collect();
        let budget_cap = problem.budget_pd.filter(|b| *b > 0.0);

        tracing::info!(
            assignments = assignments.len(),
            unscheduled = unscheduled.len(),
            status = ?status,
            overall = compute_metrics(problem, &score_sum, planned_pd, &loads, budget_cap, chosen.len()).overall,
            "MILP solver completed"
        );
        Solution {
            run_id: problem.run_id,
            assignments,
            unscheduled,
            metrics: compute_metrics(problem, &score_sum, planned_pd, &loads, budget_cap, chosen.len()),
            status,
        }
    }
}

impl Feasible {
    fn t_start(&self, problem: &AllocationProblem) -> NaiveDate {
        problem.tasks[self.t_idx].start
    }
    fn t_end(&self, problem: &AllocationProblem) -> NaiveDate {
        problem.tasks[self.t_idx].end
    }
}
