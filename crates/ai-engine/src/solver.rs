use crate::types::*;
use chrono::{Days, NaiveDate};
pub trait Solver: Send + Sync {
    fn solve(&self, problem: &AllocationProblem, scores: &ScoreMatrix) -> Solution;
}

/// Deterministic greedy solver. For each task (priority asc, then estimate desc),
/// assign to the highest-scoring resource that meets mandatory skills and keeps
/// per-day load ≤ 1.0 over the window (treating each calendar day in [start,end]
/// as a capacity-1.0 day — a conservative uniform proxy; the workload engine
/// remains authoritative for display). `percent` = min(1.0, estimate/window_days).
pub struct GreedySolver;

fn capacity_map(problem: &AllocationProblem) -> std::collections::HashMap<(i64, NaiveDate), f64> {
    problem
        .daily_capacity
        .iter()
        .map(|c| ((c.resource_id, c.day), c.factor.clamp(0.0, 1.0)))
        .collect()
}

fn day_capacity(
    caps: &std::collections::HashMap<(i64, NaiveDate), f64>,
    resource_id: i64,
    day: NaiveDate,
) -> f64 {
    caps.get(&(resource_id, day)).copied().unwrap_or(1.0).clamp(0.0, 1.0)
}

fn sum_capacity(
    caps: &std::collections::HashMap<(i64, NaiveDate), f64>,
    resource_id: i64,
    start: NaiveDate,
    end: NaiveDate,
) -> f64 {
    let mut sum = 0.0;
    let mut d = start;
    while d <= end {
        sum += day_capacity(caps, resource_id, d);
        d = d.checked_add_days(Days::new(1)).unwrap();
    }
    sum
}

