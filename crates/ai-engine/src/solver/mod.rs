use crate::types::*;
use chrono::NaiveDate;
use domain::each_day;
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

/// Per-day capacity factor for (resource, day). Missing ⇒ full day (1.0).
pub(crate) fn day_capacity(caps: &HashMap<(i64, NaiveDate), f64>, r: i64, day: NaiveDate) -> f64 {
    caps.get(&(r, day)).copied().unwrap_or(1.0)
}

/// Σ capacity factor over [start, end] for resource r.
pub(crate) fn sum_capacity(caps: &HashMap<(i64, NaiveDate), f64>, r: i64, start: NaiveDate, end: NaiveDate) -> f64 {
    each_day(start, end).map(|d| day_capacity(caps, r, d)).sum()
}

/// Σ committed load over [start, end] for resource r — the balance signal.
fn window_load(
    load: &HashMap<(i64, NaiveDate), f64>,
    caps: &HashMap<(i64, NaiveDate), f64>,
    r: i64,
    start: NaiveDate,
    end: NaiveDate,
) -> f64 {
    each_day(start, end)
        .filter(|d| day_capacity(caps, r, *d) > 0.0)
        .map(|d| load.get(&(r, d)).copied().unwrap_or(0.0))
        .sum()
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
    each_day(start, end).all(|d| {
        let limit = day_capacity(caps, r, d);
        if limit > 0.0 {
            let cur = load.get(&(r, d)).copied().unwrap_or(0.0);
            cur + needed <= limit + 1e-9
        } else {
            true
        }
    })
}

/// Jain fairness index on a slice of per-resource loads: (Σx)² / (n·Σx²).
/// Returns 0.0 for n == 0 and for the all-zero case (no work distributed → no
/// fairness to speak of — a do-nothing solution is not "perfectly balanced").
fn jain(xs: &[f64]) -> f64 {
    let n = xs.len() as f64;
    if n == 0.0 {
        return 0.0;
    }
    let sum: f64 = xs.iter().sum();
    let sq: f64 = xs.iter().map(|x| x * x).sum();
    if sq == 0.0 {
        return 0.0;
    }
    (sum * sum) / (n * sq)
}

