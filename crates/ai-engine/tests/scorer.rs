use ai_engine::scorer::{FallbackScorer, Scorer};
use ai_engine::types::*;

fn res(id: i64, skills: &[(i64, i64)]) -> CandidateResource {
    CandidateResource {
        id,
        name: format!("R{}", id),
        skills: skills.iter().cloned().collect(),
        tags: vec![],
        daily_capacity_pd: 1.0,
        available_from: None,
        available_to: None,
    }
}
fn res_with_tags(id: i64, tags: Vec<&str>) -> CandidateResource {
    CandidateResource {
        id,
        name: format!("R{}", id),
        skills: std::collections::HashMap::new(),
        tags: tags.into_iter().map(String::from).collect(),
        daily_capacity_pd: 1.0,
        available_from: None,
        available_to: None,
    }
}
fn task(id: i64, reqs: &[(i64, i64, bool)]) -> CandidateTask {
    CandidateTask {
        id,
        project_id: 1,
        title: "build api".into(),
        estimate_pd: 5.0,
        start: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
        end: chrono::NaiveDate::from_ymd_opt(2026, 7, 7).unwrap(),
        priority: 5,
        skill_reqs: reqs
            .iter()
            .map(|(s, p, m)| SkillReq {
                skill_id: *s,
                min_proficiency: *p,
                is_mandatory: *m,
                weight: 1.0,
            })
            .collect(),
    }
}

#[tokio::test]
async fn mandatory_unmet_scores_zero() {
    let s = FallbackScorer::default();
    let r = res(1, &[(1, 2)]); // proficiency 2
    let t = task(10, &[(1, 3, true)]); // needs 3
    assert_eq!(s.score(&r, &t).await, 0.0);
}

#[tokio::test]
async fn matched_skill_scores_higher_than_mismatched() {
    let s = FallbackScorer::default();
    let good = res(1, &[(1, 4)]);
    let weak = res(2, &[(2, 4)]);
    let t = task(10, &[(1, 3, true)]);
    assert!(s.score(&good, &t).await > s.score(&weak, &t).await); // weak returns 0 (mandatory fail)
}

/// Resource tags feed the FallbackScorer's Jaccard token set. A resource whose tags
/// overlap the task title scores higher than one with no overlap (no skills involved, so
/// this isolates the tag-token path that was previously dead because tags were vec![]).
#[tokio::test]
async fn resource_tags_raise_fallback_score() {
    let s = FallbackScorer::default();
    let t = CandidateTask {
        id: 10,
        project_id: 1,
        title: "rust backend".into(),
        estimate_pd: 1.0,
        start: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
        end: chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap(),
        priority: 1,
        skill_reqs: vec![],
    };
    let with_tag = s.score(&res_with_tags(1, vec!["rust"]), &t).await;
    let no_tag = s.score(&res_with_tags(2, vec![]), &t).await;
    assert!(with_tag > no_tag, "tag overlap should raise score: {} > {}", with_tag, no_tag);
    assert!(with_tag > 0.0, "shared tag must produce nonzero score");
}
