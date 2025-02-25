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
    Redirect(String, String),
    Hashing,
    Authentication(Box<axum_login::Error<AuthBackend>>),
    #[cfg(feature = "axum")]
    Status(axum::http::StatusCode),
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
impl From<time::error::Parse> for Error {
    fn from(value: time::error::Parse) -> Self {
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
            Error::Unauthorized(message) => write!(f, "Unauthorized: {}", message),
            Error::Forbidden(message) => write!(f, "Forbidden: {}", message),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let status = match self {
            Error::Redirect(e, ref r) => return redirect_html_error(&e, r).into_response(),
            Error::NotFound { .. } => StatusCode::NOT_FOUND,
            Error::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            Error::Forbidden { .. } => StatusCode::FORBIDDEN,
            _ => StatusCode::UNPROCESSABLE_ENTITY,
        };
        focus_html_error(&format!("{self}"), status).into_response()
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
pub fn html_error(reason: &str) -> maud::Markup {
    maud::html! {
        dialog open="true"
        id="error"
        class="fixed top-4 left-1/2 -translate-x-1/2 z-50 bg-gray-900 dark:bg-gray-800 rounded-lg shadow-xl p-6 min-w-[24rem] w-auto" {
            div class="flex flex-col items-center gap-4" {
                // Error icon
                div class="w-lg h-lg flex-shrink-0" {
                    svg xmlns="http://www.w3.org/2000/svg"
                        class="text-red-500"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        style="width: 12rem; height: 12rem;" {
                        path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" {}
                    }
                }

                // Error heading
                h1 class="text-2xl font-bold text-red-500" {
                    "Error"
                }

                // Error message
                p class="text-center text-gray-300 dark:text-gray-300 text-lg" {
                    (reason)
                }

                // Dismiss button - using the same style as other buttons
                button
                    class="btn btn-cancel px-6 py-2 text-lg"
                    hx-get="/ingredients"
                    hx-swap="delete"
                    hx-target="closest dialog" {
                    "Dismiss"
                }
            }
        }
    }
}

#[cfg(feature = "axum")]
pub fn redirect_html_error(reason: &str, redirect: &str) -> maud::Markup {
    maud::html! {
        dialog open="true"
        id="error"
        class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 max-w-md mx-auto" {
            div class="flex flex-col items-center gap-4" {
                // Error icon
                svg xmlns="http://www.w3.org/2000/svg"
                    class="w-12 h-12 text-red-500"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor" {
                    path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" {}
                }

                // Error heading
                h1 class="text-xl font-bold text-red-500" {
                    "Error"
                }

                // Error message
                p class="text-center text-gray-700 dark:text-gray-300" {
                    (reason)
                }

                // Back button
                button class="mt-2 bg-red-500 hover:bg-red-600 text-white font-bold py-2 px-4 rounded transition-colors duration-200"
                    hx-get=(redirect)
                    hx-target="#content" {
                    "Back"
                }
            }
        }
    }
}

#[cfg(feature = "axum")]
pub fn focus_html_error(
    reason: &str,
    status: axum::http::StatusCode,
) -> impl axum::response::IntoResponse {
    (
        status,
        [("HX-Reswap", "afterend show:top"), ("HX-Retarget", "this")],
        html_error(reason),
    )
}
