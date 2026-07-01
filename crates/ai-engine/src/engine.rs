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
    #[tracing::instrument(skip(self, problem), fields(run_id = problem.run_id, resources = problem.resources.len(), tasks = problem.tasks.len()))]
    pub async fn optimize(&self, problem: &AllocationProblem) -> OptimizedPlan {
        tracing::info!("starting optimization");
        let scores = self.scorer.matrix(problem).await;
        let solution = self.solver.solve(problem, &scores);
        let explanation_md = self.explainer.explain(problem, &solution).await;
        tracing::info!(
            assignments = solution.assignments.len(),
            unscheduled = solution.unscheduled.len(),
            overall = solution.metrics.overall,
            "optimization completed"
        );
        OptimizedPlan {
            solution,
            explanation_md,
        }
    }
}
