use axum::{
    body::Body, extract::Request, http::Response, middleware::Next, response::IntoResponse,
};
use maud::{html, DOCTYPE};

use crate::frontend::home;
use crate::frontend::CSS_HASH;

pub async fn htmx_middleware(req: Request, next: Next) -> Response<Body> {
    let is_htmx = req
        .headers()
        .get("HX-Request")
        .map_or(false, |v| v == "true");

    let response = next.run(req).await;

    if is_htmx {
        response
    } else {
        // Only wrap HTML responses
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !content_type.contains("text/html") {
            return response;
        }

        let status = response.status();
        let headers = response.headers().clone();

        // Convert response body to string
        let body = if let Ok(bytes) = axum::body::to_bytes(response.into_body(), 1 << 24).await {
            String::from_utf8(bytes.to_vec()).unwrap_or_default()
        } else {
            String::new()
        };

        // Wrap in full page layout
        let wrapped = html! {
            (DOCTYPE)
            html {
                head {
                    title { "Foodcalc" }
                    link rel="stylesheet" href=(format!("/static/{}-style.css", CSS_HASH.with(|x| *x)));
                    script src="https://unpkg.com/htmx.org@2.0.4/dist/htmx.min.js" {}
                    script src="https://unpkg.com/htmx.org@2.0.4/dist/ext/debug.js" {}
                    meta name="htmx-config" content=r#"{
                    "scrollBehavior":"smooth",
                    "responseHandling": [
                        {"code":"204", "swap": false},
                        {"code":"[23]..", "swap": true},
                        {"code":"422", "swap": true},
                        {"code":"[45]..", "swap": false, "error":true},
                        {"code":"...", "swap": true}
                    ]
                    }"#;
                    meta name="viewport" content="width=800, initial-scale=1";
                }
                    body class="
                    bg-light-bg-light text-gray-800
                    dark:bg-dark-bg-dark dark:text-gray-100" {
                    div hx-push-url="true" {
                        (home::navbar())
                        div class="flex flex-col items-center justify-center mb-16" {
                            div id="content" class="w-3/4 flex flex-col items-center justify-center" {
                                (maud::PreEscaped(body))
                            }
                        }
                    }
                }
            }
        };

        // Build new response preserving status and headers
        let mut builder = Response::builder().status(status);

        // Copy over original headers
        if let Some(headers_mut) = builder.headers_mut() {
            for (name, value) in headers.iter() {
                headers_mut.insert(name, value.clone());
            }

            // Set content type to HTML for wrapped response
            headers_mut.insert("content-type", "text/html; charset=utf-8".parse().unwrap());
        }

        builder
            .body(wrapped.into_response().into_body())
            .unwrap_or_else(|_| Response::new(Body::empty()))
    }
}
