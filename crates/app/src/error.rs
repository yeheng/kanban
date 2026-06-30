use serde::Serialize;

/// IPC-level error. Serializes to `{ "code": "...", "detail": "..." }` (design §6.4).
/// DomainError (no serde) is mapped here, never embedded via #[from].
#[derive(Debug, Serialize)]
pub struct AppError {
    pub code: &'static str,
    pub detail: String,
}

impl AppError {
    pub const fn validation(detail: String) -> Self { Self { code: "VALIDATION", detail } }
    pub const fn not_found(detail: String) -> Self { Self { code: "NOT_FOUND", detail } }
    pub const fn domain(detail: String) -> Self { Self { code: "DOMAIN", detail } }
    pub const fn db(detail: String) -> Self { Self { code: "DB", detail } }
    pub const fn internal(detail: String) -> Self { Self { code: "INTERNAL", detail } }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.detail)
    }
}
impl std::error::Error for AppError {}

/// Explicit DomainError -> AppError mapping (no #[from] embedding; design §6.4).
impl From<domain::DomainError> for AppError {
    fn from(e: domain::DomainError) -> Self {
        use domain::DomainError::*;
        match e {
            InvalidRatio(_) | InvalidDateWindow => AppError::validation(e.to_string()),
            NotFound(s) => AppError::not_found(s),
            DependencyCycle(_)
            | DependencyViolation { .. }
            | InsufficientCapacity { .. }
            | SkillMismatch { .. }
            | Solver(_) => {
                AppError::domain(e.to_string())
            }
        }
    }
}

impl From<db::DbError> for AppError {
    fn from(e: db::DbError) -> Self {
        match e {
            db::DbError::NotFound => AppError::not_found("entity".into()),
            other => AppError::db(other.to_string()),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self { AppError::db(e.to_string()) }
}
