use crate::explainer::Explainer;
use crate::scorer::Scorer;
use crate::solver::Solver;
use crate::types::*;
use std::sync::Arc;

pub struct OptimizationEngine {
    pub scorer: Arc<dyn Scorer>,
    pub solver: Arc<dyn Solver>,
    pub explainer: Arc<dyn Explainer>,
}

impl OptimizationEngine {
    pub async fn optimize(&self, problem: &AllocationProblem) -> OptimizedPlan {
        let scores = self.scorer.matrix(problem).await;
        let solution = self.solver.solve(problem, &scores);
        let explanation_md = self.explainer.explain(problem, &solution).await;
        OptimizedPlan {
            solution,
            explanation_md,
        }
    }
}
