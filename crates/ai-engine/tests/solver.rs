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
