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
