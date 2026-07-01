use ai_engine::types::{Suggestion, SuggestionItem};
use chrono::NaiveDate;

#[test]
fn suggestion_deserializes_with_kind_tag() {
    let json = r#"{"kind":"widen_window","task_id":5,"new_start":"2026-07-01","new_end":"2026-07-20"}"#;
    let s: Suggestion = serde_json::from_str(json).unwrap();
    assert_eq!(s.target_task_id(), Some(5));
    match s {
        Suggestion::WidenWindow { new_end, .. } => {
            assert_eq!(new_end, NaiveDate::from_ymd_opt(2026, 7, 20).unwrap());
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn suggestion_rejects_unknown_kind() {
    let json = r#"{"kind":"bogus","task_id":5}"#;
    let r: Result<Suggestion, _> = serde_json::from_str(json);
    assert!(r.is_err(), "unknown kind must be rejected");
}

#[test]
fn suggestion_item_roundtrips() {
    let item = SuggestionItem {
        id: None,
        suggestion: Suggestion::DropDependency { task_id: 3, predecessor_id: 1 },
        rationale_md: "解依赖以放行 T3".into(),
        status: "proposed".into(),
    };
    let json = serde_json::to_string(&item).unwrap();
    let back: SuggestionItem = serde_json::from_str(&json).unwrap();
    assert_eq!(back, item);
}

use ai_engine::advisor::{Advisor, NoAdvisor};
use ai_engine::types::{AllocationProblem, Solution};

#[tokio::test]
async fn no_advisor_returns_empty() {
    let advisor = NoAdvisor;
    let items = advisor.advise(&AllocationProblem::default(), &Solution::default()).await;
    assert!(items.is_empty(), "NoAdvisor must return zero suggestions");
}

#[cfg(feature = "llm")]
mod llm_parse_tests {
    use ai_engine::advisor::llm::parse_suggestions;
    use ai_engine::types::*;
    use chrono::NaiveDate;

    fn prob_with_task_resource() -> AllocationProblem {
        AllocationProblem {
            tasks: vec![CandidateTask {
                id: 5, project_id: 1, title: "T5".into(), estimate_pd: 3.0,
                start: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 7, 7).unwrap(),
                priority: 3, skill_reqs: vec![],
            }],
            resources: vec![CandidateResource {
                id: 2, name: "Bob".into(), skills: Default::default(), tags: vec![],
                daily_capacity_pd: 1.0, available_from: None, available_to: None,
            }],
            ..Default::default()
        }
    }

    #[test]
    fn parse_invalid_json_returns_empty() {
        let p = AllocationProblem::default();
        assert!(parse_suggestions("not json at all", &p).is_empty());
    }

    #[test]
    fn parse_drops_unknown_kind_keeps_valid() {
        let p = prob_with_task_resource();
        let txt = r#"[
            {"kind":"bogus","task_id":5},
            {"kind":"widen_window","task_id":5,"new_start":"2026-07-01","new_end":"2026-07-20","rationale":"x"}
        ]"#;
        let v = parse_suggestions(txt, &p);
        assert_eq!(v.len(), 1);
        assert!(matches!(v[0].suggestion, Suggestion::WidenWindow { .. }));
    }

    #[test]
    fn parse_strips_code_fence() {
        let p = prob_with_task_resource();
        let txt = "```json\n[{\"kind\":\"widen_window\",\"task_id\":5,\"new_start\":\"2026-07-01\",\"new_end\":\"2026-07-20\",\"rationale\":\"x\"}]\n```";
        assert_eq!(parse_suggestions(txt, &p).len(), 1);
    }
}
