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
