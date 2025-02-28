use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use foodlib::Event;
use num::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use sqlx::types::{time::OffsetDateTime, BigDecimal};

use crate::ApiState;

fn event_to_body(event: Event) -> EventBody {
    EventBody {
        id: Some(event.event_id),
        name: event.event_name,
        comment: event.comment,
        budget: event.budget.as_ref().and_then(ToPrimitive::to_f64),
    }
}

pub fn router() -> Router<crate::ApiState> {
    println!("Loading Event Router");
    Router::new()
        .route("/", get(list_events))
        .route("/", put(add_event))
        .route("/{event_id}/", get(show_event))
        .route("/{event_id}/", delete(delete_event))
        .route("/{event_id}/", post(update_event))
        .route("/{event_id}/meals/", get(meal_list))
        .route("/{event_id}/meals/", put(meal_add))
        .route("/meals/{meal_id}", get(get_meal))
        .route("/meals/{meal_id}", delete(meal_delete))
        .route("/meals/{meal_id}", post(meal_update))
}

async fn list_events(State(state): State<ApiState>) -> impl IntoResponse {
    if let Ok(event_list) = state.food_base.get_events().await {
        (
            StatusCode::OK,
            Json(
                event_list
                    .into_iter()
                    .map(event_to_body)
                    .collect::<Vec<EventBody>>(),
            ),
        )
            .into_response()
    } else {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct EventBody {
    id: Option<i32>,
    name: String,
    comment: Option<String>,
    budget: Option<f64>,
}

// TODO Add Error Handling
async fn add_event(
    State(state): State<ApiState>,
    Json(body): Json<EventBody>,
) -> impl IntoResponse {
    let budget = body.budget.and_then(|budget| BigDecimal::from_f64(budget));

    let query = state.food_base.add_event(body.name, budget, body.comment);
    match query.await {
        Ok(event) => (StatusCode::CREATED, Json(event_to_body(event))).into_response(),
        Err(_) => StatusCode::CONFLICT.into_response(),
    }
}

// TODO: Also show Meals
async fn show_event(State(state): State<ApiState>, Path(event_id): Path<i32>) -> impl IntoResponse {
    if let Some(event) = state.food_base.get_event(event_id).await {
        (StatusCode::OK, Json(event_to_body(event))).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn delete_event(
    State(state): State<ApiState>,
    Path(event_id): Path<i32>,
) -> impl IntoResponse {
    match state.food_base.delete_event(event_id).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn update_event(
    State(state): State<ApiState>,
    Path(event_id): Path<i32>,
    Json(body): Json<EventBody>,
) -> impl IntoResponse {
    let budget = body.budget.and_then(FromPrimitive::from_f64);

    let event = Event {
        event_id: event_id.clone(),
        event_name: body.name.clone(),
        comment: body.comment.clone(),
        budget,
        owner_id: -1,
    };

    if let Ok(event) = state.food_base.update_event(&event).await {
        (StatusCode::OK, Json(event_to_body(event))).into_response()
    } else {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct MealBody {
    event_id: i32,
    recipe: i32,
    place: i32,
    start: i64,
    end: i64,
    energy: BigDecimal,
    servings: i32,
    comment: Option<String>,
}

async fn meal_add(
    State(state): State<ApiState>,
    Path(event_id): Path<i32>,
    Json(body): Json<MealBody>,
) -> impl IntoResponse {
    let _ = state
        .food_base
        .add_meal(
            event_id,
            body.recipe,
            body.place,
            OffsetDateTime::from_unix_timestamp(body.start).unwrap(),
            OffsetDateTime::from_unix_timestamp(body.end).unwrap(),
            body.energy,
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

async fn meal_delete(State(state): State<ApiState>, Path(meal_id): Path<i32>) -> impl IntoResponse {
    if let Ok(_) = state.food_base.remove_meal(meal_id).await {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

//TODO: Add Better Way of updating Meal
async fn meal_update(
    State(state): State<ApiState>,
    Path(meal_id): Path<i32>,
    Json(body): Json<MealBody>,
) -> impl IntoResponse {
    let remove_query = state.food_base.remove_meal(meal_id);
    println!("Remove");

    if let Err(_) = remove_query.await {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    println!("Add");
    let add_query = state.food_base.add_meal(
        body.event_id,
        body.recipe,
        body.place,
        OffsetDateTime::from_unix_timestamp(body.start).unwrap(),
        OffsetDateTime::from_unix_timestamp(body.end).unwrap(),
        BigDecimal::from(body.energy),
        body.servings,
        body.comment,
    );

    if let Err(_) = add_query.await {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    return StatusCode::OK;
}

#[derive(Serialize, Deserialize)]
struct IdAndName {
    id: i32,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct APIDate {
    start: i64,
    end: i64,
}
#[derive(Serialize, Deserialize)]
struct MealReturn {
    event_id: i32,
    meal_id: i32,
    recipe: IdAndName,
    place: IdAndName,
    date: APIDate,
    weight: BigDecimal,
    energy: BigDecimal,
    price: BigDecimal,
    servings: i32,
    comment: Option<String>,
}
// TODO Add Error Handling
async fn meal_list(State(state): State<ApiState>, Path(event_id): Path<i32>) -> impl IntoResponse {
    let query = state.food_base.get_event_meals(event_id).await;
    match query {
        Ok(meals) => {
            let meals: Vec<MealReturn> = meals
                .into_iter()
                .map(|meal| {
                    return MealReturn {
                        meal_id: meal.meal_id,
                        event_id,
                        recipe: IdAndName {
                            id: meal.recipe_id,
                            name: meal.name,
                        },
                        place: IdAndName {
                            id: meal.place_id,
                            name: meal.place,
                        },
                        date: APIDate {
                            start: meal.start_time.unix_timestamp(),
                            end: meal.start_time.unix_timestamp(),
                        },
                        weight: meal.weight,
                        energy: meal.energy,
                        price: meal.price,
                        servings: meal.servings,
                        comment: meal.comment,
                    };
                })
                .collect();
            (StatusCode::OK, Json(meals)).into_response()
        }
        Err(error) => {
            println!("{:?}", error);
            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}

// TODO Add Error Handling
async fn get_meal(State(state): State<ApiState>, Path(meal_id): Path<i32>) -> impl IntoResponse {
    let query = state.food_base.get_event_meal(meal_id).await;
    match query {
        Ok(meal) => {
            let meal = MealReturn {
                meal_id: meal.meal_id,
                event_id: meal.event_id,
                recipe: IdAndName {
                    id: meal.recipe_id,
                    name: meal.name,
                },
                place: IdAndName {
                    id: meal.place_id,
                    name: meal.place,
                },
                date: APIDate {
                    start: meal.start_time.unix_timestamp(),
                    end: meal.start_time.unix_timestamp(),
                },
                weight: meal.weight,
                energy: meal.energy,
                price: meal.price,
                servings: meal.servings,
                comment: meal.comment,
            };
            (StatusCode::OK, Json(meal)).into_response()
        }
        Err(error) => {
            println!("{:?}", error);
            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}
