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
    let s = FallbackScorer;
    let r = res(1, &[(1, 2)]); // proficiency 2
    let t = task(10, &[(1, 3, true)]); // needs 3
    assert_eq!(s.score(&r, &t).await, 0.0);
}

#[tokio::test]
async fn matched_skill_scores_higher_than_mismatched() {
    let s = FallbackScorer;
    let good = res(1, &[(1, 4)]);
    let weak = res(2, &[(2, 4)]);
    let t = task(10, &[(1, 3, true)]);
    assert!(s.score(&good, &t).await > s.score(&weak, &t).await); // weak returns 0 (mandatory fail)
}
