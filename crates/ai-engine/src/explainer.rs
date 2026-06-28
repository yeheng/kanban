use crate::types::*;
use async_trait::async_trait;
#[async_trait]
pub trait Explainer: Send + Sync {
    async fn explain(&self, problem: &AllocationProblem, sol: &Solution) -> String;
}
