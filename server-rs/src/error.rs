use std::fmt::{Display, Formatter};
use actix_web::ResponseError;

#[derive(Debug)]
pub struct Error(anyhow::Error);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl ResponseError for Error {}

impl<T> From<T> for Error
where
    T: Into<anyhow::Error>,
{
    fn from(t: T) -> Self {
        Error(t.into())
    }
}
