use std::fmt::{Display, Formatter};
use actix_web::http::StatusCode;
use actix_web::{Responder, ResponseError};

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

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self.0.downcast_ref() {
            Some(HttpError::NotFound) => StatusCode::NOT_FOUND,
            Some(HttpError::BadRequest) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
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
