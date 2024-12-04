use std::fmt;

#[derive(Debug)]
pub enum Error {
    Database(sqlx::Error),
    NotFound { entity: &'static str, id: String },
    Validation { message: String },
    Conflict { message: String },
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::Database(err)
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
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
