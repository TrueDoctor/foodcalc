use axum::{
    extract::{Path, State},
    http::header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    response::{IntoResponse, Response},
};

use foodlib_new::auth_context::AuthCtx;
use foodlib_new::entities::event::ShoppingTour;
use foodlib_new::ops::export::{
    EventOverviewInfo, EventOverviewMeal, EventOverviewPrep, EventOverviewTour,
};
use foodlib_new::Error;
use bigdecimal::ToPrimitive;
use pdf_export::renderer::DishLabelLayout;
use crate::api::map_api_error;
use crate::MyAppState;

/// `GET /api/events/{id}/exports/overview` — management summary PDF: frame
/// data with budget status, planned meals with planner numbers, shopping
/// tours and food-prep tasks.
pub async fn export_overview(
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
    let total_cost = match state.events().get_total_cost(event_id).await {
        Ok(cost) => cost,
        Err(err) => return map_api_error(err).into_response(),
    };
    let meals = match state.meals().get_event_meals_planner(event_id).await {
        Ok(meals) => meals,
        Err(err) => return map_api_error(err).into_response(),
    };
    let tours = match state.events().get_shopping_tours(event_id).await {
        Ok(tours) => tours,
        Err(err) => return map_api_error(err).into_response(),
    };
    let preps = match state.events().list_food_preps(event_id).await {
        Ok(preps) => preps,
        Err(err) => return map_api_error(err).into_response(),
    };

    let info = EventOverviewInfo {
        event_name: event.name.clone(),
        comment: event.comment.clone(),
        budget: event.budget.clone(),
        total_cost,
        meals: meals
            .iter()
            .map(|meal| EventOverviewMeal {
                recipe_name: meal.name.clone(),
                place: meal.place.clone(),
                start_time: meal.start_time,
                servings: meal.servings,
                weight_kg: (meal.weight.to_f64().unwrap_or(0.0) > 0.0)
                    .then(|| meal.weight.clone()),
                energy_per_portion_kj: meal.energy_per_serving.clone(),
                price: meal.price.clone(),
            })
            .collect(),
        tours: {
            let mut rows = Vec::with_capacity(tours.len());
            for tour in &tours {
                // Item counts come from the tour's shopping list; a failed
                // count degrades to 0 instead of failing the whole export.
                let item_count = state
                    .events()
                    .get_shopping_list(tour.id)
                    .await
                    .map(|items| items.len() as i64)
                    .unwrap_or(0);
                rows.push(EventOverviewTour {
                    tour_date: tour.tour_date,
                    store_name: tour.store_name.clone(),
                    item_count,
                });
            }
            rows
        },
        preps: preps
            .iter()
            .map(|prep| EventOverviewPrep {
                prep_date: prep.prep_date,
                recipe_name: prep.recipe_name.clone(),
                use_from: prep.use_from,
                use_until: prep.use_until,
            })
            .collect(),
    };
    let filename = format!("{}_uebersicht", sanitize(&event.name));

    #[cfg(feature = "typst")]
    // TODO: implement appropriate Typst export function for event overview
    let renderer = pdf_export::DocumentRenderer::new();
    // let result = renderer.export_event_allergens(info);
    // let result = foodlib_new::typst::export_event_overview(info)
    //     .await
    //     .map_err(|err| Error::Misc(err.to_string()));
    let result = Err(Error::Misc("Typst export for event overview not implemented".into()));
    todo!("Implement Typst export for event overview");
    #[cfg(not(feature = "typst"))]
    let result: Result<Vec<u8>, Error> =
        Err(Error::Misc("Server compiled without typst support".into()));

    pdf_response(&filename, result)
}

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
    // TODO: Use a single Typst engine for all recipes instead of creating a new one per request.
    let renderer = pdf_export::DocumentRenderer::new();
    let result = renderer.export_recipes(infos)
        .map_err(|e| foodlib_new::Error::Misc(e.to_string()));
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
    // TODO: Use a single Typst engine for all recipes instead of creating a new one per request.
    let renderer = pdf_export::DocumentRenderer::new();
    let result = renderer.export_recipes(vec![info])
        .map_err(|e| foodlib_new::Error::Misc(e.to_string()));
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
    // TODO: Use a single Typst engine for all recipes instead of creating a new one per request.
    let renderer = pdf_export::DocumentRenderer::new();
    let result = renderer.export_event_allergens(info)
        .map_err(|e| foodlib_new::Error::Misc(e.to_string()));
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
    // TODO: Use a single Typst engine for all recipes instead of creating a new one per request.
    let renderer = pdf_export::DocumentRenderer::new();
    let result = renderer.export_dish_labels(info, DishLabelLayout::Tent)
        .map_err(|e| foodlib_new::Error::Misc(e.to_string()));
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
            item.ingredient_name,
            item.weight
                .as_ref()
                .map(|weight| weight.to_string())
                .unwrap_or_else(|| "?".to_string())
        ));
        for (recipe, weight) in &item.recipe_usage {
            output.push_str(&format!(
                "      {recipe}: {} kg\n",
                weight
                    .as_ref()
                    .map(|part| part.to_string())
                    .unwrap_or_else(|| "?".to_string())
            ));
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