impl Solver for GreedySolver {
    fn solve(&self, problem: &AllocationProblem, scores: &ScoreMatrix) -> Solution {
        let caps = capacity_map(problem);
        // per-resource per-day committed percent (existing + assigned)
        let mut load: std::collections::HashMap<
            i64,
            std::collections::HashMap<NaiveDate, f64>,
        > = std::collections::HashMap::new();
        for e in &problem.existing {
            let mut d = e.start;
            while d <= e.end {
                if day_capacity(&caps, e.resource_id, d) > 0.0 {
                    *load
                        .entry(e.resource_id)
                        .or_default()
                        .entry(d)
                        .or_insert(0.0) += e.percent;
                }
                d = d.succ_opt().unwrap();
            }
        }

        let mut order: Vec<&CandidateTask> = problem.tasks.iter().collect();
        // total_cmp (not partial_cmp().unwrap()) so a NaN score can't panic the sort.
        order.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| b.estimate_pd.total_cmp(&a.estimate_pd))
        });

        let mut assignments = Vec::new();
        let mut unscheduled = Vec::new();
        let mut score_sum = 0.0;
        let mut planned_pd = 0.0;
        let budget_cap = problem
            .budget_pd
            .filter(|b| *b > 0.0 && problem.weights.budget > problem.weights.skill_fit.max(problem.weights.balance));

        for t in order {
            if let Some(budget_pd) = budget_cap {
                if planned_pd + t.estimate_pd > budget_pd + 1e-9 {
                    unscheduled.push(t.id);
                    continue;
                }
            }
            // candidate resources: mandatory skills met AND within the resource's
            // availability window (mirrors trg_allocation_validate_insert, so anything
            // the solver accepts the DB trigger will accept on apply()). Sorted by score desc.
            let mut cands: Vec<&CandidateResource> = problem
                .resources
                .iter()
                .filter(|r| {
                    let skills_ok = t.skill_reqs.iter().filter(|rq| rq.is_mandatory).all(|rq| {
                        r.skills
                            .get(&rq.skill_id)
                            .is_some_and(|p| *p >= rq.min_proficiency)
                    });
                    let avail_ok = match (r.available_from, r.available_to) {
                        (Some(from), Some(to)) => t.start >= from && t.end <= to,
                        _ => true, // no window set ⇒ unconstrained (matches the trigger)
                    };
                    skills_ok && avail_ok
                })
                .collect();
            cands.sort_by(|a, b| {
                scores
                    .get(&(b.id, t.id))
                    .unwrap_or(&0.0)
                    .total_cmp(scores.get(&(a.id, t.id)).unwrap_or(&0.0))
            });

            let mut chosen = None;
            for r in cands {
                let capacity_sum = sum_capacity(&caps, r.id, t.start, t.end);
                if capacity_sum <= 0.0 {
                    continue;
                }
                let raw_needed = if t.estimate_pd <= 0.0 {
                    0.01
                } else {
                    t.estimate_pd / (capacity_sum * r.daily_capacity_pd.max(0.000_001))
                };
                if !raw_needed.is_finite() || raw_needed > 1.0 + 1e-9 {
                    continue;
                }
                let needed = raw_needed.clamp(0.01, 1.0);
                // per-day load across the task window must stay within the capacity limit
                let ok = {
                    let mut d = t.start;
                    let mut ok = true;
                    while d <= t.end {
                        let limit = day_capacity(&caps, r.id, d);
                        if limit <= 0.0 {
                            d = d.succ_opt().unwrap();
                            continue;
                        }
                        let cur = *load.entry(r.id).or_default().entry(d).or_insert(0.0);
                        if cur + needed > limit + 1e-9 {
                            ok = false;
                            break;
                        }
                        d = d.succ_opt().unwrap();
                    }
                    ok
                };
                if ok {
                    chosen = Some(r);
                    break;
                }
            }

            match chosen {
                Some(r) => {
                    let capacity_sum = sum_capacity(&caps, r.id, t.start, t.end);
                    let raw_needed = if t.estimate_pd <= 0.0 {
                        0.01
                    } else {
                        t.estimate_pd / (capacity_sum * r.daily_capacity_pd.max(0.000_001))
                    };
                    let needed = raw_needed.clamp(0.01, 1.0);
                    let mut d = t.start;
                    while d <= t.end {
                        if day_capacity(&caps, r.id, d) > 0.0 {
                            *load.entry(r.id).or_default().entry(d).or_insert(0.0) += needed;
                        }
                        d = d.succ_opt().unwrap();
                    }
                    let sc = *scores.get(&(r.id, t.id)).unwrap_or(&0.0);
                    score_sum += sc;
                    planned_pd += t.estimate_pd;
                    assignments.push(ScoredAssignment {
                        resource_id: r.id,
                        task_id: t.id,
                        start: t.start,
                        end: t.end,
                        percent: needed,
                        score: sc,
                        rationale: format!("greedy: best feasible (score {:.2})", sc),
                    });
                }
                None => unscheduled.push(t.id),
            }
        }

        let n = assignments.len() as f64;
        let skill_fit = if n > 0.0 {
            (score_sum / n) * 100.0
        } else {
            0.0
        };
        let scheduled_ratio = if problem.tasks.is_empty() {
            100.0
        } else {
            n / problem.tasks.len() as f64 * 100.0
        };
        let budget_score = match problem.budget_pd {
            Some(budget_pd) if budget_pd > 0.0 && planned_pd > budget_pd => {
                ((budget_pd / planned_pd) * 100.0).clamp(0.0, 100.0)
            }
            Some(_) => 100.0,
            None => 100.0,
        };
        Solution {
            run_id: problem.run_id,
            assignments,
            unscheduled,
            metrics: SolutionMetrics {
                overall: skill_fit * problem.weights.skill_fit
                    + scheduled_ratio * problem.weights.balance
                    + budget_score * problem.weights.budget,
                skill_fit,
                utilization: scheduled_ratio,
                fairness: 0.0,
            },
        }
    }
}

/// MILP formulation (design §5.5.1): x[r,t] ∈ {0,1} + continuous percent, capacity
/// Σ_t percent ≤ 1.0 in ratio space (design §3.8). The full good_lp encoding is a
/// substantial impl-time task; for now `MilpSolver` delegates to `GreedySolver` so the
/// `milp` feature compiles and the seam is in place. Replace the body with the real
/// `good_lp::variable_problem` (HiGHS solver) when wiring production — the formulation:
///   max  Σ w_skill·score[r,t]·x[r,t]  -  w_bal·spread
///   s.t. Σ_t percent[r,t]·x[r,t] ≤ 1.0           ∀ r, day   (capacity)
///        mandatory skill gate (prefiltered via scores==0)
///        x[r,t] ∈ {0,1}, percent ∈ [0,1]
#[cfg(feature = "milp")]
pub mod milp {
    use crate::solver::Solver;
    use crate::types::*;

    pub struct MilpSolver;
    impl Solver for MilpSolver {
        fn solve(&self, problem: &AllocationProblem, scores: &ScoreMatrix) -> Solution {
            // Delegate to the deterministic greedy until the full MILP body is wired.
            super::GreedySolver.solve(problem, scores)
        }
    }
}
