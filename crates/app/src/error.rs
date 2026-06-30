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
            InvalidRatio(_)
            | InvalidValue { .. }
            | InvalidStatus(_)
            | InvalidInput(_)
            | InvalidDateWindow => AppError::validation(e.to_string()),
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
            db::DbError::Sqlx(inner) => classify_sqlx(&inner),
            other => AppError::db(other.to_string()),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self { classify_sqlx(&e) }
}

/// Single source of truth for turning a sqlx error into an `AppError`.
///
/// SQLite reports CHECK / NOT NULL / FOREIGN KEY / UNIQUE constraint failures *and* trigger
/// `RAISE(ABORT, ...)` rejections with primary result code 19 (`SQLITE_CONSTRAINT`); the
/// extended codes (275 CHECK, 787 FK, 1299 NOTNULL, 1811 TRIGGER, 2067 UNIQUE, ...) all share
/// the low byte 19. Those are user-correctable business-rule rejections (e.g. an allocation
/// outside its task window), so they map to `DOMAIN` (422) and surface the DB message — NOT a
/// blanket `DB` 500 that reads as a server crash and pollutes error telemetry. Everything else
/// (connection loss, protocol errors, ...) stays a genuine `DB` failure.
fn classify_sqlx(e: &sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = e {
        if let Some(code) = db_err.code() {
            if code.parse::<i64>().map(|n| n % 256 == 19).unwrap_or(false) {
                return AppError::domain(db_err.message().to_string());
            }
        }
    }
    AppError::db(e.to_string())
}
