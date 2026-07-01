use ai_engine::scorer::FallbackScorer;
use ai_engine::solver::{GreedySolver, Solver};
use ai_engine::types::*;
use ai_engine::Scorer;
use chrono::NaiveDate;
use std::collections::HashMap;

fn d(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

async fn problem() -> (AllocationProblem, ScoreMatrix) {
    let p = AllocationProblem {
        resources: vec![
            CandidateResource {
                id: 1,
                name: "R1".into(),
                skills: HashMap::from([(1, 4)]),
                tags: vec![],
                daily_capacity_pd: 1.0,
                available_from: None,
                available_to: None,
            },
            CandidateResource {
                id: 2,
                name: "R2".into(),
                skills: HashMap::from([(1, 4)]),
                tags: vec![],
                daily_capacity_pd: 1.0,
                available_from: None,
                available_to: None,
            },
        ],
        tasks: vec![
            CandidateTask {
                id: 10,
                project_id: 1,
                title: "T1".into(),
                estimate_pd: 5.0,
                start: d("2026-07-01"),
                end: d("2026-07-05"),
                priority: 1,
                skill_reqs: vec![SkillReq {
                    skill_id: 1,
                    min_proficiency: 3,
                    is_mandatory: true,
                    weight: 1.0,
                }],
            },
            CandidateTask {
                id: 11,
                project_id: 1,
                title: "T2".into(),
                estimate_pd: 5.0,
                start: d("2026-07-01"),
                end: d("2026-07-05"),
                priority: 2,
                skill_reqs: vec![SkillReq {
                    skill_id: 1,
                    min_proficiency: 3,
                    is_mandatory: true,
                    weight: 1.0,
                }],
            },
        ],
        ..Default::default()
    };
    let m = FallbackScorer::default().matrix(&p).await;
    (p, m)
}

#[tokio::test]
async fn schedules_both_tasks_to_distinct_resources() {
    let (p, m) = problem().await;
    let sol = GreedySolver.solve(&p, &m);
    assert_eq!(sol.assignments.len(), 2);
    assert_eq!(sol.unscheduled.len(), 0);
    let mut rids: Vec<i64> = sol.assignments.iter().map(|a| a.resource_id).collect();
    rids.sort();
    assert_eq!(rids, vec![1, 2]); // balanced across the two resources
}

#[tokio::test]
async fn unscheduled_when_no_feasible_resource() {
    let (mut p, m) = problem().await;
    p.resources = vec![CandidateResource {
        id: 1,
        name: "R1".into(),
        skills: HashMap::from([(1, 4)]),
        tags: vec![],
        daily_capacity_pd: 1.0,
                available_from: None,
                available_to: None,
    }];
    let sol = GreedySolver.solve(&p, &m);
    // one task fills R1 to 1.0; the other can't fit -> unscheduled
    assert_eq!(sol.assignments.len(), 1);
    assert_eq!(sol.unscheduled.len(), 1);
}

#[test]
fn skips_resource_outside_availability_even_with_better_score() {
    let p = AllocationProblem {
        resources: vec![
            CandidateResource {
                id: 1,
                name: "Unavailable".into(),
                skills: HashMap::new(),
                tags: vec![],
                daily_capacity_pd: 1.0,
                available_from: Some(d("2026-08-01")),
                available_to: Some(d("2026-08-31")),
            },
            CandidateResource {
                id: 2,
                name: "Available".into(),
                skills: HashMap::new(),
                tags: vec![],
                daily_capacity_pd: 1.0,
                available_from: Some(d("2026-07-01")),
                available_to: Some(d("2026-07-31")),
            },
        ],
        tasks: vec![CandidateTask {
            id: 10,
            project_id: 1,
            title: "T".into(),
            estimate_pd: 0.5,
            start: d("2026-07-01"),
            end: d("2026-07-01"),
            priority: 1,
            skill_reqs: vec![],
        }],
        ..Default::default()
    };
    let scores = HashMap::from([((1, 10), 1.0), ((2, 10), 0.1)]);
    let sol = GreedySolver.solve(&p, &scores);

    assert_eq!(sol.assignments.len(), 1);
    assert_eq!(sol.assignments[0].resource_id, 2);
    assert!(sol.unscheduled.is_empty());
}

#[test]
fn daily_capacity_factor_controls_assignment_percent() {
    let p = AllocationProblem {
        resources: vec![CandidateResource {
            id: 1,
            name: "HalfTime".into(),
            skills: HashMap::new(),
            tags: vec![],
            daily_capacity_pd: 1.0,
            available_from: None,
            available_to: None,
        }],
        tasks: vec![CandidateTask {
            id: 10,
            project_id: 1,
            title: "T".into(),
            estimate_pd: 0.5,
            start: d("2026-07-01"),
            end: d("2026-07-02"),
            priority: 1,
            skill_reqs: vec![],
        }],
        daily_capacity: vec![
            DailyCapacity { resource_id: 1, day: d("2026-07-01"), factor: 0.5 },
            DailyCapacity { resource_id: 1, day: d("2026-07-02"), factor: 0.5 },
        ],
        ..Default::default()
    };
    let scores = HashMap::from([((1, 10), 1.0)]);
    let sol = GreedySolver.solve(&p, &scores);

    assert_eq!(sol.assignments.len(), 1);
    assert!((sol.assignments[0].percent - 0.5).abs() < 1e-9);
}

#[tokio::test]
async fn balances_equal_score_tasks_across_resources() {
    // Regression: with the load-balance tiebreaker, two tied-score tasks that COULD
    // both pile onto one resource (0.4 each ≤ 1.0 cap) must split across R1/R2.
    // The old score-only sort put both on R1.
    let (mut p, m) = problem().await;
    for t in &mut p.tasks {
        t.estimate_pd = 2.0;
    }
    let sol = GreedySolver.solve(&p, &m);
    let mut rids: Vec<i64> = sol.assignments.iter().map(|a| a.resource_id).collect();
    rids.sort();
    assert_eq!(rids, vec![1, 2], "tied-score tasks should balance, not pile up");
}

#[tokio::test]
async fn budget_hard_gate_blocks_overflow() {
    // Budget is an unconditional hard gate: two 2.0-PD tasks vs a 3.0 budget ⇒ only one fits.
    let (mut p, m) = problem().await;
    for t in &mut p.tasks {
        t.estimate_pd = 2.0;
    }
    p.budget_pd = Some(3.0);
    let sol = GreedySolver.solve(&p, &m);
    assert_eq!(sol.assignments.len(), 1);
    assert_eq!(sol.unscheduled.len(), 1);
}

#[tokio::test]
async fn fairness_nonzero_when_load_balanced() {
    // Two resources with equal committed load ⇒ Jain index ≈ 1.0 (×100).
    // (Previously fairness was a hardcoded 0.0.)
    let (p, m) = problem().await;
    let sol = GreedySolver.solve(&p, &m);
    assert!(
        sol.metrics.fairness > 99.0,
        "expected ~100 fairness, got {}",
        sol.metrics.fairness
    );
}

/// Two resources, two tasks each 2.0 PD on a 5-day window. R1 scores 0.80 on BOTH tasks;
/// R2 scores 0.79 on both. With balance weight ≈ 0, greedy (score desc, load-asc tiebreak)
/// ships both tasks to R1 (0.80 > 0.79, no tie to break) ⇒ R1 load 0.8, R2 load 0.0, fairness
/// low. With balance weight dominant, the fairness-aware rank_score must pull one task to R2
/// ⇒ fairness rises sharply. This is the case the pure score-sort greedy provably misses.
#[test]
fn balance_weight_raises_fairness_above_score_only() {
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
                id: 1, project_id: 1, title: "A".into(), estimate_pd: 2.0,
                start: d("2026-07-01"), end: d("2026-07-05"), priority: 1, skill_reqs: vec![],
            },
            CandidateTask {
                id: 2, project_id: 1, title: "B".into(), estimate_pd: 2.0,
                start: d("2026-07-01"), end: d("2026-07-05"), priority: 2, skill_reqs: vec![],
            },
        ],
        ..Default::default()
    };
    let m: HashMap<(i64, i64), f64> = HashMap::from([
        ((1, 1), 0.80), ((1, 2), 0.80),
        ((2, 1), 0.79), ((2, 2), 0.79),
    ]);

    // Balance weight = 0 ⇒ pure score sort ⇒ both on R1 (0.80 > 0.79), R2 idle.
    let mut p_off = p.clone();
    p_off.weights = ObjectiveWeights { skill_fit: 1.0, balance: 0.0, budget: 0.0 };
    let sol_off = GreedySolver.solve(&p_off, &m);
    let rids_off: Vec<i64> = { let mut v: Vec<i64> = sol_off.assignments.iter().map(|a| a.resource_id).collect(); v.sort(); v };
    assert_eq!(rids_off, vec![1, 1], "w_balance=0 must match the old score-only behavior (both on R1)");

    // Balance weight dominant ⇒ one task each ⇒ fairness jumps.
    let mut p_on = p.clone();
    p_on.weights = ObjectiveWeights { skill_fit: 0.1, balance: 0.9, budget: 0.0 };
    let sol_on = GreedySolver.solve(&p_on, &m);
    let rids_on: Vec<i64> = { let mut v: Vec<i64> = sol_on.assignments.iter().map(|a| a.resource_id).collect(); v.sort(); v };
    assert_eq!(rids_on, vec![1, 2], "w_balance dominant must split tasks across R1/R2");
    assert!(
        sol_on.metrics.fairness > sol_off.metrics.fairness + 1.0,
        "balance-dominant fairness {} must exceed score-only {}",
        sol_on.metrics.fairness, sol_off.metrics.fairness
    );
}

/// Dependency cascade (greedy): T2 depends on T1. When T1 is infeasible (mandatory skill
/// unmet), T1 is unscheduled ⇒ T2 must ALSO be unscheduled (it can't be scheduled without its
/// predecessor). When T1 is feasible, both schedule.
#[test]
fn dependency_cascades_unscheduled_when_predecessor_infeasible() {
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
            ..Default::default()
        }
    };
    let m: HashMap<(i64, i64), f64> = HashMap::from([((1, 1), 0.5), ((1, 2), 0.5)]);

    // Predecessor infeasible ⇒ both unscheduled (cascade).
    let sol = GreedySolver.solve(&make(false), &m);
    let mut unsched = sol.unscheduled.clone(); unsched.sort();
    assert_eq!(unsched, vec![1, 2], "T2 must cascade-unschedule when T1 is infeasible");

    // Predecessor feasible ⇒ both scheduled.
    let sol = GreedySolver.solve(&make(true), &m);
    assert_eq!(sol.assignments.len(), 2, "both schedule when the predecessor is feasible");
    let mut sched: Vec<i64> = sol.assignments.iter().map(|a| a.task_id).collect(); sched.sort();
    assert_eq!(sched, vec![1, 2]);
}
