use ai_engine::explainer::{Explainer, TemplateExplainer};
use ai_engine::types::*;

#[tokio::test]
async fn explains_counts_and_scores() {
    let sol = Solution {
        run_id: 1,
        assignments: vec![ScoredAssignment { resource_id: 1, task_id: 10, start: chrono::NaiveDate::from_ymd_opt(2026,7,1).unwrap(), end: chrono::NaiveDate::from_ymd_opt(2026,7,5).unwrap(), percent: 1.0, score: 0.8, rationale: "".into() }],
        unscheduled: vec![11],
        metrics: SolutionMetrics { overall: 70.0, skill_fit: 80.0, utilization: 50.0, fairness: 0.0 },
    };
    let md = TemplateExplainer.explain(&AllocationProblem::default(), &sol).await;
    assert!(md.contains("已分配 **1**"));
    assert!(md.contains("未排期 **1**"));
}
