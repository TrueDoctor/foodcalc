use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use foodlib::Event;
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::types::PgMoney,
    types::{chrono::NaiveDateTime, BigDecimal},
};

use crate::ApiState;

pub fn router() -> Router<crate::ApiState> {
    println!("Loading Event Router");
    Router::new()
        .route("/", get(list))
        //.route("/", post(update_event))
        .route("/:event_id/", get(show_event))
        .route("/:event_id/", delete(delete_event))
        .route("/:event_id/", post(update_event))
        .route("/:event_id/meals/", get(meal_list))
        .route("/:event_id/meals/", put(meal_add))
        .route("/:event_id/meals/", delete(meal_delete))
        .route("/:event_id/meals/", post(meal_update))
}

async fn list(State(state): State<ApiState>) -> impl IntoResponse {
    if let Ok(event_list) = state.food_base.get_events().await {
        return (StatusCode::OK, Json(event_list));
    } else {
        todo!()
    }
}

// TODO: Also show Meals
async fn show_event(State(state): State<ApiState>, Path(event_id): Path<i32>) -> impl IntoResponse {
    if let Some(event) = state.food_base.get_event(event_id).await {
        return (StatusCode::OK, Json(event));
    } else {
        todo!()
    }
}

async fn delete_event(
    State(state): State<ApiState>,
    Path(event_id): Path<i32>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct UpdateEventBody {
    name: String,
    comment: Option<String>,
    budget: Option<f64>,
}

async fn update_event(
    State(state): State<ApiState>,
    Path(event_id): Path<i32>,
    Json(body): Json<UpdateEventBody>,
) -> impl IntoResponse {
    let event = Event {
        event_id: event_id.clone(),
        event_name: body.name.clone(),
        comment: body.comment.clone(),
        budget: body.budget.map(|b| PgMoney(b as i64)),
    };

    if let Ok(result) = state.food_base.update_event(&event).await {
        return (StatusCode::OK, Json(result));
    } else {
        todo!();
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct MealBody {
    recipe: i32,
    place: i32,
    start: i64,
    end: i64,
    energy: u32,
    servings: i32,
    comment: Option<String>,
}

async fn meal_add(
    State(state): State<ApiState>,
    Path(event_id): Path<i32>,
    Json(body): Json<MealBody>,
) -> impl IntoResponse {
    state
        .food_base
        .add_meal(
            event_id,
            body.recipe,
            body.place,
            NaiveDateTime::from_timestamp_millis(body.start).unwrap(),
            NaiveDateTime::from_timestamp_millis(body.end).unwrap(),
            BigDecimal::from(body.energy),
            body.servings,
            body.comment,
        )
        .await;
    StatusCode::OK
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct MealSelectorBody {
    recipe: i32,
    place: i32,
    start: i64,
}

async fn meal_delete(
    State(state): State<ApiState>,
    Path(event_id): Path<i32>,
    Json(body): Json<MealBody>,
) -> impl IntoResponse {
    state.food_base.remove_meal_by_reference(
        event_id,
        body.recipe,
        body.place,
        NaiveDateTime::from_timestamp_millis(body.start).unwrap(),
    );
    StatusCode::OK
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct UpdateMealBody {
    selector: MealSelectorBody,
    data: MealBody,
}

//TODO: Add Better Way of updating Meal
async fn meal_update(
    State(state): State<ApiState>,
    Path(event_id): Path<i32>,
    Json(body): Json<UpdateMealBody>,
) -> impl IntoResponse {
    let selector = body.selector;
    let new_data = body.data;

    let remove_query = state.food_base.remove_meal_by_reference(
        event_id,
        selector.recipe,
        selector.place,
        NaiveDateTime::from_timestamp_millis(selector.start).unwrap(),
    );
    println!("Remove");

    if let Err(_) = remove_query.await {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    println!("Add");
    let add_query = state.food_base.add_meal(
        event_id,
        new_data.recipe,
        new_data.place,
        NaiveDateTime::from_timestamp_millis(new_data.start).unwrap(),
        NaiveDateTime::from_timestamp_millis(new_data.end).unwrap(),
        BigDecimal::from(new_data.energy),
        new_data.servings,
        new_data.comment,
    );

    if let Err(_) = add_query.await {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    return StatusCode::OK;
}

async fn meal_list(State(state): State<ApiState>, Path(event_id): Path<i32>) -> impl IntoResponse {
    let query = state.food_base.get_event_meals(event_id).await;
    if let Ok(meals) = query {
        return (StatusCode::OK, Json(meals));
    } else {
        todo!()
    }
}
