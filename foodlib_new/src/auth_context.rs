use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::Extension;
use sqlx::PgPool;
use std::sync::Arc;

use crate::auth::AuthSession;
use crate::entities::user::User;
use crate::error::{Error, Result};
use crate::FoodLib;

/// Authenticated user together with the IDs of every group they belong to.
///
/// Use this extractor in handlers that need to make group-membership decisions.
/// It performs a single DB lookup per request — handlers should pass a
/// reference to the existing `AuthCtx` rather than re-extracting.
#[derive(Debug, Clone)]
pub struct AuthCtx {
    pub user: User,
    pub group_ids: Vec<i32>,
    pool: Arc<PgPool>,
}

impl AuthCtx {
    /// True if the user is an admin or belongs to the given group.
    pub fn can_access_group(&self, group_id: i32) -> bool {
        self.user.is_admin || self.group_ids.contains(&group_id)
    }

    /// Returns Forbidden unless the user has access to the given group.
    pub fn assert_group(&self, group_id: i32) -> Result<()> {
        if self.can_access_group(group_id) {
            Ok(())
        } else {
            Err(Error::Forbidden(
                "You don't have permission to modify this resource".into(),
            ))
        }
    }

    pub async fn assert_can_edit_recipe(&self, recipe_id: i32) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"SELECT group_id FROM recipes WHERE recipe_id = $1"#,
            recipe_id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "Recipe",
            id: recipe_id.to_string(),
        })?;
        self.assert_group(group_id)
    }

    pub async fn assert_can_edit_event(&self, event_id: i32) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"SELECT group_id FROM events WHERE event_id = $1"#,
            event_id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "Event",
            id: event_id.to_string(),
        })?;
        self.assert_group(group_id)
    }

    pub async fn assert_can_edit_ingredient(&self, ingredient_id: i32) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"SELECT group_id FROM ingredients WHERE ingredient_id = $1"#,
            ingredient_id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "Ingredient",
            id: ingredient_id.to_string(),
        })?;
        self.assert_group(group_id)
    }

    pub async fn assert_can_edit_inventory(&self, inventory_id: i32) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"SELECT group_id FROM inventories WHERE inventory_id = $1"#,
            inventory_id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "Inventory",
            id: inventory_id.to_string(),
        })?;
        self.assert_group(group_id)
    }

    /// Meals, food preps, shopping tours and source overrides all belong to an
    /// event; permission follows the parent event.
    pub async fn assert_can_edit_meal(&self, meal_id: i32) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"
            SELECT e.group_id
            FROM event_meals m
            JOIN events e ON e.event_id = m.event_id
            WHERE m.meal_id = $1
            "#,
            meal_id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "Meal",
            id: meal_id.to_string(),
        })?;
        self.assert_group(group_id)
    }

    pub async fn assert_can_edit_food_prep(&self, prep_id: i32) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"
            SELECT e.group_id
            FROM food_prep fp
            JOIN events e ON e.event_id = fp.event_id
            WHERE fp.prep_id = $1
            "#,
            prep_id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "FoodPrep",
            id: prep_id.to_string(),
        })?;
        self.assert_group(group_id)
    }

    pub async fn assert_can_edit_shopping_tour(&self, tour_id: i32) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"
            SELECT e.group_id
            FROM shopping_tours t
            JOIN events e ON e.event_id = t.event_id
            WHERE t.tour_id = $1
            "#,
            tour_id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or(Error::NotFound {
            entity: "ShoppingTour",
            id: tour_id.to_string(),
        })?;
        self.assert_group(group_id)
    }
}

impl<S> FromRequestParts<S> for AuthCtx
where
    S: Send + Sync,
    AuthSession: FromRequestParts<S>,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let auth = AuthSession::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::Unauthorized("no auth headers".to_string()))?;
        let user = auth
            .user
            .ok_or(Error::Unauthorized("Authentication required".to_string()))?;

        let Extension(foodlib): Extension<FoodLib> = Extension::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::Misc("FoodLib extension missing".into()))?;

        let groups = foodlib.users().get_user_groups(user.id).await?;
        let group_ids = groups.into_iter().map(|g| g.id).collect();

        Ok(AuthCtx {
            user,
            group_ids,
            pool: foodlib.pool_arc(),
        })
    }
}
