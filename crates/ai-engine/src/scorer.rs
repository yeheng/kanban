use crate::types::*;
use async_trait::async_trait;

#[async_trait]
pub trait Scorer: Send + Sync {
    async fn score(&self, r: &CandidateResource, t: &CandidateTask) -> f64; // 0..1
    async fn matrix(&self, problem: &AllocationProblem) -> ScoreMatrix {
        let mut m = ScoreMatrix::new();
        for r in &problem.resources {
            for t in &problem.tasks {
                m.insert((r.id, t.id), self.score(r, t).await);
            }
        }
        m
    }
}

/// Deterministic offline scorer (no LLM). Keyword-Jaccard over resource
/// skills+tags vs task title+skill_reqs (coarse proficiency buckets), plus a
/// proficiency bonus. Mandatory skills unmet ⇒ 0 (hard filter reflected in score).
pub struct FallbackScorer;

impl FallbackScorer {
    fn tokens(r: &CandidateResource) -> Vec<String> {
        let mut v: Vec<String> = r.tags.to_vec();
        for (sid, prof) in &r.skills { v.push(format!("skill{}p{}", sid, prof / 3)); } // coarse bucket
        v.into_iter().map(|s| s.to_lowercase()).collect()
    }
    fn task_tokens(t: &CandidateTask) -> Vec<String> {
        let mut v: Vec<String> = t.title.split_whitespace().map(|s| s.to_lowercase()).collect();
        for req in &t.skill_reqs { v.push(format!("skill{}p{}", req.skill_id, req.min_proficiency / 3)); }
        v
    }
    fn jaccard(a: &[String], b: &[String]) -> f64 {
        let sa: std::collections::HashSet<&String> = a.iter().collect();
        let sb: std::collections::HashSet<&String> = b.iter().collect();
        if sa.is_empty() && sb.is_empty() { return 0.0; }
        let inter = sa.intersection(&sb).count() as f64;
        let union = sa.union(&sb).count() as f64;
        inter / union
    }
}

#[async_trait]
impl Scorer for FallbackScorer {
    async fn score(&self, r: &CandidateResource, t: &CandidateTask) -> f64 {
        // mandatory skills must be met at min proficiency, else 0 (hard filter reflected in score)
        for req in &t.skill_reqs {
            if req.is_mandatory {
                match r.skills.get(&req.skill_id) {
                    Some(p) if *p >= req.min_proficiency => {}
                    _ => return 0.0,
                }
            }
        }
        let base = Self::jaccard(&Self::tokens(r), &Self::task_tokens(t));
        // proficiency bonus: avg proficiency on required skills / 5
        let bonus = if t.skill_reqs.is_empty() { 0.0 } else {
            let s: f64 = t.skill_reqs.iter().filter_map(|req| r.skills.get(&req.skill_id)).map(|p| *p as f64).sum();
            s / (t.skill_reqs.len() as f64 * 5.0)
        };
        (base * 0.6 + bonus * 0.4).clamp(0.0, 1.0)
    }
}
