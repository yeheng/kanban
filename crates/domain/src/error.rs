use thiserror::Error;

/// Domain errors. Domain crate has NO serde dependency (design §2.5/§6.4).
/// Variations beyond Phase 1 (SkillMismatch, Solver, InsufficientCapacity) are
/// defined now so later phases don't reshape the enum.
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid ratio {0}")]
    InvalidRatio(f64),
    #[error("invalid date window")]
    InvalidDateWindow,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("dependency cycle detected involving task {0}")]
    DependencyCycle(i64),
    #[error("dependency violation: task {task_id} conflicts with task {related_task_id}")]
    DependencyViolation { task_id: i64, related_task_id: i64 },
    #[error("insufficient capacity for resource {resource_id}: shortfall {shortfall_pd} PD")]
    InsufficientCapacity { resource_id: i64, shortfall_pd: f64 },
    #[error("skill mismatch on task {task_id}: missing skill {skill_id}")]
    SkillMismatch { task_id: i64, skill_id: i64 },
    #[error("solver error: {0}")]
    Solver(String),
}
