use crate::state::AppState;
use axum::Router;

pub mod allocations;
pub mod calendar;
pub mod catalog;
pub mod gantt;
pub mod optimization;
pub mod projects;
pub mod reports;
pub mod resources;
pub mod settings;
pub mod tasks;
pub mod teams;
pub mod workload;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .merge(projects::router())
        .merge(catalog::router())
        .merge(tasks::router())
        .merge(resources::router())
        .merge(teams::router())
        .merge(workload::router())
        .merge(allocations::router())
        .merge(calendar::router())
        .merge(gantt::router())
        .merge(optimization::router())
        .merge(reports::router())
        .merge(settings::router())
}
