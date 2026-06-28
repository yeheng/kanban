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
    (end - start).num_days() + 1
}

impl Solver for GreedySolver {
    fn solve(&self, problem: &AllocationProblem, scores: &ScoreMatrix) -> Solution {
        // per-resource per-day committed percent (existing + assigned)
        let mut load: std::collections::HashMap<i64, std::collections::HashMap<chrono::NaiveDate, f64>> = std::collections::HashMap::new();
        for e in &problem.existing {
            let mut d = e.start;
            while d <= e.end {
                *load.entry(e.resource_id).or_default().entry(d).or_insert(0.0) += e.percent;
                d = d.succ_opt().unwrap();
            }
        }

        let mut order: Vec<&CandidateTask> = problem.tasks.iter().collect();
        order.sort_by(|a, b| a.priority.cmp(&b.priority).then_with(|| b.estimate_pd.partial_cmp(&a.estimate_pd).unwrap()));

        let mut assignments = Vec::new();
        let mut unscheduled = Vec::new();
        let mut score_sum = 0.0;

        for t in order {
            let days = window_days(t.start, t.end);
            let needed = (t.estimate_pd / days as f64).clamp(0.01, 1.0);
            // candidate resources: mandatory skills met, sorted by score desc
            let mut cands: Vec<&CandidateResource> = problem.resources.iter()
                .filter(|r| t.skill_reqs.iter().filter(|rq| rq.is_mandatory)
                    .all(|rq| r.skills.get(&rq.skill_id).is_some_and(|p| *p >= rq.min_proficiency)))
                .collect();
            cands.sort_by(|a, b| scores.get(&(b.id, t.id)).unwrap_or(&0.0)
                .partial_cmp(scores.get(&(a.id, t.id)).unwrap_or(&0.0)).unwrap());

            let mut chosen = None;
            for r in cands {
                // per-day load across the task window must stay within the capacity limit
                let ok = {
                    let mut d = t.start; let mut ok = true;
                    while d <= t.end {
                        let cur = *load.entry(r.id).or_default().entry(d).or_insert(0.0);
                        let limit = 1.0; // ratio-space cap Σ percent ≤ 1.0 (design §3.8); SoftWarn penalizes via metrics, not feasibility
                        if cur + needed > limit + 1e-9 { ok = false; break; }
                        d = d.succ_opt().unwrap();
                    }
                    ok
                };
                if ok { chosen = Some(r); break; }
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
                        resource_id: r.id, task_id: t.id, start: t.start, end: t.end,
                        percent: needed, score: sc,
                        rationale: format!("greedy: best feasible (score {:.2})", sc),
                    });
                }
                None => unscheduled.push(t.id),
            }
        }

        let n = assignments.len() as f64;
        let skill_fit = if n > 0.0 { (score_sum / n) * 100.0 } else { 0.0 };
        let scheduled_ratio = if problem.tasks.is_empty() { 100.0 } else { n / problem.tasks.len() as f64 * 100.0 };
        Solution {
            run_id: problem.run_id, assignments, unscheduled,
            metrics: SolutionMetrics {
                overall: skill_fit * 0.6 + scheduled_ratio * 0.4,
                skill_fit, utilization: scheduled_ratio, fairness: 0.0,
            },
        }
    }
}
