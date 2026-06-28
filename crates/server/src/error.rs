use app::error::AppError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

pub struct HttpError(pub AppError);

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let status = match self.0.code {
            "VALIDATION" => StatusCode::BAD_REQUEST,
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "DOMAIN" => StatusCode::UNPROCESSABLE_ENTITY,
            "DB" => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(self.0)).into_response()
    }
}

impl<E> From<E> for HttpError
where
    E: Into<AppError>,
{
    fn from(e: E) -> Self {
        Self(e.into())
    }
}
