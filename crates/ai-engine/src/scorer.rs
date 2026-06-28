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
