use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::Extension;
use sqlx::PgPool;
use std::sync::Arc;

use crate::auth::AuthSession;
use crate::entities::user::User;
use crate::error::{Error, Result};
use crate::policy::{
    EventPerm, GlobalPermission, IngredientPerm, InventoryPerm, Permission, RecipePerm, UserPolicy,
};
use crate::FoodLib;

/// Authenticated user together with their resolved RBAC policy.
///
/// Use this extractor in handlers that need to make group-membership decisions.
/// It performs a single DB lookup per request — handlers should pass a
/// reference to the existing `AuthCtx` rather than re-extracting.
#[derive(Debug, Clone)]
pub struct AuthCtx {
    pub user: User,
    pub group_ids: Vec<i32>,
    pub policy: UserPolicy,
    pool: Arc<PgPool>,
}

impl AuthCtx {
    pub fn can_global(&self, perm: GlobalPermission) -> bool {
        self.policy.has_global_permission(perm)
    }

    pub fn assert_global(&self, perm: GlobalPermission) -> Result<()> {
        if self.can_global(perm) {
            Ok(())
        } else {
            Err(Error::Forbidden(format!(
                "Missing global permission: {}",
                perm
            )))
        }
    }

    /// True if the user has any role in the group (baseline read membership).
    pub fn can_access_group(&self, group_id: i32) -> bool {
        self.group_ids.contains(&group_id)
    }

    pub fn has_group_permission(&self, group_id: i32, perm: Permission) -> bool {
        self.policy.has_group_permission(group_id, perm)
    }

    pub fn has_event_permission(&self, event_id: i32, group_id: i32, perm: Permission) -> bool {
        self.policy.has_event_permission(event_id, group_id, perm)
    }

    /// Returns Forbidden unless the user belongs to the given group.
    pub fn assert_group(&self, group_id: i32) -> Result<()> {
        if self.can_access_group(group_id) {
            Ok(())
        } else {
            Err(Error::Forbidden(
                "You don't have permission to access this group".into(),
            ))
        }
    }

    /// Returns Forbidden unless the user holds `perm` in `group_id`.
    pub fn assert_group_perm(&self, group_id: i32, perm: Permission) -> Result<()> {
        if self.has_group_permission(group_id, perm) {
            Ok(())
        } else {
            Err(Error::Forbidden(format!(
                "Missing group permission: {}",
                perm
            )))
        }
    }

    /// Returns Forbidden unless the user holds `perm` via `group_id` OR via a
    /// direct per-event role on `event_id`.
    pub fn assert_event_perm(
        &self,
        event_id: i32,
        group_id: i32,
        perm: Permission,
    ) -> Result<()> {
        if self.has_event_permission(event_id, group_id, perm) {
            Ok(())
        } else {
            Err(Error::Forbidden(format!(
                "Missing event permission: {}",
                perm
            )))
        }
    }

    // ------------------------------------------------------------------------
    // Resource-Lookup Assertions (Resolves entity ID -> scope -> permission)
    // ------------------------------------------------------------------------

    pub async fn assert_recipe_perm(&self, recipe_id: i32, perm: RecipePerm) -> Result<()> {
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

        self.assert_group_perm(group_id, Permission::Recipe(perm))
    }

    pub async fn assert_ingredient_perm(
        &self,
        ingredient_id: i32,
        perm: IngredientPerm,
    ) -> Result<()> {
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

        self.assert_group_perm(group_id, Permission::Ingredient(perm))
    }

    pub async fn assert_inventory_perm(
        &self,
        inventory_id: i32,
        perm: InventoryPerm,
    ) -> Result<()> {
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

        self.assert_group_perm(group_id, Permission::Inventory(perm))
    }

    pub async fn assert_event_action(&self, event_id: i32, perm: EventPerm) -> Result<()> {
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

        self.assert_event_perm(event_id, group_id, Permission::Event(perm))
    }

    /// Meals belong to an event; checks both group permissions and per-event overrides.
    pub async fn assert_meal_action(&self, meal_id: i32, perm: EventPerm) -> Result<()> {
        let row = sqlx::query!(
            r#"
            SELECT e.event_id, e.group_id
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

        self.assert_event_perm(row.event_id, row.group_id, Permission::Event(perm))
    }

    /// Food preps belong to an event; checks both group permissions and per-event overrides.
    pub async fn assert_food_prep_action(&self, prep_id: i32, perm: EventPerm) -> Result<()> {
        let row = sqlx::query!(
            r#"
            SELECT e.event_id, e.group_id
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

        self.assert_event_perm(row.event_id, row.group_id, Permission::Event(perm))
    }

    /// Shopping tours belong to an event AND enforce the completed-tour lock.
    pub async fn assert_shopping_tour_action(
        &self,
        tour_id: i32,
        perm: EventPerm,
    ) -> Result<()> {
        let row = sqlx::query!(
            r#"
            SELECT e.event_id, e.group_id, t.completed
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

        if row.completed && perm == EventPerm::ShoppingEditTour {
            return Err(Error::Forbidden(
                "Completed shopping tours cannot be modified".into(),
            ));
        }

        self.assert_event_perm(row.event_id, row.group_id, Permission::Event(perm))
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

        // Single repository call loads global_permissions, group_permissions, and event_overrides
        let policy = foodlib.users().get_user_policy(user.id).await?;
        let group_ids = policy.group_permissions.keys().copied().collect();

        Ok(AuthCtx {
            user,
            group_ids,
            policy,
            pool: foodlib.pool_arc(),
        })
    }
}

impl AuthCtx {
    /// Deprecated: Use assert_inventory_perm instead
    pub async fn assert_can_edit_inventory(&self, inventory_id: i32) -> Result<()> {
        self.assert_inventory_perm(inventory_id, InventoryPerm::Edit).await
    }
    /// Deprecated: Use assert_event_perm instead
    pub async fn assert_can_edit_event(&self, event_id: i32) -> Result<()> {
        self.assert_event_action(event_id, EventPerm::Edit).await
    }
    /// Deprecated: Use assert_event_perm instead
    pub async fn assert_can_edit_meal(&self, event_id: i32) -> Result<()> {
        self.assert_event_action(event_id, EventPerm::Edit).await
    }
    /// Deprecated: Use assert_event_perm instead
    pub async fn assert_can_edit_food_prep(&self, event_id: i32) -> Result<()> {
        self.assert_event_action(event_id, EventPerm::Edit).await
    }

    /// Deprecated: Use assert_event_perm instead
    pub async fn assert_can_edit_recipe(&self, recipe_id: i32) -> Result<()> {
        self.assert_recipe_perm(recipe_id, RecipePerm::Edit).await
    }

    /// Deprecated: Use assert_event_perm instead
    pub async fn assert_can_edit_shopping_tour(&self, recipe_id: i32) -> Result<()> {
        self.assert_shopping_tour_action(recipe_id, EventPerm::Edit).await
    }
}
