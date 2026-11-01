use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use foodlib_new::{
    auth_context::AuthCtx,
    entities::event::{IngredientWithoutTour, ShoppingListItem, ShoppingTour},
};

use crate::api::{bad_request, map_api_error, respond, respond_created};
use crate::MyAppState;

/// Shopping list of an event, grouped by tour.
#[derive(serde::Serialize)]
pub struct ShoppingListDto {
    pub tours: Vec<TourWithItems>,
    /// Ingredients that no tour covers yet (their store has no tour in the
    /// relevant date window).
    pub without_tour: Vec<IngredientWithoutTour>,
}

#[derive(serde::Serialize)]
pub struct TourWithItems {
    #[serde(flatten)]
    pub tour: ShoppingTour,
    pub items: Vec<ShoppingListItem>,
    /// Sum of the item prices; `None` when any item has no priced source.
    pub total_price: Option<BigDecimal>,
}

#[derive(Debug, Deserialize)]
pub struct TourInput {
    /// RFC 3339 timestamp of the tour date.
    pub tour_date: String,
    pub store_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct SourceOverrideInput {
    pub source_id: i32,
}

/// `GET /api/events/{id}/shopping-list` — tours with their items plus
/// ingredients that are not covered by any tour yet.
pub async fn shopping_list(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    let tours = match state.events().get_shopping_tours(id).await {
        Ok(tours) => tours,
        Err(err) => return map_api_error(err).into_response(),
    };
    let items = match state.events().get_event_shopping_list(id).await {
        Ok(items) => items,
        Err(err) => return map_api_error(err).into_response(),
    };
    let without_tour = match state.events().get_ingredients_without_tour(id).await {
        Ok(items) => items,
        Err(err) => return map_api_error(err).into_response(),
    };

    let mut grouped: std::collections::HashMap<i32, Vec<ShoppingListItem>> =
        std::collections::HashMap::new();
    for item in items {
        if let Some(tour_id) = item.tour_id {
            grouped.entry(tour_id).or_default().push(item);
        }
    }

    let tours = tours
        .into_iter()
        .map(|tour| {
            let items = grouped.remove(&tour.id).unwrap_or_default();
            let total_price = items.iter().try_fold(
                BigDecimal::from(0),
                |sum, item| match &item.price {
                    Some(price) => Some(sum + price),
                    // Missing price on any item → no reliable tour total.
                    None => None,
                },
            );
            TourWithItems {
                tour,
                items,
                total_price,
            }
        })
        .collect();

    respond(Ok(ShoppingListDto {
        tours,
        without_tour,
    }))
}

/// `GET /api/events/{id}/tours` — all shopping tours of the event.
pub async fn list_tours(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    respond(state.events().get_shopping_tours(id).await)
}

/// `POST /api/events/{id}/tours` — schedule a shopping tour for a store.
/// Ingredients sourced from that store whose `buy_by` falls into this tour's
/// date window are assigned automatically by the `shopping_tour_ingredients`
/// view — no per-item assignment needed.
pub async fn add_tour(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<TourInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    let tour_date = match parse_time(&payload.tour_date) {
        Ok(time) => time,
        Err(response) => return response,
    };
    let tour = ShoppingTour {
        id: 0,
        event_id: id,
        tour_date,
        store_id: payload.store_id,
        store_name: None,
    };
    respond_created(state.events().add_shopping_tour(tour).await)
}

/// `PUT /api/events/{id}/tours/{tour_id}` — reschedule or move a tour.
pub async fn update_tour(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((event_id, tour_id)): Path<(i32, i32)>,
    Json(payload): Json<TourInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_shopping_tour(tour_id).await {
        return map_api_error(err).into_response();
    }
    let current = match state.events().get_shopping_tour(tour_id).await {
        Ok(tour) => tour,
        Err(err) => return map_api_error(err).into_response(),
    };
    if current.event_id != event_id {
        return bad_request("tour does not belong to this event".into());
    }
    let tour_date = match parse_time(&payload.tour_date) {
        Ok(time) => time,
        Err(response) => return response,
    };

    let tour = ShoppingTour {
        id: tour_id,
        event_id: current.event_id,
        tour_date,
        store_id: payload.store_id,
        store_name: current.store_name,
    };
    respond(state.events().update_shopping_tour(tour).await)
}

/// `DELETE /api/events/{id}/tours/{tour_id}` — remove the tour. Its items
/// fall back to "without tour".
pub async fn delete_tour(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((_event_id, tour_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_shopping_tour(tour_id).await {
        return map_api_error(err).into_response();
    }
    match state.events().delete_shopping_tour(tour_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

/// `GET /api/events/{id}/source-overrides` — per-event reroutes of
/// ingredients to a different source/store.
pub async fn list_overrides(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    respond(state.events().get_source_overrides(id).await)
}

/// `POST /api/events/{id}/source-overrides` — reroute an ingredient source
/// for this event (idempotent upsert).
pub async fn add_override(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path(id): Path<i32>,
    Json(payload): Json<SourceOverrideInput>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(id).await {
        return map_api_error(err).into_response();
    }
    respond_created(
        state
            .events()
            .set_source_override(id, payload.source_id)
            .await,
    )
}

/// `DELETE /api/events/{id}/source-overrides/{source_id}` — remove a reroute.
pub async fn delete_override(
    State(state): State<MyAppState>,
    ctx: AuthCtx,
    Path((event_id, source_id)): Path<(i32, i32)>,
) -> Response {
    if let Err(err) = ctx.assert_can_edit_event(event_id).await {
        return map_api_error(err).into_response();
    }
    match state
        .events()
        .delete_source_override(event_id, source_id)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => map_api_error(err).into_response(),
    }
}

fn parse_time(value: &str) -> Result<OffsetDateTime, Response> {
    OffsetDateTime::parse(value, &Rfc3339)
        .map_err(|_| bad_request(format!("invalid RFC 3339 timestamp: {value}")))
}
