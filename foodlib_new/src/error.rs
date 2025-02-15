use std::fmt;

use crate::auth::AuthBackend;

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
    Redirect(String, &'static str),
    Hashing,
    Authentication(Box<axum_login::Error<AuthBackend>>),
    #[cfg(feature = "axum")]
    Status(axum::http::StatusCode),
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
impl From<eyre::Report> for Error {
    fn from(value: eyre::Report) -> Self {
        Error::Misc(value.to_string())
    }
}
#[cfg(feature = "axum")]
impl From<axum::http::StatusCode> for Error {
    fn from(value: axum::http::StatusCode) -> Self {
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
            Error::Authentication(e) => write!(f, "Authentication Error: {}", e),
            Error::Misc(e) => write!(f, "{}", e),
            Error::Redirect(e, _) => write!(f, "{}", e),
            #[cfg(feature = "axum")]
            Error::Status(e) => write!(f, "{}", e),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let (redirect, status) = match self {
            Error::Redirect(_, r) => (r, StatusCode::INTERNAL_SERVER_ERROR),
            Error::NotFound { .. } => ("", StatusCode::NOT_FOUND),
            _ => ("", StatusCode::INTERNAL_SERVER_ERROR),
        };
        focus_html_error(&format!("{self}"), redirect, status).into_response()
    }
}
#[cfg(feature = "axum")]
impl From<Error> for axum::response::Response {
    fn from(val: Error) -> Self {
        use axum::response::IntoResponse;
        val.into_response()
    }
}

#[cfg(feature = "axum")]
pub fn html_error(reason: &str, redirect: &str) -> maud::Markup {
    maud::html! {
        dialog open="true"
        id="error"
        class="flex flex-col items-center justify-center text-red-500" {
            div {
                h1 { "Error" }
                p { (reason) }
                button class="btn btn-primary" hx-get=(redirect) hx-target="#content"  { "Back" }
            }
        }

    }
}
#[cfg(feature = "axum")]
pub fn focus_html_error(
    reason: &str,
    redirect: &str,
    status: axum::http::StatusCode,
) -> impl axum::response::IntoResponse {
    (
        status,
        [("HX-Swap", "beforeend show:top")],
        html_error(reason, redirect),
    )
}
