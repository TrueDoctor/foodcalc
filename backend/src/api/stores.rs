use std::sync::Arc;

use axum::{
    extract::Path,
    http::StatusCode,
    routing::{post, MethodRouter},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::types::PgMoney, error::BoxDynError};

use crate::db::{FoodBase, IngredientSorce};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct SerializableIngredientSource {
    ingredient_id: i32,
    store_id: i32,
    package_size: bigdecimal::BigDecimal,
    price: bigdecimal::BigDecimal,
    url: Option<String>,
    unit_id: i32,
    comment: Option<String>,
}

impl From<IngredientSorce> for SerializableIngredientSource {
    fn from(source: IngredientSorce) -> Self {
        Self {
            ingredient_id: source.ingredient_id,
            store_id: source.store_id,
            package_size: source.package_size,
            price: source.price.to_bigdecimal(2),
            unit_id: source.unit_id,
            url: source.url,
            comment: source.comment,
        }
    }
}

impl TryInto<IngredientSorce> for SerializableIngredientSource {
    fn try_into(self) -> Result<IngredientSorce, Self::Error> {
        Ok(IngredientSorce {
            ingredient_id: self.ingredient_id,
            store_id: self.store_id,
            package_size: self.package_size,
            price: PgMoney::from_bigdecimal(self.price, 2)?,
            unit_id: self.unit_id,
            url: self.url,
            comment: self.comment,
        })
    }

    type Error = BoxDynError;
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct CreateIngredientSource {
    ingredient_id: i32,
    store_id: i32,
    price: bigdecimal::BigDecimal,
    weight: bigdecimal::BigDecimal,
    url: Option<String>,
    unit_id: i32,
}

pub fn add_ingredient_source(db: Arc<FoodBase>) -> MethodRouter {
    let db = db.clone();

    post(|ingredient_source: Json<CreateIngredientSource>| async move {
        let price = match PgMoney::from_bigdecimal(ingredient_source.price.clone(), 2) {
            Ok(price) => price,
            Err(_) => return StatusCode::BAD_REQUEST,
        };
        let result = db
            .add_ingredient_source(
                ingredient_source.ingredient_id,
                ingredient_source.store_id,
                ingredient_source.weight.clone(),
                price,
                ingredient_source.url.clone(),
                ingredient_source.unit_id,
            )
            .await;
        match result {
            Ok(_) => StatusCode::CREATED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })
}

pub fn fetch_metro_prices(db: Arc<FoodBase>) -> MethodRouter {
    let db = db.clone();

    post(|Path(id): Path<i32>| async move {
        let result = db.fetch_metro_prices(Some(id)).await;
        match result {
            Ok(_) => StatusCode::OK,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })
}

pub fn fetch_all_metro_prices(db: Arc<FoodBase>) -> MethodRouter {
    let db = db.clone();

    post(|| async move {
        let result = db.fetch_metro_prices(None).await;
        match result {
            Ok(_) => StatusCode::OK,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })
}

pub fn get_metro_ingredient_sources(db: Arc<FoodBase>) -> MethodRouter {
    let db = db.clone();

    post(|Path(id): Path<i32>| async move {
        let result = db.get_metro_ingredient_sources(Some(id)).await;
        match result {
            Ok(sources) => {
                let sources = sources.into_iter().map(|source| SerializableIngredientSource::from(source)).collect();
                (StatusCode::OK, Json(sources))
            } ,
            _ => (StatusCode::INTERNAL_SERVER_ERROR,Json(vec![])),
        }
    })
}

pub fn get_all_metro_ingredient_sources(db: Arc<FoodBase>) -> MethodRouter {
    let db = db.clone();

    post(|| async move {
        let result = db.get_metro_ingredient_sources(None).await;
        match result {
            Ok(sources) => {
                let sources = sources.into_iter().map(|source| SerializableIngredientSource::from(source)).collect();
                (StatusCode::OK, Json(sources))
            } ,
            _ => (StatusCode::INTERNAL_SERVER_ERROR,Json(vec![])),
        }
    })
}

pub fn update_ingredient_source_price(db: Arc<FoodBase>) -> MethodRouter {
    let db = db.clone();

    #[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    struct IngrdientPrice {
        ingredient_id: i32,
        price: bigdecimal::BigDecimal,
        url: Option<String>,
        weight: bigdecimal::BigDecimal,
    }

    post(|Json(ingredient): Json<IngrdientPrice>| async move {
        let price = match PgMoney::from_bigdecimal(ingredient.price.clone(), 2) {
            Ok(price) => price,
            Err(_) => return StatusCode::BAD_REQUEST,
        };
        let result = db.update_ingredient_source_price(ingredient.ingredient_id,ingredient.url,price,ingredient.weight).await;
        match result {
            Ok(_) => StatusCode::OK,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })
}
