use crate::types::*;
pub trait Solver: Send + Sync {
    fn solve(&self, problem: &AllocationProblem, scores: &ScoreMatrix) -> Solution;
}

/// Deterministic greedy solver. For each task (priority asc, then estimate desc),
/// assign to the highest-scoring resource that meets mandatory skills and keeps
/// per-day load ≤ 1.0 over the window (treating each calendar day in [start,end]
/// as a capacity-1.0 day — a conservative uniform proxy; the workload engine
/// remains authoritative for display). `percent` = min(1.0, estimate/window_days).
pub struct GreedySolver;

fn window_days(start: chrono::NaiveDate, end: chrono::NaiveDate) -> i64 {
    // Floor at 1 so a degenerate/NULL-defaulted window (end < start) can't produce
    // ≤0 days → inf/NaN percent downstream.
    ((end - start).num_days() + 1).max(1)
}

impl Solver for GreedySolver {
    fn solve(&self, problem: &AllocationProblem, scores: &ScoreMatrix) -> Solution {
        // per-resource per-day committed percent (existing + assigned)
        let mut load: std::collections::HashMap<
            i64,
            std::collections::HashMap<chrono::NaiveDate, f64>,
        > = std::collections::HashMap::new();
        for e in &problem.existing {
            let mut d = e.start;
            while d <= e.end {
                *load
                    .entry(e.resource_id)
                    .or_default()
                    .entry(d)
                    .or_insert(0.0) += e.percent;
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

        for t in order {
            let days = window_days(t.start, t.end);
            let needed = (t.estimate_pd / days as f64).clamp(0.01, 1.0);
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
                // per-day load across the task window must stay within the capacity limit
                let ok = {
                    let mut d = t.start;
                    let mut ok = true;
                    while d <= t.end {
                        let cur = *load.entry(r.id).or_default().entry(d).or_insert(0.0);
                        let limit = 1.0; // ratio-space cap Σ percent ≤ 1.0 (design §3.8); SoftWarn penalizes via metrics, not feasibility
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
                    let mut d = t.start;
                    while d <= t.end {
                        *load.entry(r.id).or_default().entry(d).or_insert(0.0) += needed;
                        d = d.succ_opt().unwrap();
                    }
                    let sc = *scores.get(&(r.id, t.id)).unwrap_or(&0.0);
                    score_sum += sc;
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
        Solution {
            run_id: problem.run_id,
            assignments,
            unscheduled,
            metrics: SolutionMetrics {
                overall: skill_fit * problem.weights.skill_fit + scheduled_ratio * problem.weights.balance,
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
