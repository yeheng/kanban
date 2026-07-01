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

/// Trap #1 (per-day L_max, not per-resource Σ-percent): one resource, two zero-score tasks on
/// *different* days (each 1.0 PD, daily cap 1.0). A per-resource Σ-percent L_max would make
/// assigning both ⇒ L_max = 2.0 ⇒ penalty 0.4·2.0 = 0.8 > reward ⇒ MILP refuses. The per-day
/// L_max keeps the daily peak at 1.0 (one task/day, normal), so both schedule. This locks in
/// the fix that was found while debugging the app `apply_skips_out_of_window` test.
#[test]
fn per_day_lmax_does_not_penalize_different_day_tasks() {
    let p = AllocationProblem {
        resources: vec![CandidateResource {
            id: 1, name: "Alice".into(), skills: HashMap::new(), tags: vec![],
            daily_capacity_pd: 1.0, available_from: None, available_to: None,
        }],
        tasks: vec![
            CandidateTask {
                id: 1, project_id: 1, title: "T1".into(), estimate_pd: 1.0,
                start: d("2026-07-01"), end: d("2026-07-01"), priority: 5, skill_reqs: vec![],
            },
            CandidateTask {
                id: 2, project_id: 1, title: "T2".into(), estimate_pd: 1.0,
                start: d("2026-07-02"), end: d("2026-07-02"), priority: 5, skill_reqs: vec![],
            },
        ],
        // Balanced weights (the app default). Coverage reward weighted by w_skill+w_balance.
        weights: ObjectiveWeights { skill_fit: 0.4, balance: 0.4, budget: 0.2 },
        ..Default::default()
    };
    // Zero scores (no skills/tags ⇒ FallbackScorer gives 0) — the trap condition.
    let m: ScoreMatrix = HashMap::from([((1, 1), 0.0), ((1, 2), 0.0)]);
    let sol = MilpSolver::default().solve(&p, &m);
    assert_eq!(
        sol.assignments.len(), 2,
        "different-day tasks must both schedule (per-day L_max); got {} assignments",
        sol.assignments.len()
    );
}

/// Trap #2 (coverage reward not cancelled by balance penalty at zero scores): with all scores
/// 0 and balance-dominant weights, the coverage reward (w_skill+w_balance) must strictly
/// exceed the balance penalty so a feasible single task is still placed. Guards against the
/// earlier `cov_w = w_balance`-only weighting where reward == penalty ⇒ MILP assigned nothing.
#[test]
fn coverage_reward_beats_balance_penalty_at_zero_scores() {
    let p = AllocationProblem {
        resources: vec![CandidateResource {
            id: 1, name: "Alice".into(), skills: HashMap::new(), tags: vec![],
            daily_capacity_pd: 1.0, available_from: None, available_to: None,
        }],
        tasks: vec![CandidateTask {
            id: 1, project_id: 1, title: "T".into(), estimate_pd: 1.0,
            start: d("2026-07-01"), end: d("2026-07-02"), priority: 5, skill_reqs: vec![],
        }],
        // Balance-dominant — the worst case for the cancellation trap.
        weights: ObjectiveWeights { skill_fit: 0.1, balance: 0.9, budget: 0.0 },
        ..Default::default()
    };
    let m: ScoreMatrix = HashMap::from([((1, 1), 0.0)]);
    let sol = MilpSolver::default().solve(&p, &m);
    assert!(
        !sol.assignments.is_empty(),
        "a feasible zero-score task must still be placed (coverage reward > balance penalty)"
    );
}

/// Dependency coupling (MILP): T2 depends on T1. The constraint `Σ_r x[r,T2] ≤ Σ_r x[r,T1]`
/// means T2 cannot be scheduled unless T1 is also scheduled. When T1 is infeasible (mandatory
/// skill unmet, so no x[T1] variable exists), T1's RHS is 0 ⇒ T2's x must be 0 too. When both
/// are feasible, both are placed.
#[test]
fn milp_dependency_coupling_cascades_when_predecessor_infeasible() {
    let skill = 1i64;
    let make = |alice_has_skill: bool| {
        AllocationProblem {
            resources: vec![CandidateResource {
                id: 1, name: "Alice".into(),
                skills: if alice_has_skill { HashMap::from([(skill, 4)]) } else { HashMap::new() },
                tags: vec![], daily_capacity_pd: 1.0, available_from: None, available_to: None,
            }],
            tasks: vec![
                CandidateTask {
                    id: 1, project_id: 1, title: "T1".into(), estimate_pd: 1.0,
                    start: d("2026-07-01"), end: d("2026-07-01"), priority: 1,
                    skill_reqs: vec![SkillReq { skill_id: skill, min_proficiency: 3, is_mandatory: true, weight: 1.0 }],
                },
                CandidateTask {
                    id: 2, project_id: 1, title: "T2".into(), estimate_pd: 1.0,
                    start: d("2026-07-02"), end: d("2026-07-02"), priority: 2, skill_reqs: vec![],
                },
            ],
            dependencies: vec![TaskDependency { task_id: 2, predecessor_id: 1 }],
            weights: ObjectiveWeights { skill_fit: 0.4, balance: 0.4, budget: 0.2 },
            ..Default::default()
        }
    };
    let m: ScoreMatrix = HashMap::from([((1, 1), 0.5), ((1, 2), 0.5)]);

    // Predecessor infeasible ⇒ both unscheduled (no x[T1] var ⇒ Σ x[T2] ≤ 0).
    let sol = MilpSolver::default().solve(&make(false), &m);
    assert!(sol.assignments.is_empty(), "T2 must be blocked when T1 is infeasible (MILP coupling)");

    // Predecessor feasible ⇒ both scheduled.
    let sol = MilpSolver::default().solve(&make(true), &m);
    assert_eq!(sol.assignments.len(), 2, "both schedule when predecessor is feasible (MILP)");
}
