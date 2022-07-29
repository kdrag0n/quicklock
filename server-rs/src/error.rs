use std::fmt::{Display, Formatter};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug)]
pub struct Error(anyhow::Error);

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
    #[error("not found")]
    NotFound,
    #[error("bad request")]
    BadRequest,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match self.0.downcast_ref() {
            Some(HttpError::NotFound) => (StatusCode::NOT_FOUND, "not found"),
            Some(HttpError::BadRequest) => (StatusCode::BAD_REQUEST, "bad request"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error"),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

impl<T> From<T> for Error
where
    T: Into<anyhow::Error>,
{
    fn from(t: T) -> Self {
        Error(t.into())
    }
}

pub type AppResult<T> = Result<T, Error>;
