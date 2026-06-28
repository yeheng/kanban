use ai_engine::explainer::TemplateExplainer;
use ai_engine::scorer::FallbackScorer;
use ai_engine::solver::GreedySolver;
use ai_engine::types::*;
use ai_engine::OptimizationEngine;
use chrono::NaiveDate;
use std::sync::Arc;

#[tokio::test]
async fn engine_pipeline_produces_plan_with_explanation() {
    let p = AllocationProblem {
        run_id: 42,
        resources: vec![CandidateResource {
            id: 1,
            name: "R1".into(),
            skills: std::collections::HashMap::from([(1, 4)]),
            tags: vec![],
            daily_capacity_pd: 1.0,
        }],
        tasks: vec![CandidateTask {
            id: 10,
            project_id: 1,
            title: "T1".into(),
            estimate_pd: 5.0,
            start: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            end: NaiveDate::from_ymd_opt(2026, 7, 5).unwrap(),
            priority: 1,
            skill_reqs: vec![SkillReq {
                skill_id: 1,
                min_proficiency: 3,
                is_mandatory: true,
                weight: 1.0,
            }],
        }],
        ..Default::default()
    };

    let engine = OptimizationEngine {
        scorer: Arc::new(FallbackScorer),
        solver: Arc::new(GreedySolver),
        explainer: Arc::new(TemplateExplainer),
    };
    let plan = engine.optimize(&p).await;

    assert_eq!(plan.solution.run_id, 42);
    assert_eq!(plan.solution.assignments.len(), 1);
    assert_eq!(plan.solution.assignments[0].resource_id, 1);
    assert!(plan.explanation_md.contains("优化方案说明"));
}
