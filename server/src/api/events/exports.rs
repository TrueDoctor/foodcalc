use axum::{
    extract::{Path, State},
    http::header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    response::{IntoResponse, Response},
};

use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::event::ShoppingTour;
use foodlib_new::Error;

use crate::api::map_api_error;
use crate::MyAppState;

/// `GET /api/events/{id}/exports/recipes` — combined PDF with every meal
/// recipe and food-prep sheet, ordered chronologically.
pub async fn export_recipes(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(event_id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(event_id).await {
        return map_api_error(err).into_response();
    }
    let event = match state.events().get(event_id).await {
        Ok(event) => event,
        Err(err) => return map_api_error(err).into_response(),
    };
    let infos = match state.export().fetch_event_combined(event_id).await {
        Ok(infos) => infos,
        Err(err) => return map_api_error(err).into_response(),
    };
    #[cfg(not(feature = "typst"))]
    let _ = &infos;

    #[cfg(feature = "typst")]
    let result = foodlib_new::typst::export_combined(infos)
        .await
        .map_err(|err| Error::Misc(err.to_string()));
    #[cfg(not(feature = "typst"))]
    let result: Result<Vec<u8>, Error> =
        Err(Error::Misc("Server compiled without typst support".into()));

    pdf_response(&format!("{}_alle_rezepte", sanitize(&event.name)), result)
}

/// `GET /api/events/{id}/exports/meal/{meal_id}` — the meal's recipe PDF,
/// scaled to the meal's planned energy and portions.
pub async fn export_meal(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((_event_id, meal_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_meal(meal_id).await {
        return map_api_error(err).into_response();
    }
    let info = match state.export().fetch_meal_recipe(meal_id).await {
        Ok(info) => info,
        Err(err) => return map_api_error(err).into_response(),
    };
    let title = info.name.to_owned();

    #[cfg(feature = "typst")]
    let result = foodlib_new::typst::export_recipes(info)
        .await
        .map_err(|err| Error::Misc(err.to_string()));
    #[cfg(not(feature = "typst"))]
    let result: Result<Vec<u8>, Error> =
        Err(Error::Misc("Server compiled without typst support".into()));

    pdf_response(&title, result)
}

/// `GET /api/events/{id}/exports/allergens` — allergen list PDF for the event.
pub async fn export_allergens(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(event_id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(event_id).await {
        return map_api_error(err).into_response();
    }
    let info = match state.export().fetch_event_allergens(event_id).await {
        Ok(info) => info,
        Err(err) => return map_api_error(err).into_response(),
    };
    let filename = format!("{}_allergene", sanitize(&info.event_name));

    #[cfg(feature = "typst")]
    let result = foodlib_new::typst::export_event_allergens(info)
        .await
        .map_err(|err| Error::Misc(err.to_string()));
    #[cfg(not(feature = "typst"))]
    let result: Result<Vec<u8>, Error> =
        Err(Error::Misc("Server compiled without typst support".into()));

    pdf_response(&filename, result)
}

/// `GET /api/events/{id}/exports/labels` — tent-card dish labels PDF with the
/// allergen summary per dish.
pub async fn export_labels(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(event_id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(event_id).await {
        return map_api_error(err).into_response();
    }
    let info = match state.export().fetch_event_allergens(event_id).await {
        Ok(info) => info,
        Err(err) => return map_api_error(err).into_response(),
    };
    let filename = format!("{}_dish_labels_tent", sanitize(&info.event_name));

    #[cfg(feature = "typst")]
    let result = foodlib_new::typst::export_dish_labels(
        info,
        foodlib_new::typst::DishLabelLayout::Tent,
    )
    .await
    .map_err(|err| Error::Misc(err.to_string()));
    #[cfg(not(feature = "typst"))]
    let result: Result<Vec<u8>, Error> =
        Err(Error::Misc("Server compiled without typst support".into()));

    pdf_response(&filename, result)
}

/// `GET /api/events/{id}/exports/tour/{tour_id}` — the tour's shopping list
/// as a plain-text checklist (categories, amounts, per-recipe usage).
pub async fn export_tour(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((_event_id, tour_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_shopping_tour(tour_id).await {
        return map_api_error(err).into_response();
    }
    let tour = match state.events().get_shopping_tour(tour_id).await {
        Ok(tour) => tour,
        Err(err) => return map_api_error(err).into_response(),
    };
    let items = match state
        .events()
        .tour_shopping_list_with_usage(tour_id)
        .await
    {
        Ok(items) => items,
        Err(err) => return map_api_error(err).into_response(),
    };

    let text = format_tour_text(&tour, &items);
    let filename = format!(
        "{}_{}_einkaufsliste",
        sanitize(tour.store_name.as_deref().unwrap_or("tour")),
        tour.tour_date.date()
    );

    (
        [
            (
                CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}.txt\""),
            ),
            (CONTENT_TYPE, "text/plain; charset=utf-8".to_string()),
        ],
        text,
    )
        .into_response()
}

fn pdf_response(filename: &str, result: Result<Vec<u8>, Error>) -> Response {
    match result {
        Ok(bytes) => (
            [
                (
                    CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{filename}.pdf\""),
                ),
                (CONTENT_TYPE, "application/pdf".to_string()),
            ],
            bytes,
        )
            .into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// Renders the tour checklist: category sections, totals per ingredient and
/// the recipe usage breakdown that makes each total up.
fn format_tour_text(
    tour: &ShoppingTour,
    items: &[foodlib_new::entities::event::ShoppingListItemWithUsage],
) -> String {
    let date_format = time::format_description::parse("[day].[month].[year]").ok();
    let date_text = match &date_format {
        Some(format) => tour
            .tour_date
            .format(format)
            .unwrap_or_else(|_| tour.tour_date.to_string()),
        None => tour.tour_date.to_string(),
    };

    let mut output = format!(
        "Einkaufsliste – {} – {}\n\n",
        tour.store_name.as_deref().unwrap_or("Tour"),
        date_text
    );

    let mut current_category = String::new();
    for item in items {
        let category = item.category.clone().unwrap_or_default();
        if category != current_category {
            if !current_category.is_empty() {
                output.push('\n');
            }
            let label = if category.is_empty() {
                "Sonstiges"
            } else {
                &category
            };
            output.push_str(&format!("=== {label} ===\n"));
            current_category = category;
        }
        output.push_str(&format!(
            "[ ] {} – {} kg\n",
            item.ingredient_name, item.weight
        ));
        for (recipe, weight) in &item.recipe_usage {
            output.push_str(&format!("      {recipe}: {weight} kg\n"));
        }
    }
    output
}

/// Filesystem-safe filename fragment.
fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}