impl Solver for GreedySolver {
    #[tracing::instrument(skip(self, problem, scores), fields(run_id = problem.run_id, resources = problem.resources.len(), tasks = problem.tasks.len()))]
    fn solve(&self, problem: &AllocationProblem, scores: &ScoreMatrix) -> Solution {
        tracing::info!("greedy solver started");
        let caps: HashMap<(i64, NaiveDate), f64> = problem
            .daily_capacity
            .iter()
            .map(|c| ((c.resource_id, c.day), c.factor.clamp(0.0, 1.0)))
            .collect();

        // load[(r, day)] = committed daily ratio (existing + new); total_load[r] = its row sum.
        let mut load: HashMap<(i64, NaiveDate), f64> = HashMap::new();
        let mut total_load: HashMap<i64, f64> = HashMap::new();
        for e in &problem.existing {
            for d in each_day(e.start, e.end) {
                if day_capacity(&caps, e.resource_id, d) > 0.0 {
                    *load.entry((e.resource_id, d)).or_insert(0.0) += e.percent;
                    *total_load.entry(e.resource_id).or_insert(0.0) += e.percent;
                }
            }
        }

        // Order tasks by dependency topology first (predecessors before dependents), then by
        // (priority asc, estimate desc) within each layer. The topological order is what makes
        // the cascade correct: a predecessor is decided (scheduled or unscheduled) before its
        // dependent is considered, so the dependent can check the predecessor's outcome.
        let order = topo_order(&problem.tasks, &problem.dependencies);

        let mut assignments = Vec::new();
        let mut unscheduled = Vec::new();
        let mut scheduled: std::collections::HashSet<i64> = std::collections::HashSet::new();
        let mut score_sum = 0.0;
        let mut planned_pd = 0.0;
        let budget_cap = problem.budget_pd.filter(|b| *b > 0.0);

        for t in order {
            // Dependency cascade: a task whose predecessor is NOT scheduled (infeasible, over
            // budget, or itself cascade-unscheduled) cannot be scheduled either.
            let preds_ok = problem
                .dependencies
                .iter()
                .filter(|d| d.task_id == t.id)
                .all(|d| scheduled.contains(&d.predecessor_id));
            if !preds_ok {
                unscheduled.push(t.id);
                continue;
            }
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
            // Rank key = score − λ·(resource_total_load / n_resources), where λ = w_balance.
            // This makes the fairness objective an ACTIVE driver (not just a tiebreaker): a
            // less-loaded resource wins even when its raw score is slightly lower, with the
            // strength of the pull set by the balance weight. At w_balance = 0 ⇒ λ = 0 ⇒ the
            // key is pure score, and the `.then_with(load asc)` tiebreaker reproduces the
            // original byte-identical behavior (verified by a snapshot test).
            let lambda = problem.weights.balance;
            let n_res = problem.resources.len().max(1) as f64;
            let mut ranked: Vec<(&CandidateResource, f64, f64)> = eligible
                .into_iter()
                .map(|r| {
                    let sc = scores.get(&(r.id, t.id)).copied().unwrap_or(0.0);
                    let wl = window_load(&load, &caps, r.id, t.start, t.end);
                    (r, sc, wl)
                })
                .collect();
            ranked.sort_by(|a, b| {
                let ra = a.1 - lambda * (total_load.get(&a.0.id).copied().unwrap_or(0.0) / n_res);
                let rb = b.1 - lambda * (total_load.get(&b.0.id).copied().unwrap_or(0.0) / n_res);
                rb.total_cmp(&ra).then_with(|| a.2.total_cmp(&b.2))
            });

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
                    for d in each_day(t.start, t.end) {
                        if day_capacity(&caps, r.id, d) > 0.0 {
                            *load.entry((r.id, d)).or_insert(0.0) += needed;
                            *total_load.entry(r.id).or_insert(0.0) += needed;
                        }
                    }
                    score_sum += sc;
                    planned_pd += t.estimate_pd;
                    scheduled.insert(t.id);
                    assignments.push(ScoredAssignment {
                        resource_id: r.id,
                        task_id: t.id,
                        resource_name: r.name.clone(),
                        task_title: t.title.clone(),
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

        tracing::info!(
            assignments = assignments.len(),
            unscheduled = unscheduled.len(),
            overall = compute_metrics(problem, &score_sum, planned_pd, &loads, budget_cap, n).overall,
            "greedy solver completed"
        );
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

/// Topologically order the candidate tasks so every predecessor (per `dependencies`) comes
/// before its dependent. Within each topological layer, break ties by (priority asc, estimate
/// desc) — the same key the old plain sort used — so dependency-free problems produce the
/// identical order as before. Edges referencing tasks not in `tasks` are ignored (they can't
/// be enforced inside this run anyway). Cycles are impossible here: `add_dependency` rejects
/// cycles at write time; a self-loop can't exist (CHECK constraint). If the unexpected happens,
/// the offending edge is skipped (no infinite loop) rather than panicking.
pub(crate) fn topo_order<'a>(
    tasks: &'a [CandidateTask],
    dependencies: &[TaskDependency],
) -> Vec<&'a CandidateTask> {
    use std::collections::{HashMap, HashSet, VecDeque};
    let id_idx: HashMap<i64, usize> = tasks.iter().enumerate().map(|(i, t)| (t.id, i)).collect();
    // in-degree counts predecessors that are themselves candidate tasks.
    let mut indeg: Vec<usize> = vec![0; tasks.len()];
    let mut succ: HashMap<usize, Vec<usize>> = HashMap::new();
    for d in dependencies {
        let (Some(&ti), Some(&pi)) = (id_idx.get(&d.task_id), id_idx.get(&d.predecessor_id))
        else { continue }; // edge references a task outside this run — ignore
        if ti == pi { continue; } // defensive: self-edge (shouldn't exist)
        indeg[ti] += 1;
        succ.entry(pi).or_default().push(ti);
    }
    // Kahn's algorithm with a sorted VecDeque: pop_front is O(1) (vs O(n) for Vec::remove(0)),
    // and new entries are inserted in sorted position via binary_search.
    let mut ready: VecDeque<usize> = (0..tasks.len()).filter(|&i| indeg[i] == 0).collect();
    ready.make_contiguous().sort_by(|&a, &b| tasks[a].priority.cmp(&tasks[b].priority)
        .then_with(|| tasks[b].estimate_pd.total_cmp(&tasks[a].estimate_pd)));
    let mut out: Vec<&CandidateTask> = Vec::with_capacity(tasks.len());
    let mut emitted: HashSet<usize> = HashSet::new();
    while let Some(i) = ready.pop_front() {
        emitted.insert(i);
        out.push(&tasks[i]);
        if let Some(next) = succ.get(&i) {
            for &j in next {
                indeg[j] -= 1;
                if indeg[j] == 0 && !emitted.contains(&j) {
                    // insert keeping the priority/estimate order
                    let pos = ready.make_contiguous().binary_search_by(|&k| {
                        tasks[k].priority.cmp(&tasks[j].priority)
                            .then_with(|| tasks[j].estimate_pd.total_cmp(&tasks[k].estimate_pd))
                    }).unwrap_or_else(|e| e);
                    ready.insert(pos, j);
                }
            }
        }
    }
    // Any task still not emitted (defensive: a cycle slipped through) appended in stable order.
    for (i, t) in tasks.iter().enumerate() {
        if !emitted.contains(&i) { out.push(t); }
    }
    out
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
///
/// Weight→metric mapping: `skill_fit` weight → avg score, `balance` weight → Jain fairness,
/// `budget` weight → budget adherence. Coverage (`scheduled_ratio`) is reported as its own
/// sub-metric and is an implicit objective of both solvers (greedy schedules everything it
/// can; MILP has an explicit coverage term), not a component of `overall`.
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
            + fairness * problem.weights.balance
            + budget_score * problem.weights.budget,
        skill_fit,
        scheduled_ratio,
        fairness,
    }
}
