use crate::types::*;
use chrono::{Days, NaiveDate};
use std::collections::HashMap;

#[cfg(feature = "milp")]
pub mod milp;

pub trait Solver: Send + Sync {
    fn solve(&self, problem: &AllocationProblem, scores: &ScoreMatrix) -> Solution;
}

/// Deterministic greedy solver. Tasks are taken in (priority asc, estimate desc)
/// order. For each task, candidate resources (mandatory skills + availability
/// window, mirroring `trg_allocation_validate_insert`) are sorted by
/// **score desc, then window-committed-load asc** — the tiebreaker is the load
/// balancer: among equally-good fits, the less-loaded resource wins. A candidate
/// is feasible iff the uniform daily percent needed to deliver the estimate keeps
/// per-day load ≤ the day's capacity factor over the window. `percent` is that
/// uniform daily ratio (estimate_pd / (Σ day-factors × daily_capacity_pd)).
///
/// Budget is an unconditional hard gate when `budget_pd > 0` (it is *not* gated
/// on objective-weight comparison). `fairness` is the Jain index over per-resource
/// total committed ratio-days. The workload engine remains authoritative for display.
pub struct GreedySolver;

/// Walk every calendar day in [start, end] inclusive. Panics on a pathological
/// range (chrono's checked_add_days returning None); real task windows never hit it.
pub(crate) fn each_day<F: FnMut(NaiveDate)>(start: NaiveDate, end: NaiveDate, mut f: F) {
    let mut d = start;
    while d <= end {
        f(d);
        d = d.checked_add_days(Days::new(1)).unwrap();
    }
}

/// Per-day capacity factor for (resource, day). Missing ⇒ full day (1.0).
pub(crate) fn day_capacity(caps: &HashMap<(i64, NaiveDate), f64>, r: i64, day: NaiveDate) -> f64 {
    caps.get(&(r, day)).copied().unwrap_or(1.0)
}

/// Σ capacity factor over [start, end] for resource r.
pub(crate) fn sum_capacity(caps: &HashMap<(i64, NaiveDate), f64>, r: i64, start: NaiveDate, end: NaiveDate) -> f64 {
    let mut s = 0.0;
    each_day(start, end, |d| s += day_capacity(caps, r, d));
    s
}

/// Σ committed load over [start, end] for resource r — the balance signal.
fn window_load(
    load: &HashMap<(i64, NaiveDate), f64>,
    caps: &HashMap<(i64, NaiveDate), f64>,
    r: i64,
    start: NaiveDate,
    end: NaiveDate,
) -> f64 {
    let mut s = 0.0;
    each_day(start, end, |d| {
        if day_capacity(caps, r, d) > 0.0 {
            s += load.get(&(r, d)).copied().unwrap_or(0.0);
        }
    });
    s
}

/// Uniform daily percent needed to deliver `estimate_pd` over a window whose
/// capacity sum is `capacity_sum`. `None` if even an empty schedule can't fit it
/// (would need >100%/day) or the window has no capacity.
pub(crate) fn needed_percent(estimate_pd: f64, capacity_sum: f64, daily_capacity_pd: f64) -> Option<f64> {
    if capacity_sum <= 0.0 {
        return None;
    }
    if estimate_pd <= 0.0 {
        return Some(0.01);
    }
    let raw = estimate_pd / (capacity_sum * daily_capacity_pd.max(1e-9));
    if !raw.is_finite() || raw > 1.0 + 1e-9 {
        return None;
    }
    Some(raw.clamp(0.01, 1.0))
}

/// True iff assigning `needed` daily keeps every in-window day within its capacity.
fn fits(
    load: &HashMap<(i64, NaiveDate), f64>,
    caps: &HashMap<(i64, NaiveDate), f64>,
    r: i64,
    start: NaiveDate,
    end: NaiveDate,
    needed: f64,
) -> bool {
    let mut d = start;
    while d <= end {
        let limit = day_capacity(caps, r, d);
        if limit > 0.0 {
            let cur = load.get(&(r, d)).copied().unwrap_or(0.0);
            if cur + needed > limit + 1e-9 {
                return false;
            }
        }
        d = d.checked_add_days(Days::new(1)).unwrap();
    }
    true
}

/// Jain fairness index on a slice of per-resource loads: (Σx)² / (n·Σx²). Returns
/// 1.0 for the all-zero case (no imbalance exists) and for n == 0.
fn jain(xs: &[f64]) -> f64 {
    let n = xs.len() as f64;
    if n == 0.0 {
        return 0.0;
    }
    let sum: f64 = xs.iter().sum();
    let sq: f64 = xs.iter().map(|x| x * x).sum();
    if sq == 0.0 {
        return 1.0;
    }
    (sum * sum) / (n * sq)
}

