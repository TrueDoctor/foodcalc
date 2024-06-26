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
        .route("/:event_id/", get(show_event))
        .route("/:event_id/", delete(delete_event))
        .route("/:event_id/", post(update_event))
        .route("/:event_id/meals/", get(meal_list))
        .route("/:event_id/meals/", put(meal_add))
        .route("/meals/:meal_id", get(get_meal))
        .route("/meals/:meal_id", delete(meal_delete))
        .route("/meals/:meal_id", post(meal_update))
}

async fn list(State(state): State<ApiState>) -> impl IntoResponse {
    if let Ok(event_list) = state.food_base.get_events().await {
        (StatusCode::OK, Json(event_list)).into_response()
    } else {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

// TODO: Also show Meals
async fn show_event(State(state): State<ApiState>, Path(event_id): Path<i32>) -> impl IntoResponse {
    if let Some(event) = state.food_base.get_event(event_id).await {
        (StatusCode::OK, Json(event)).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn delete_event(
    State(_state): State<ApiState>,
    Path(_event_id): Path<i32>,
) -> impl IntoResponse {
    todo!()
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct UpdateEventBody {
    event_name: String,
    comment: Option<String>,
    budget: Option<i64>,
}

async fn update_event(
    State(state): State<ApiState>,
    Path(event_id): Path<i32>,
    Json(body): Json<UpdateEventBody>,
) -> impl IntoResponse {
    let budget = body.budget.map(|budget| PgMoney(budget));

    dbg!(budget);
    let event = Event {
        event_id: event_id.clone(),
        event_name: body.event_name.clone(),
        comment: body.comment.clone(),
        budget: budget,
    };

    if let Ok(result) = state.food_base.update_event(&event).await {
        (StatusCode::OK, Json(result)).into_response()
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
    energy: u32,
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

async fn meal_delete(State(state): State<ApiState>, Path(meal_id): Path<i32>) -> impl IntoResponse {
    if let Ok(_) = state.food_base.remove_meal(meal_id).await {
        StatusCode::OK
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
        NaiveDateTime::from_timestamp_millis(body.start).unwrap(),
        NaiveDateTime::from_timestamp_millis(body.end).unwrap(),
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
async fn meal_list(State(state): State<ApiState>, Path(event_id): Path<i32>) -> impl IntoResponse {
    let query = state.food_base.get_event_meals(event_id).await;
    match query {
        Ok(meals) => {
            let meals: Vec<MealReturn> = meals
                .into_iter()
                .map(|meal| {
                    return MealReturn {
                        meal_id: meal.meal_id,
                        event_id: event_id,
                        recipe: IdAndName {
                            id: meal.recipe_id,
                            name: meal.name,
                        },
                        place: IdAndName {
                            id: meal.place_id,
                            name: meal.place,
                        },
                        date: APIDate {
                            start: meal.start_time.timestamp_millis(),
                            end: meal.start_time.timestamp_millis(),
                        },
                        weight: meal.weight,
                        energy: meal.energy,
                        price: meal.price.to_bigdecimal(2),
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
                    start: meal.start_time.timestamp_millis(),
                    end: meal.start_time.timestamp_millis(),
                },
                weight: meal.weight,
                energy: meal.energy,
                price: meal.price.to_bigdecimal(2),
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
