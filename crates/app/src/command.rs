use crate::error::AppError;
use crate::service::{catalog, projects, tasks};
use crate::state::AppState;
use db::models::{KanbanTask, Project, Skill, Tag};

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

#[tauri::command]
pub async fn ensure_skill(state: tauri::State<'_, AppState>, name: String) -> Result<i64, AppError> {
    catalog::CatalogService::ensure_skill(&state.pool, &name).await
}
#[tauri::command]
pub async fn list_skills(state: tauri::State<'_, AppState>) -> Result<Vec<Skill>, AppError> {
    catalog::CatalogService::list_skills(&state.pool).await
}
#[tauri::command]
pub async fn ensure_tag(state: tauri::State<'_, AppState>, name: String, color: Option<String>) -> Result<i64, AppError> {
    catalog::CatalogService::ensure_tag(&state.pool, &name, color.as_deref()).await
}
#[tauri::command]
pub async fn list_tags(state: tauri::State<'_, AppState>) -> Result<Vec<Tag>, AppError> {
    catalog::CatalogService::list_tags(&state.pool).await
}

#[tauri::command]
pub async fn create_task(
    state: tauri::State<'_, AppState>,
    project_id: i64, title: String, description: Option<String>,
    estimate_pd: f64, start: Option<String>, end: Option<String>,
    is_long_term: bool, sort_order: i64,
    skill_reqs: Vec<(i64, i64, bool, f64)>, tag_ids: Vec<i64>,
) -> Result<i64, AppError> {
    tasks::TasksService::create(
        &state.pool, project_id, &title, description.as_deref(), estimate_pd,
        start.as_deref(), end.as_deref(), is_long_term, sort_order, &skill_reqs, &tag_ids).await
}

#[tauri::command]
pub async fn set_task_status(state: tauri::State<'_, AppState>, id: i64, status: String) -> Result<(), AppError> {
    tasks::TasksService::set_status(&state.pool, id, &status).await
}

#[tauri::command]
pub async fn kanban_tasks(state: tauri::State<'_, AppState>, project_id: i64) -> Result<Vec<KanbanTask>, AppError> {
    tasks::TasksService::kanban(&state.pool, project_id).await
}