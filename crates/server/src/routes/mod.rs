use crate::state::AppState;
use axum::Router;

pub mod catalog;
pub mod projects;
pub mod resources;
pub mod tasks;
pub mod teams;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .merge(projects::router())
        .merge(catalog::router())
        .merge(tasks::router())
        .merge(resources::router())
        .merge(teams::router())
}
