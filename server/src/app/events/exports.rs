//! Export use-cases for events: permission-checked data fetching plus
//! rendering into PDF (Typst, behind the `typst` feature) or plain text.
//!
//! The transport receives an [`ExportedFile`] (filename + bytes + mime) and
//! only wraps it into the HTTP attachment response.

use bigdecimal::ToPrimitive;

use foodlib_new::entities::event::ShoppingTour;
use foodlib_new::ops::export::{EventAllergenInfo, EventOverviewInfo, EventOverviewMeal, EventOverviewPrep, EventOverviewTour};
use foodlib_new::policy::EventPerm;

#[cfg(feature = "typst")]
use pdf_export::renderer::DishLabelLayout;

use crate::app::{AppCtx, Error, MyAppState, Result};

/// A finished export artifact: filename (without extension), mime type and
/// the rendered bytes.
pub struct ExportedFile {
    pub filename: String,
    pub mime: &'static str,
    pub bytes: Vec<u8>,
}

/// Renders a batch of recipe infos into a PDF via the Typst engine.
// TODO: Use a single Typst engine for all recipes instead of creating a new one per request.
// (Without the `typst` feature the parameters are intentionally unused.)
#[allow(unused_variables)]
pub(crate) fn render_recipes(
    infos: Vec<foodlib_new::ops::export::RecipeInfo>,
) -> Result<Vec<u8>> {
    #[cfg(feature = "typst")]
    let result = {
        let renderer = pdf_export::DocumentRenderer::new();
        renderer
            .export_recipes(infos)
            .map_err(|e| Error::Misc(e.to_string()))
    };
    #[cfg(not(feature = "typst"))]
    let result = Err(Error::Misc("Server compiled without typst support".into()));

    result
}

// (Without the `typst` feature the parameters are intentionally unused.)
#[allow(unused_variables)]
fn render_allergen_list(info: EventAllergenInfo) -> Result<Vec<u8>> {
    #[cfg(feature = "typst")]
    // TODO: Use a single Typst engine for all recipes instead of creating a new one per request.
    let result = {
        let renderer = pdf_export::DocumentRenderer::new();
        renderer
            .export_event_allergens(info)
            .map_err(|e| Error::Misc(e.to_string()))
    };
    #[cfg(not(feature = "typst"))]
    let result: Result<Vec<u8>> = Err(Error::Misc("Server compiled without typst support".into()));

    result
}

// (Without the `typst` feature the parameters are intentionally unused.)
#[allow(unused_variables)]
fn render_export_dish_labels(info: EventAllergenInfo) -> Result<Vec<u8>> {
    #[cfg(feature = "typst")]
    // TODO: Use a single Typst engine for all recipes instead of creating a new one per request.
    let result = {
        let renderer = pdf_export::DocumentRenderer::new();
        renderer
            .export_dish_labels(info, DishLabelLayout::Tent)
            .map_err(|e| Error::Misc(e.to_string()))
    };
    #[cfg(not(feature = "typst"))]
    let result: Result<Vec<u8>> = Err(Error::Misc("Server compiled without typst support".into()));

    result
}

/// `GET …/exports/overview` — management summary PDF: frame data with budget
/// status, planned meals with planner numbers, shopping tours and food-prep
/// tasks. Requires `events:export`.
pub async fn export_summary_pdf(state: &MyAppState, ctx: &AppCtx, event_id: i32) -> Result<ExportedFile> {
    ctx.ctx.assert_event_action(event_id, EventPerm::Export).await?;
    let event = state.events().get(event_id).await?;
    let total_cost = state.events().get_total_cost(event_id).await?;
    let meals = state.meals().get_event_meals_planner(event_id).await?;
    let tours = state.events().get_shopping_tours(event_id).await?;
    let preps = state.events().list_food_preps(event_id).await?;

    // Built and kept for the future Typst exporter wiring (currently the
    // overview export returns "not implemented").
    #[allow(unused_variables)]
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

    // TODO: implement appropriate Typst export function for event overview
    let bytes: Result<Vec<u8>> =
        Err(Error::Misc("Typst export for event overview not implemented".into()));

    bytes.map(|bytes| ExportedFile {
        filename,
        mime: "application/pdf",
        bytes,
    })
}

/// `GET …/exports/recipes` — combined PDF with every meal recipe and
/// food-prep sheet, ordered chronologically. Requires `events:edit`.
pub async fn export_all_recipes_for_one_event(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: i32,
) -> Result<ExportedFile> {
    actor.ctx.assert_event_action(event_id, EventPerm::Edit).await?;
    let event = state.events().get(event_id).await?;
    let infos = state.export().fetch_event_combined(event_id).await?;
    let bytes = render_recipes(infos)?;
    Ok(ExportedFile {
        filename: format!("{}_alle_rezepte", sanitize(&event.name)),
        mime: "application/pdf",
        bytes,
    })
}

/// `GET …/exports/meal/{meal_id}` — the meal's recipe PDF, scaled to the
/// meal's planned energy and portions. Requires `events:export`.
pub async fn export_recipe_for_meal(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: i32,
    meal_id: i32,
) -> Result<ExportedFile> {
    actor.ctx.assert_event_action(event_id, EventPerm::Export).await?;
    let info = state.export().fetch_meal_recipe(meal_id).await?;
    let title = info.name.to_owned();
    let bytes = render_recipes(vec![info])?;
    Ok(ExportedFile {
        filename: title,
        mime: "application/pdf",
        bytes,
    })
}

/// `GET …/exports/allergens` — allergen list PDF for the event. Requires
/// `events:export`.
pub async fn export_allergen_list_for_event(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: i32,
) -> Result<ExportedFile> {
    actor.ctx.assert_event_action(event_id, EventPerm::Export).await?;
    let info = state.export().fetch_event_allergens(event_id).await?;
    let filename = format!("{}_allergene", sanitize(&info.event_name));

    let result = render_allergen_list(info);

    result.map(|bytes| ExportedFile {
        filename,
        mime: "application/pdf",
        bytes,
    })
}


/// `GET …/exports/labels` — tent-card dish labels PDF with the allergen
/// summary per dish. Requires `events:export`.
pub async fn export_dish_labels(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: i32,
) -> Result<ExportedFile> {
    actor.ctx.assert_event_action(event_id, EventPerm::Export).await?;
    let info = state.export().fetch_event_allergens(event_id).await?;
    let filename = format!("{}_dish_labels_tent", sanitize(&info.event_name));

    let result = render_export_dish_labels(info);
    result.map(|bytes| ExportedFile {
        filename,
        mime: "application/pdf",
        bytes,
    })
}


pub async fn export_shopping_tour_list(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: i32,
    tour_id: i32,
) -> Result<ExportedFile> {
    // Permission check
    actor.ctx.assert_event_action(event_id, EventPerm::Export).await?;

    let tour: ShoppingTour = state.events().get_shopping_tour(tour_id).await?;
    let items = state
        .events()
        .tour_shopping_list_with_usage(tour_id)
        .await?;

    let text = format_tour_text(&tour, &items);

    Ok(ExportedFile {
        filename: format!(
            "{}_{}_einkaufsliste",
            sanitize(tour.store_name.as_deref().unwrap_or("tour")),
            tour.tour_date.date()
        ),
        mime: "text/plain; charset=utf-8",
        bytes: text.into_bytes(),
    })
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
