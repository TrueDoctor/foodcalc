use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use foodlib_new::Error;

pub struct WebError(pub Error);

impl From<Error> for WebError {
    fn from(err: Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            Error::Redirect(ref e, ref r) => return redirect_html_error(e, r).into_response(),
            Error::NotFound { .. } => StatusCode::NOT_FOUND,
            Error::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Error::Forbidden(_) => StatusCode::FORBIDDEN,
            _ => StatusCode::UNPROCESSABLE_ENTITY,
        };
        focus_html_error(&format!("{}", self.0), status).into_response()
    }
}

pub fn html_error(reason: &str) -> maud::Markup {
    maud::html! {
        dialog open="true"
        id="error"
        class="fixed top-4 left-1/2 -translate-x-1/2 z-50 bg-gray-900 dark:bg-gray-800 rounded-lg shadow-xl p-6 min-w-[24rem] w-auto" {
            div class="flex flex-col items-center gap-4" {
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
                h1 class="text-2xl font-bold text-red-500" { "Error" }
                p class="text-center text-gray-300 dark:text-gray-300 text-lg" {
                    @for line in reason.lines() {
                        (line)
                        br;
                    }
                }
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

pub fn redirect_html_error(reason: &str, redirect: &str) -> maud::Markup {
    maud::html! {
        dialog open="true"
        id="error"
        class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6 max-w-md mx-auto" {
            div class="flex flex-col items-center gap-4" {
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
                h1 class="text-xl font-bold text-red-500" { "Error" }
                p class="text-center text-gray-700 dark:text-gray-300" { (reason) }
                button class="mt-2 bg-red-500 hover:bg-red-600 text-white font-bold py-2 px-4 rounded transition-colors duration-200"
                    hx-get=(redirect)
                    hx-target="#content" {
                    "Back"
                }
            }
        }
    }
}

pub fn focus_html_error(reason: &str, status: StatusCode) -> impl IntoResponse {
    (
        status,
        [("HX-Reswap", "afterend show:top"), ("HX-Retarget", "this")],
        html_error(reason),
    )
}