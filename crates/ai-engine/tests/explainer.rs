use ai_engine::explainer::{Explainer, TemplateExplainer};
use ai_engine::types::*;

#[tokio::test]
async fn explains_counts_and_scores() {
    let sol = Solution {
        run_id: 1,
        assignments: vec![ScoredAssignment {
            resource_id: 1,
            task_id: 10,
            resource_name: "Alice".into(),
            task_title: "Backend Task".into(),
            start: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            end: chrono::NaiveDate::from_ymd_opt(2026, 7, 5).unwrap(),
            percent: 1.0,
            score: 0.8,
            rationale: "".into(),
        }],
        unscheduled: vec![11],
        metrics: SolutionMetrics {
            overall: 70.0,
            skill_fit: 80.0,
            scheduled_ratio: 50.0,
            fairness: 0.0,
        },
        status: SolverStatus::Feasible,
    };
    let md = TemplateExplainer
        .explain(&AllocationProblem::default(), &sol)
        .await;
    assert!(md.contains("已分配 **1**"));
    assert!(md.contains("未排期 **1**"));
}

/// LlmExplainer must degrade gracefully to the template when no Ollama provider is
/// reachable (from_env fails or the prompt errors) — never panic, never return empty.
#[cfg(feature = "llm")]
#[tokio::test]
async fn llm_explainer_falls_back_to_template_without_provider() {
    use ai_engine::explainer::{Explainer, TemplateExplainer, llm::LlmExplainer};
    // Point Ollama at a dead port so the prompt can't succeed.
    let sol = Solution {
        run_id: 1,
        assignments: vec![],
        unscheduled: vec![],
        metrics: SolutionMetrics {
            overall: 50.0,
            skill_fit: 50.0,
            scheduled_ratio: 0.0,
            fairness: 100.0,
        },
        status: SolverStatus::Feasible,
    };
    let md = LlmExplainer {
        provider: "ollama".into(),
        model: "qwen2.5:7b".into(),
        base_url: Some("http://127.0.0.1:1".into()),
        api_key: None,
        prompt_template: None,
        preamble: None,
    }
    .explain(&AllocationProblem::default(), &sol)
    .await;
    let fallback = TemplateExplainer
        .explain(&AllocationProblem::default(), &sol)
        .await;
    // Either it fell back immediately (no client) or the prompt errored then fell back;
    // both paths yield the template body.
    assert!(!md.is_empty(), "degradation must not return empty");
    assert!(
        md.contains("优化方案说明") || md == fallback,
        "expected template fallback, got: {md}"
    );
}
