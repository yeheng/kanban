use crate::types::*;
pub trait Solver: Send + Sync {
    fn solve(&self, problem: &AllocationProblem, scores: &ScoreMatrix) -> Solution;
}
