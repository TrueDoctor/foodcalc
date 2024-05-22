use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use foodlib::Event;
use maud::{html, Markup};
use serde::Deserialize;
use sqlx::postgres::types::PgMoney;

use crate::MyAppState;

pub(crate) fn event_detail_router() -> axum::Router<MyAppState> {
    axum::Router::new()
        .route("/:event_id", axum::routing::get(event_form))
        .route("/:event_id", axum::routing::post(update_event))
}

async fn event_form(state: State<MyAppState>, event_id: Path<i32>) -> Markup {
    html! {
        form class="flex flex-col items-center justify-center" action=(format!("/{}", event_id.0)) {
        }
    }
}

#[derive(Deserialize)]
struct EventForm {
    name: String,
    comment: String,
    budget: Option<f64>,
}

async fn update_event(
    state: State<MyAppState>,
    event_id: Path<i32>,
    event_data: Form<EventForm>,
) -> impl IntoResponse {
    let budget = event_data
        .budget
        .map(|budget| PgMoney((budget * 100.) as i64));

    dbg!(budget);
    let event = Event {
        event_id: event_id.clone(),
        event_name: event_data.name.clone(),
        comment: (!event_data.comment.is_empty()).then(|| event_data.comment.clone()),
        budget,
    };

    if let Ok(result) = state.db_connection.update_event(&event).await {
        (StatusCode::OK, event_form(state, event_id).await).into_response()
    } else {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
