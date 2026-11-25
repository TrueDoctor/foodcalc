use std::fmt;

#[derive(Debug)]
pub enum Error {
    Database(sqlx::Error),
    NotFound {
        entity: &'static str,
        id: String,
    },
    Validation {
        message: String,
    },
    Conflict {
        message: String,
    },
    UserNotFound {
        name: String,
    },
    Misc(String),
    Redirect(String, String),
    Hashing,
    Unauthorized(String),
    Forbidden(String),
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

impl From<eyre::Report> for Error {
    fn from(value: eyre::Report) -> Self {
        Error::Misc(value.to_string())
    }
}

impl From<time::error::Parse> for Error {
    fn from(value: time::error::Parse) -> Self {
        Error::Misc(value.to_string())
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
            Error::Misc(e) => write!(f, "{}", e),
            Error::Redirect(e, _) => write!(f, "{}", e),
            Error::Unauthorized(message) => write!(f, "Unauthorized: {}", message),
            Error::Forbidden(message) => write!(f, "Forbidden: {}", message),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;