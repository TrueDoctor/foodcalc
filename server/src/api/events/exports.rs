//! Export endpoints for events — thin HTTP attachment wrappers over
//! `crate::app::events::exports`. Permission checks, data fetching and
//! rendering (Typst / text) live in the app layer; handlers only map the
//! returned [`ExportedFile`] to headers + body.

use axum::{
    extract::{Path, State},
    http::header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    response::{IntoResponse, Response},
};

use crate::api::map_api_error;
use crate::app::{events::exports, AppCtx};
use crate::MyAppState;

/// Maps an app-layer `ExportedFile` to a download response.
fn file_response(
    file: Result<exports::ExportedFile, foodlib_new::Error>,
    extension: &str,
) -> Response {
    match file {
        Ok(file) => (
            [
                (
                    CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}.{}\"", file.filename, extension),
                ),
                (CONTENT_TYPE, file.mime.to_string()),
            ],
            file.bytes,
        )
            .into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `GET /api/events/{id}/exports/overview` — management summary PDF: frame
/// data with budget status, planned meals with planner numbers, shopping
/// tours and food-prep tasks.
pub async fn export_overview(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path(event_id): Path<i32>,
) -> Response {
    file_response(exports::export_summary_pdf(&state, &ctx, event_id).await, "pdf")
}

/// `GET /api/events/{id}/exports/recipes` — combined PDF with every meal
/// recipe and food-prep sheet, ordered chronologically.
pub async fn export_recipes(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path(event_id): Path<i32>,
) -> Response {
    file_response(exports::export_all_recipes_for_one_event(&state, &ctx, event_id).await, "pdf")
}

/// `GET /api/events/{id}/exports/meal/{meal_id}` — the meal's recipe PDF,
/// scaled to the meal's planned energy and portions.
pub async fn export_meal(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path((event_id, meal_id)): Path<(i32, i32)>,
) -> Response {
    file_response(exports::export_recipe_for_meal(&state, &ctx, event_id, meal_id).await, "pdf")
}

/// `GET /api/events/{id}/exports/allergens` — allergen list PDF for the event.
pub async fn export_allergens(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path(event_id): Path<i32>,
) -> Response {
    file_response(exports::export_allergen_list_for_event(&state, &ctx, event_id).await, "pdf")
}

/// `GET /api/events/{id}/exports/labels` — tent-card dish labels PDF with the
/// allergen summary per dish.
pub async fn export_labels(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path(event_id): Path<i32>,
) -> Response {
    file_response(exports::export_dish_labels(&state, &ctx, event_id).await, "pdf")
}

/// `GET /api/events/{id}/exports/tour/{tour_id}` — the tour's shopping list
/// as a plain-text checklist (categories, amounts, per-recipe usage).
pub async fn export_tour(
    State(state): State<MyAppState>,
    ctx: AppCtx,
    Path((event_id, tour_id)): Path<(i32, i32)>,
) -> Response {
    file_response(exports::export_shopping_tour_list(&state, &ctx, event_id, tour_id).await, "txt")
}
