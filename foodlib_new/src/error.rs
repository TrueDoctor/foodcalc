use std::fmt;

use crate::auth::AuthBackend;

#[derive(Debug)]
pub enum Error {
    Database(sqlx::Error),
    NotFound { entity: &'static str, id: String },
    Validation { message: String },
    Conflict { message: String },
    UserNotFound { name: String },
    Hashing,
    Authentication(Box<axum_login::Error<AuthBackend>>),
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::Database(err)
    }
}
impl From<bcrypt::BcryptError> for Error {
    fn from(_: bcrypt::BcryptError) -> Self {
        Error::Hashing
    }
}
impl From<axum_login::Error<AuthBackend>> for Error {
    fn from(value: axum_login::Error<AuthBackend>) -> Self {
        Error::Authentication(Box::new(value))
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Database(e) => write!(f, "Database error: {}", e),
            Error::NotFound { entity, id } => write!(f, "{} with id {} not found", entity, id),
            Error::Validation { message } => write!(f, "Validation error: {}", message),
            Error::Conflict { message } => write!(f, "Conflict error: {}", message),
            Error::UserNotFound { name } => write!(f, "Did not find user: {}", name),
            Error::Hashing => write!(f, "Failed to compute hash"),
            Error::Authentication(e) => write!(f, "Authentication Error: {}", e),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