impl Solver for GreedySolver {
    fn solve(&self, problem: &AllocationProblem, scores: &ScoreMatrix) -> Solution {
        let caps: HashMap<(i64, NaiveDate), f64> = problem
            .daily_capacity
            .iter()
            .map(|c| ((c.resource_id, c.day), c.factor.clamp(0.0, 1.0)))
            .collect();

        // load[(r, day)] = committed daily ratio (existing + new); total_load[r] = its row sum.
        let mut load: HashMap<(i64, NaiveDate), f64> = HashMap::new();
        let mut total_load: HashMap<i64, f64> = HashMap::new();
        for e in &problem.existing {
            each_day(e.start, e.end, |d| {
                if day_capacity(&caps, e.resource_id, d) > 0.0 {
                    *load.entry((e.resource_id, d)).or_insert(0.0) += e.percent;
                    *total_load.entry(e.resource_id).or_insert(0.0) += e.percent;
                }
            });
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
        let budget_cap = problem.budget_pd.filter(|b| *b > 0.0);

        for t in order {
            // Hard budget gate (unconditional — not weight-gated).
            if let Some(budget) = budget_cap {
                if planned_pd + t.estimate_pd > budget + 1e-9 {
                    unscheduled.push(t.id);
                    continue;
                }
            }

            // Eligible resources: mandatory skills met AND within availability window.
            let eligible: Vec<&CandidateResource> = problem
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
                        _ => true,
                    };
                    skills_ok && avail_ok
                })
                .collect();

            // Score each candidate once: (score, window-committed-load) for the sort.
            // Sort by score desc, then committed-load asc — the balance tiebreaker.
            let mut ranked: Vec<(&CandidateResource, f64, f64)> = eligible
                .into_iter()
                .map(|r| {
                    let sc = scores.get(&(r.id, t.id)).copied().unwrap_or(0.0);
                    let wl = window_load(&load, &caps, r.id, t.start, t.end);
                    (r, sc, wl)
                })
                .collect();
            ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.2.total_cmp(&b.2)));

            let mut chosen: Option<(&CandidateResource, f64, f64)> = None;
            for (r, sc, _wl) in ranked {
                let cap_sum = sum_capacity(&caps, r.id, t.start, t.end);
                let Some(needed) = needed_percent(t.estimate_pd, cap_sum, r.daily_capacity_pd)
                else {
                    continue;
                };
                if fits(&load, &caps, r.id, t.start, t.end, needed) {
                    chosen = Some((r, needed, sc));
                    break;
                }
            }

            match chosen {
                Some((r, needed, sc)) => {
                    each_day(t.start, t.end, |d| {
                        if day_capacity(&caps, r.id, d) > 0.0 {
                            *load.entry((r.id, d)).or_insert(0.0) += needed;
                            *total_load.entry(r.id).or_insert(0.0) += needed;
                        }
                    });
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

        let n = assignments.len();
        let loads: Vec<f64> = problem
            .resources
            .iter()
            .map(|r| total_load.get(&r.id).copied().unwrap_or(0.0))
            .collect();

        Solution {
            run_id: problem.run_id,
            assignments,
            unscheduled,
            metrics: compute_metrics(
                problem,
                &score_sum,
                planned_pd,
                &loads,
                budget_cap,
                n,
            ),
            status: SolverStatus::Feasible,
        }
    }
}

/// Build `SolutionMetrics` shared by both solvers so the greedy and MILP paths report
/// identical aggregate numbers for the same assignment set.
/// - `score_sum`  : Σ of chosen assignment scores.
/// - `planned_pd` : Σ of chosen task estimates.
/// - `loads`      : per-resource total committed ratio-days (for the Jain fairness index).
/// - `budget_cap` : resolved hard budget cap, if any.
/// - `chosen`     : number of assignments placed (denominator of skill_fit & scheduled_ratio;
///   passed explicitly because two assignments may share a resource, so `loads`'s non-zero
///   count would undercount vs. `assignments.len()`).
pub(crate) fn compute_metrics(
    problem: &AllocationProblem,
    score_sum: &f64,
    planned_pd: f64,
    loads: &[f64],
    budget_cap: Option<f64>,
    chosen: usize,
) -> SolutionMetrics {
    let nc = chosen as f64;
    let nt = problem.tasks.len() as f64;
    let skill_fit = if nc > 0.0 { (score_sum / nc) * 100.0 } else { 0.0 };
    let scheduled_ratio = if nt == 0.0 { 100.0 } else { nc / nt * 100.0 };
    let budget_score = match budget_cap {
        Some(budget) if planned_pd > budget => {
            ((budget / planned_pd) * 100.0).clamp(0.0, 100.0)
        }
        Some(_) => 100.0,
        None => 100.0,
    };
    let fairness = jain(loads) * 100.0;
    SolutionMetrics {
        overall: skill_fit * problem.weights.skill_fit
            + scheduled_ratio * problem.weights.balance
            + budget_score * problem.weights.budget,
        skill_fit,
        scheduled_ratio,
        fairness,
    }
}
