use crate::error::HttpError;
use crate::state::AppState;
use app::service::reports::{ReportKind, ReportService};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        // GET /api/reports/catalog → report roadmap with available formats (design §8 / G5).
        .route("/api/reports/catalog", get(catalog))
        // GET /api/reports/{kind}?project_id=&start=&end=&format=  → file download (bytes).
        .route("/api/reports/{kind}", get(export_report))
        // GET /api/reports/snapshot?start=&end=  → pretty JSON (also downloadable via the view).
        .route("/api/reports/snapshot", get(snapshot))
}

#[tracing::instrument]
async fn catalog() -> Json<Vec<app::service::reports::ReportCatalogEntry>> {
    tracing::debug!("report catalog");
    Json(ReportService::report_catalog())
}

#[derive(Debug, Deserialize)]
struct ReportQuery {
    project_id: Option<i64>,
    start: String,
    end: String,
    format: String,
}

#[tracing::instrument(skip(state), fields(kind = %kind, format = %q.format, project_id = q.project_id))]
async fn export_report(
    State(state): State<AppState>,
    Path(kind): Path<String>,
    Query(q): Query<ReportQuery>,
) -> Result<Response, HttpError> {
    tracing::debug!("exporting report");
    let kind = parse_kind(&kind)?;
    tracing::debug!(?kind, "building report");
    let table = ReportService::build(&state.pool, kind, q.project_id, &q.start, &q.end).await?;
    let (bytes, mime, ext) = match q.format.as_str() {
        "csv" => (ReportService::to_csv(&table)?, "text/csv; charset=utf-8", "csv"),
        "xlsx" => (ReportService::to_xlsx(&table)?, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", "xlsx"),
        // PDF is feature-gated on the app crate (app/pdf). The default server binary has no PDF
        // generator compiled in; surface a clear error. Enable by forwarding the feature if needed.
        "pdf" => {
            tracing::warn!("pdf export requested but app/pdf feature not compiled");
            return Err(domain::DomainError::Solver(
                "PDF export requires the app/pdf feature (not compiled into this server)".into()).into());
        }
        other => {
            tracing::warn!(format = other, "unsupported report format");
            return Err(domain::DomainError::Solver(format!("unsupported format: {other}")).into());
        }
    };
    let filename = format!("{}.{}", slug(&table.title), ext);
    tracing::info!(filename = %filename, rows = table.rows.len(), columns = table.columns.len(), "exported report");
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, mime.to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{filename}\"")),
        ],
        Body::from(bytes),
    ).into_response())
}

#[derive(Debug, Deserialize)]
struct SnapshotQuery { start: String, end: String }

#[tracing::instrument(skip(state))]
async fn snapshot(
    State(state): State<AppState>,
    Query(q): Query<SnapshotQuery>,
) -> Result<Json<serde_json::Value>, HttpError> {
    tracing::debug!("report snapshot");
    let json = ReportService::snapshot_json(&state.pool, &q.start, &q.end).await?;
    let value: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| app::error::AppError::internal(e.to_string()))?;
    Ok(Json(value))
}

fn parse_kind(s: &str) -> Result<ReportKind, HttpError> {
    match s {
        "ResourceUtilization" => Ok(ReportKind::ResourceUtilization),
        "TeamUtilization" => Ok(ReportKind::TeamUtilization),
        "ProjectBurn" => Ok(ReportKind::ProjectBurn),
        "AiDecisions" => Ok(ReportKind::AiDecisions),
        "Cost" => Ok(ReportKind::Cost),
        other => {
            tracing::warn!(kind = other, "unknown report kind");
            Err(domain::DomainError::Solver(format!("unknown report kind: {other}")).into())
        }
    }
}

fn slug(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' }).collect()
}
