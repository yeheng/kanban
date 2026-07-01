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

use ai_engine::advisor::apply_suggestions;
use ai_engine::types::*;

fn task(id: i64, start: &str, end: &str) -> CandidateTask {
    CandidateTask { id, project_id: 1, title: format!("T{id}"), estimate_pd: 3.0,
        start: NaiveDate::parse_from_str(start, "%Y-%m-%d").unwrap(),
        end: NaiveDate::parse_from_str(end, "%Y-%m-%d").unwrap(),
        priority: 3, skill_reqs: vec![] }
}
fn resource(id: i64) -> CandidateResource {
    CandidateResource { id, name: format!("R{id}"), skills: Default::default(), tags: vec![],
        daily_capacity_pd: 1.0, available_from: None, available_to: None }
}
fn date(s: &str) -> NaiveDate { NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap() }

#[test]
fn widen_window_only_relaxes_not_narrows() {
    let mut p = AllocationProblem { tasks: vec![task(5, "2026-07-03", "2026-07-07")], ..Default::default() };
    let _ = apply_suggestions(&mut p, &[
        Suggestion::WidenWindow { task_id: 5, new_start: date("2026-07-04"), new_end: date("2026-07-06") },
    ]);
    assert_eq!(p.tasks[0].start, date("2026-07-03"));
    assert_eq!(p.tasks[0].end, date("2026-07-07"));
}

#[test]
fn widen_window_relaxes_outward() {
    let mut p = AllocationProblem { tasks: vec![task(5, "2026-07-03", "2026-07-07")], ..Default::default() };
    let _ = apply_suggestions(&mut p, &[
        Suggestion::WidenWindow { task_id: 5, new_start: date("2026-07-01"), new_end: date("2026-07-20") },
    ]);
    assert_eq!(p.tasks[0].start, date("2026-07-01"));
    assert_eq!(p.tasks[0].end, date("2026-07-20"));
}

#[test]
fn drop_dependency_removes_edge() {
    let mut p = AllocationProblem {
        dependencies: vec![TaskDependency { task_id: 3, predecessor_id: 1 }],
        ..Default::default()
    };
    let _ = apply_suggestions(&mut p, &[Suggestion::DropDependency { task_id: 3, predecessor_id: 1 }]);
    assert!(p.dependencies.is_empty());
}

#[test]
fn upsert_skill_overrides_proficiency() {
    let mut p = AllocationProblem { resources: vec![resource(2)], ..Default::default() };
    let _ = apply_suggestions(&mut p, &[Suggestion::UpsertResourceSkill { resource_id: 2, skill_id: 10, new_proficiency: 4 }]);
    assert_eq!(p.resources[0].skills.get(&10), Some(&4));
}

#[test]
fn change_resource_capacity_overrides() {
    let mut p = AllocationProblem { resources: vec![resource(2)], ..Default::default() };
    let _ = apply_suggestions(&mut p, &[Suggestion::ChangeResourceCapacity { resource_id: 2, new_daily_capacity_pd: 1.2 }]);
    assert!((p.resources[0].daily_capacity_pd - 1.2).abs() < 1e-9);
}

#[test]
fn add_resource_and_advisory_kinds_returned_or_skipped() {
    let mut p = AllocationProblem { tasks: vec![task(5, "2026-07-01", "2026-07-07")], ..Default::default() };
    let pending = apply_suggestions(&mut p, &[
        Suggestion::AddResource { resource_id: 9 },
        Suggestion::SwapResource { task_id: 5, new_resource_id: 2 },
        Suggestion::ChangePercent { task_id: 5, new_percent: 0.5 },
    ]);
    assert_eq!(pending.len(), 1);
    assert!(matches!(pending[0], Suggestion::AddResource { resource_id: 9 }));
}

#[test]
fn widen_resource_window_only_relaxes() {
    let mut p = AllocationProblem { resources: vec![CandidateResource {
        id: 2, name: "R2".into(), skills: Default::default(), tags: vec![],
        daily_capacity_pd: 1.0,
        available_from: Some(date("2026-07-05")), available_to: Some(date("2026-07-10")),
    }], ..Default::default() };
    let _ = apply_suggestions(&mut p, &[
        Suggestion::WidenResourceWindow { resource_id: 2, new_available_from: date("2026-07-01"), new_available_to: date("2026-07-20") },
    ]);
    assert_eq!(p.resources[0].available_from, Some(date("2026-07-01")));
    assert_eq!(p.resources[0].available_to, Some(date("2026-07-20")));
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
