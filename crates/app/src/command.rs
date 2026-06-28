use crate::error::AppError;
use crate::service::projects;
use crate::state::AppState;
use db::models::Project;

#[tauri::command]
pub async fn create_project(
    state: tauri::State<'_, AppState>,
    name: String, description: Option<String>,
    start: Option<String>, end: Option<String>,
    priority: i64, budget_pd: Option<f64>,
) -> Result<i64, AppError> {
    projects::ProjectsService::create(
        &state.pool, &name, description.as_deref(),
        start.as_deref(), end.as_deref(), priority, budget_pd.unwrap_or(0.0)).await
}

#[tauri::command]
pub async fn list_projects(state: tauri::State<'_, AppState>) -> Result<Vec<Project>, AppError> {
    projects::ProjectsService::list(&state.pool).await
}