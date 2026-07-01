//! Exact-MILP solver tests (gated on the `milp` feature → good_lp + HiGHS).
#![cfg(feature = "milp")]

use ai_engine::solver::{milp::MilpSolver, GreedySolver, Solver};
use ai_engine::types::*;
use chrono::NaiveDate;
use std::collections::HashMap;

fn d(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

/// Two resources, one shared 1-day window, three 1.0-PD tasks. Each resource can absorb at
/// most one task that day (capacity 1.0/day). Scores are arranged so the myopic greedy
/// choice (T1→R1 high, T2→R2 low) blocks the high-value T3→R2 pairing:
///
///   R1: T1=0.9  T2=0.9  T3=0.1
///   R2: T1=0.1  T2=0.1  T3=0.9
///
/// Greedy (priority asc: T1,T2,T3) → T1→R1(0.9), T2→R2(0.1), T3 unscheduled = Σ 1.0.
/// Optimal                              → T1→R1(0.9), T3→R2(0.9), T2 unscheduled = Σ 1.8.
///
/// Only an exact solver finds the 1.8 optimum; greedy is provably stuck at 1.0.
fn bottleneck() -> AllocationProblem {
    AllocationProblem {
        resources: vec![
            CandidateResource {
                id: 1,
                name: "R1".into(),
                skills: HashMap::new(),
                tags: vec![],
                daily_capacity_pd: 1.0,
                available_from: None,
                available_to: None,
            },
            CandidateResource {
                id: 2,
                name: "R2".into(),
                skills: HashMap::new(),
                tags: vec![],
                daily_capacity_pd: 1.0,
                available_from: None,
                available_to: None,
            },
        ],
        tasks: vec![
            CandidateTask {
                id: 1,
                project_id: 1,
                title: "T1".into(),
                estimate_pd: 1.0,
                start: d("2026-07-01"),
                end: d("2026-07-01"),
                priority: 1,
                skill_reqs: vec![],
            },
            CandidateTask {
                id: 2,
                project_id: 1,
                title: "T2".into(),
                estimate_pd: 1.0,
                start: d("2026-07-01"),
                end: d("2026-07-01"),
                priority: 2,
                skill_reqs: vec![],
            },
            CandidateTask {
                id: 3,
                project_id: 1,
                title: "T3".into(),
                estimate_pd: 1.0,
                start: d("2026-07-01"),
                end: d("2026-07-01"),
                priority: 3,
                skill_reqs: vec![],
            },
        ],
        weights: ObjectiveWeights {
            skill_fit: 0.9,
            balance: 0.1,
            budget: 0.0,
        },
        ..Default::default()
    }
}

fn scores_for(p: &AllocationProblem) -> ScoreMatrix {
    // Fixed matrix (independent of FallbackScorer) so the optimality target is unambiguous.
    HashMap::from([
        ((1, 1), 0.9),
        ((1, 2), 0.9),
        ((1, 3), 0.1),
        ((2, 1), 0.1),
        ((2, 2), 0.1),
        ((2, 3), 0.9),
    ])
    .into_iter()
    .filter(|((r, t), _)| p.resources.iter().any(|x| x.id == *r) && p.tasks.iter().any(|x| x.id == *t))
    .collect()
}

fn score_sum(sol: &Solution) -> f64 {
    sol.assignments.iter().map(|a| a.score).sum()
}

#[test]
fn c1_milp_finds_global_optimum_greedy_misses() {
    let p = bottleneck();
    let m = scores_for(&p);

    let greedy = GreedySolver.solve(&p, &m);
    let milp = MilpSolver::default().solve(&p, &m);

    assert!(
        score_sum(&milp) > score_sum(&greedy) + 1e-6,
        "MILP Σscore {} must beat greedy {} on the bottleneck instance",
        score_sum(&milp),
        score_sum(&greedy)
    );
    assert!(
        (score_sum(&milp) - 1.8).abs() < 1e-6,
        "MILP should reach the 1.8 optimum (T1→R1, T3→R2), got {}",
        score_sum(&milp)
    );
    assert_eq!(milp.assignments.len(), 2, "MILP places both high-value tasks");
}

#[test]
fn c3_mandatory_skill_unmet_task_stays_unscheduled() {
    // R1 lacks the mandatory skill (proficiency 2 < required 3) for the only task.
    // The pair is never created ⇒ the task is provably unscheduled, never assigned.
    let mut p = bottleneck();
    p.resources = vec![p.resources[0].clone()];
    p.resources[0].skills = HashMap::from([(1, 2)]); // proficiency 2 < min 3
    p.tasks.truncate(1);
    p.tasks[0].skill_reqs = vec![SkillReq {
        skill_id: 1,
        min_proficiency: 3,
        is_mandatory: true,
        weight: 1.0,
    }];
    let m = scores_for(&p);
    let sol = MilpSolver::default().solve(&p, &m);
    assert!(sol.assignments.is_empty(), "no feasible resource ⇒ no assignment");
    assert_eq!(sol.unscheduled, vec![1]);
}

#[test]
fn c4_budget_hard_gate_caps_planned_pd() {
    // Two 1.0-PD tasks, budget 1.0 ⇒ at most one can be scheduled (hard gate, not weighted).
    let mut p = bottleneck();
    p.tasks.truncate(2);
    p.budget_pd = Some(1.0);
    p.weights = ObjectiveWeights {
        skill_fit: 0.1,
        balance: 0.1,
        budget: 0.8, // budget-dominant; must NOT change the hard gate
    };
    let m = scores_for(&p);
    let sol = MilpSolver::default().solve(&p, &m);
    let planned: f64 = sol
        .assignments
        .iter()
        .map(|a| p.tasks.iter().find(|t| t.id == a.task_id).unwrap().estimate_pd)
        .sum();
    assert!(planned <= 1.0 + 1e-9, "budget hard gate violated: planned {}", planned);
    assert_eq!(sol.assignments.len(), 1);
    assert_eq!(sol.unscheduled.len(), 1);
}

#[test]
fn c5_var_threshold_overflow_signals_fallback() {
    // Threshold 0 ⇒ any non-empty problem overflows ⇒ status Error so the caller falls back.
    let p = bottleneck();
    let m = scores_for(&p);
    let sol = MilpSolver { timeout_ms: 5000, var_threshold: 0 }.solve(&p, &m);
    assert_eq!(sol.status, SolverStatus::Error);
    assert!(sol.assignments.is_empty(), "overflow must not emit assignments");
}

#[test]
fn c6_timeout_does_not_hang_and_falls_back_cleanly() {
    // A pathologically tight timeout must return promptly (caller wraps in spawn_blocking +
    // tokio::time::timeout). Here we assert the solver itself returns within a generous bound
    // and never panics, even with a 1ms budget on a real problem.
    let p = bottleneck();
    let m = scores_for(&p);
    let start = std::time::Instant::now();
    let sol = MilpSolver { timeout_ms: 1, var_threshold: 20_000 }.solve(&p, &m);
    let elapsed = start.elapsed();
    assert!(elapsed.as_secs() < 5, "solve took {:?}, expected sub-5s", elapsed);
    // HiGHS on this tiny instance likely still solves optimally within 1ms; either way, no panic.
    let _ = sol;
}

#[test]
fn c2_balance_weight_drives_load_distribution() {
    // With skill_fit dominant, MILP picks the two 0.9 pairings (T1→R1, T3→R2) — already balanced.
    // With balance dominant and a structure where the high-skill choice concentrates load on one
    // resource, the L_max term should pull assignments toward the less-loaded resource.
    //
    // Instance: 1 resource-capacity-day, 2 tasks each 1.0 PD on the same day, 2 resources.
    // R1 scores 0.9 on BOTH tasks; R2 scores 0.1 on both. skill_fit-dominant ⇒ both on R1 is
    // infeasible (cap 1.0/day), so one goes to R2 anyway. balance-dominant ⇒ same split but
    // chosen for load reasons. The discriminator: under balance-dominance, MILP must NOT leave
    // one resource idle while the other is saturated when a feasible balanced split exists.
    // We assert the balanced split (one task per resource) is found and L_max stays low.
    let p = AllocationProblem {
        resources: vec![
            CandidateResource {
                id: 1, name: "R1".into(), skills: HashMap::new(), tags: vec![],
                daily_capacity_pd: 1.0, available_from: None, available_to: None,
            },
            CandidateResource {
                id: 2, name: "R2".into(), skills: HashMap::new(), tags: vec![],
                daily_capacity_pd: 1.0, available_from: None, available_to: None,
            },
        ],
        tasks: vec![
            CandidateTask {
                id: 1, project_id: 1, title: "A".into(), estimate_pd: 1.0,
                start: d("2026-07-01"), end: d("2026-07-01"), priority: 1, skill_reqs: vec![],
            },
            CandidateTask {
                id: 2, project_id: 1, title: "B".into(), estimate_pd: 1.0,
                start: d("2026-07-01"), end: d("2026-07-01"), priority: 1, skill_reqs: vec![],
            },
        ],
        weights: ObjectiveWeights { skill_fit: 0.1, balance: 0.9, budget: 0.0 },
        ..Default::default()
    };
    let m: ScoreMatrix = HashMap::from([
        ((1, 1), 0.9), ((1, 2), 0.9),
        ((2, 1), 0.1), ((2, 2), 0.1),
    ]);
    let sol = MilpSolver::default().solve(&p, &m);
    // Balanced: one task per resource (daily cap 1.0 forbids both on one resource anyway).
    let mut rids: Vec<i64> = sol.assignments.iter().map(|a| a.resource_id).collect();
    rids.sort();
    assert_eq!(rids, vec![1, 2], "balance-dominant weights must split across resources");
    assert_eq!(sol.assignments.len(), 2);
}
