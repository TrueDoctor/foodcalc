use bigdecimal::BigDecimal;
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    entities::unit::*,
    error::{Error, Result},
};

#[derive(Clone)]
pub struct UnitOps {
    pool: Arc<PgPool>,
}

impl UnitOps {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, unit: Unit) -> Result<Unit> {
        let row = sqlx::query_as!(
            Unit,
            r#"
            INSERT INTO units (name)
            VALUES ($1)
            RETURNING unit_id as "id", name
            "#,
            unit.name,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get(&self, id: i32) -> Result<Unit> {
        let row = sqlx::query_as!(
            Unit,
            r#"
            SELECT unit_id as "id", name
            FROM units 
            WHERE unit_id = $1
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update(&self, unit: Unit) -> Result<Unit> {
        let row = sqlx::query_as!(
            Unit,
            r#"
            UPDATE units
            SET name = $1
            WHERE unit_id = $2
            RETURNING unit_id as "id", name
            "#,
            unit.name,
            unit.id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete(&self, id: i32) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Delete from base_conversions
        sqlx::query!(
            r#"DELETE FROM base_conversions WHERE from_unit = $1 OR to_unit = $1"#,
            id
        )
        .execute(&mut *tx)
        .await?;

        // Delete from weights
        sqlx::query!(r#"DELETE FROM weights WHERE unit_id = $1"#, id)
            .execute(&mut *tx)
            .await?;

        // Delete from ingredient_sources
        sqlx::query!(r#"DELETE FROM ingredient_sources WHERE unit_id = $1"#, id)
            .execute(&mut *tx)
            .await?;

        // Finally delete the unit
        sqlx::query!(r#"DELETE FROM units WHERE unit_id = $1"#, id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<Unit>> {
        let rows = sqlx::query_as!(
            Unit,
            r#"
            SELECT unit_id as "id", name
            FROM units
            ORDER BY name
            "#
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }

    // Unit conversion operations
    pub async fn add_conversion(&self, conversion: UnitConversion) -> Result<UnitConversion> {
        let row = sqlx::query_as!(
            UnitConversion,
            r#"
            INSERT INTO base_conversions (from_unit, to_unit, from_amount, to_amount)
            VALUES ($1, $2, $3, $4)
            RETURNING from_unit, to_unit, from_amount, to_amount
            "#,
            conversion.from_unit,
            conversion.to_unit,
            conversion.from_amount,
            conversion.to_amount,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn update_conversion(&self, conversion: UnitConversion) -> Result<UnitConversion> {
        let row = sqlx::query_as!(
            UnitConversion,
            r#"
            UPDATE base_conversions
            SET from_amount = $3, to_amount = $4
            WHERE from_unit = $1 AND to_unit = $2
            RETURNING from_unit, to_unit, from_amount, to_amount
            "#,
            conversion.from_unit,
            conversion.to_unit,
            conversion.from_amount,
            conversion.to_amount,
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete_conversion(&self, from_unit: i32, to_unit: i32) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM base_conversions 
            WHERE from_unit = $1 AND to_unit = $2
            "#,
            from_unit,
            to_unit
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_conversion(&self, from_unit: i32, to_unit: i32) -> Result<Option<BigDecimal>> {
        let row = sqlx::query!(
            r#"
            SELECT to_amount / from_amount as "conversion_factor!"
            FROM conversions 
            WHERE from_unit = $1 AND to_unit = $2
            "#,
            from_unit,
            to_unit
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row.map(|r| r.conversion_factor))
    }

    pub async fn get_all_conversions(&self) -> Result<Vec<ConversionRow>> {
        let rows = sqlx::query_as!(
            ConversionRow,
            r#"
            SELECT from_unit as "from_unit!", to_unit as "to_unit!", from_amount as "from_amount!", to_amount as "to_amount!" 
            FROM conversions
            "#
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn convert_unit(
        &self,
        amount: BigDecimal,
        from_unit: i32,
        to_unit: i32,
    ) -> Result<BigDecimal> {
        let conversion_factor = self.get_conversion(from_unit, to_unit).await?;

        match conversion_factor {
            Some(factor) => Ok(amount * factor),
            None => Err(Error::Validation {
                message: format!(
                    "No conversion path found from unit {} to unit {}",
                    from_unit, to_unit
                ),
            }),
        }
    }

    pub async fn refresh_conversions(&self) -> Result<()> {
        sqlx::query!("REFRESH MATERIALIZED VIEW conversions")
            .execute(&*self.pool)
            .await?;
        Ok(())
    }
}
