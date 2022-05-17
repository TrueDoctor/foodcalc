use sqlx::{postgres::PgPool, types::BigDecimal};
use std::sync::Arc;

#[derive(Clone)]
pub struct Ingredient {
    pub ingredient_id: i32,
    pub name: String,
    pub energy: BigDecimal,
    pub comment: Option<String>,
}

#[derive(Clone)]
pub struct Store {
    pub store_id: i32,
    pub name: String,
}

#[derive(Clone)]
pub struct Unit {
    pub unit_id: i32,
    pub name: String,
}

#[derive(Clone)]
pub struct IngredientSorce {
    pub ingredient_id: i32,
    pub store_id: i32,
    pub package_size: i32,
    pub unit_id: i32,
    pub name: String,
    pub price: u64,
    pub url: Option<String>,
    pub comment: Option<String>,
}

pub struct FoodBase {
    pg_pool: Arc<PgPool>,
}

impl FoodBase {
    pub fn new(pg_pool: PgPool) -> Self {
        Self {
            pg_pool: Arc::new(pg_pool),
        }
    }

    pub async fn add_ingredient(
        &self,
        name: String,
        energy: BigDecimal,
        comment: Option<String>,
    ) -> eyre::Result<i32> {
        let ingredient = sqlx::query!(
            r#"
                INSERT INTO ingredients ( name, energy, comment )
                VALUES ( $1, $2, $3 )
                RETURNING ingredient_id
            "#,
            name,
            energy,
            comment
        )
        .fetch_one(&*self.pg_pool)
        .await?;

        Ok(ingredient.ingredient_id)
    }

    pub async fn get_ingredients(&self) -> eyre::Result<Vec<Ingredient>> {
        let records = sqlx::query_as!(
            Ingredient,
            r#" SELECT * FROM ingredients ORDER BY ingredient_id "#,
        )
        .fetch_all(&*self.pg_pool)
        .await?;

        Ok(records)
    }
}
