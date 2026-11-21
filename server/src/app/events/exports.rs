//! Export use-cases for events: permission-checked data fetching plus
//! delegation to [`crate::app::exports`] renderers.

use std::collections::HashMap;

use bigdecimal::ToPrimitive;
use serde::Deserialize;

pub(crate) use crate::app::exports::{
    csv, render_pdf, sanitize_filename, text, DishLabelLayout, ExportFormat, ExportedFile,
    MetroExportContext, TourExportData,
};
use crate::app::{AppCtx, Error, MyAppState, Result};
use foodlib_new::entities::event::ShoppingTour;
use foodlib_new::entities::types::ids::{EventId, IngredientId, MealId, TourId};
use foodlib_new::ops::export::{
    EventOverviewInfo, EventOverviewMeal, EventOverviewPrep, EventOverviewTour,
};
use foodlib_new::policy::{EventPerm, Permission};
use foodlib_new::types::amount::Amount;

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct TourExportQuery {
    #[serde(default)]
    pub format: ExportFormat,
    /// Defaults to `true` for `Csv` (Metro order always deducts stock) or can be
    /// explicitly toggled via `?deduct_inventory=true|false` for `Txt`.
    #[serde(default)]
    pub deduct_inventory: Option<bool>,
}

/// `GET …/exports/overview` — management summary PDF.
pub async fn export_summary_pdf(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<ExportedFile> {
    actor
        .ctx
        .assert_event_actions(event_id, [EventPerm::ExportMealPlan, EventPerm::ViewCalculation])
        .await?;

    let event = state.events().get(event_id).await?;
    let total_cost = state.events().get_total_cost(event_id).await?;
    let meals = state.meals().get_event_meals_planner(event_id).await?;
    let tours = state.events().get_shopping_tours(event_id).await?;
    let preps = state.events().list_food_preps(event_id).await?;

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

    let filename = format!("{}_uebersicht", sanitize_filename(&event.name));
    render_pdf(state, filename, move |pdf| pdf.export_event_overview(info)).await
}

/// `GET …/exports/recipes` — combined PDF with every meal recipe and food-prep sheet.
pub async fn export_all_recipes_for_one_event(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<ExportedFile> {
    // Permission
    actor.ctx.assert_event_action(event_id, EventPerm::ExportMealPlan).await?;

    let event = state.events().get(event_id).await?;
    let infos = state.export().fetch_event_combined(event_id).await?;
    let filename = format!("{}_alle_rezepte", sanitize_filename(&event.name));

    render_pdf(state, filename, move |pdf| pdf.export_recipes(infos)).await
}

/// `GET …/exports/meal/{meal_id}` — single meal's scaled recipe PDF.
pub async fn export_recipe_for_meal(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    meal_id: MealId,
) -> Result<ExportedFile> {
    actor
        .ctx
        .assert_event_action(event_id, EventPerm::ExportMealPlan)
        .await?;

    let belongs_to_event = state
        .meals()
        .get_event_meals_planner(event_id)
        .await?
        .iter()
        .any(|m| m.meal_id == meal_id);
    if !belongs_to_event {
        return Err(Error::NotFound {
            entity: "Meal",
            id: meal_id.to_string(),
        });
    }

    let info = state.export().fetch_meal_recipe(meal_id).await?;
    let filename = sanitize_filename(&info.name);

    render_pdf(state, filename, move |pdf| pdf.export_recipes(vec![info])).await
}

/// Export the allergen list for an event.
pub async fn export_allergen_list_for_event(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<ExportedFile> {
    actor
        .ctx
        .assert_event_action(event_id, EventPerm::ExportMealPlan)
        .await?;

    let info = state.export().fetch_event_allergens(event_id).await?;
    let filename = format!("{}_allergene", sanitize_filename(&info.event_name));

    render_pdf(state, filename, move |pdf| pdf.export_event_allergens(info)).await
}

/// Export the dish labels for an event.
pub async fn export_dish_labels(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
) -> Result<ExportedFile> {
    actor
        .ctx
        .assert_event_action(event_id, EventPerm::ExportMealPlan)
        .await?;

    let info = state.export().fetch_event_allergens(event_id).await?;
    let filename = format!("{}_dish_labels_tent", sanitize_filename(&info.event_name));

    render_pdf(state, filename, move |pdf| {
        pdf.export_dish_labels(info, DishLabelLayout::Tent)
    })
        .await
}


pub async fn export_shopping_tour_list(
    state: &MyAppState,
    actor: &AppCtx,
    event_id: EventId,
    tour_id: TourId,
    query: TourExportQuery,
) -> Result<ExportedFile> {
    actor
        .ctx
        .assert_event_action(event_id, EventPerm::ExportMealPlan)
        .await?;

    let tour: ShoppingTour = state.events().get_shopping_tour(tour_id).await?;
    if tour.event_id != event_id {
        return Err(Error::NotFound {
            entity: "ShoppingTour",
            id: tour_id.to_string(),
        });
    }

    let event = state.events().get(event_id).await?;
    let items = state
        .events()
        .tour_shopping_list_with_usage(tour_id)
        .await?;

    let deduct = query
        .deduct_inventory
        .unwrap_or(matches!(query.format, ExportFormat::Csv));

    let (inventory_stock, show_prices) = if deduct {
        let show_prices = actor.ctx.has_event_permission(
            event_id,
            event.group_id,
            Permission::Event(EventPerm::ViewCalculation),
        );
        let stock = load_tour_inventory_stock(state, event_id).await?;
        (Some(stock), show_prices)
    } else {
        (None, false)
    };

    let metro = if matches!(query.format, ExportFormat::Csv) {
        let sources = state.ingredients().list_sources(None).await?;

        let article_numbers = state
            .events()
            .resolve_metro_article_numbers(tour_id)
            .await?;
        Some(MetroExportContext {
            sources,
            article_numbers,
        })
    } else {
        None
    };

    let base_name = format!(
        "{}_{}",
        sanitize_filename(tour.store_name.as_deref().unwrap_or("tour")),
        tour.tour_date.date()
    );

    let data = TourExportData {
        tour,
        items,
        inventory_stock,
        show_prices,
        metro,
    };

    match query.format {
        ExportFormat::Txt => Ok(ExportedFile {
            filename: format!("{base_name}_einkaufsliste"),
            mime: "text/plain; charset=utf-8",
            bytes: text::render_shopping_tour(&data),
        }),
        ExportFormat::Csv => Ok(ExportedFile {
            filename: format!("{base_name}_metro_list"),
            mime: "text/csv; charset=utf-8",
            bytes: csv::render_shopping_tour(&data)?,
        }),
        ExportFormat::Pdf => {
            render_pdf(state, format!("{base_name}_einkaufsliste"), move |pdf| {
                pdf.export_shopping_tour(&data)
            })
                .await
        }
    }
}

/// Sums available stock per ingredient across the event's linked inventories.
async fn load_tour_inventory_stock(
    state: &MyAppState,
    event_id: EventId,
) -> Result<HashMap<IngredientId, Amount>> {
    let inventories = state.events().list_event_inventories(event_id).await?;
    let mut totals: HashMap<IngredientId, Amount> = HashMap::new();

    for inv in inventories {
        let stock_items = state.inventories().get_items(inv.id).await?;
        for item in stock_items {
            totals
                .entry(item.ingredient_id)
                .and_modify(|acc| *acc = acc.clone() + item.amount.clone())
                .or_insert(item.amount);
        }
    }
    Ok(totals)
}