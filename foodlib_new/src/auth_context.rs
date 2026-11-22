use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::Extension;
use sqlx::PgPool;
use std::sync::Arc;

use crate::auth::AuthSession;
use crate::entities::types::ids::{
    EventId, FoodPrepId, GroupId, IngredientId, InventoryId, MealId, RecipeId, TourId,
};
use crate::entities::user::User;
use crate::error::{Error, Result};
use crate::policy::{
    EventPerm, GlobalPermission, IngredientPerm, InventoryPerm, Permission, RecipePerm, UserPolicy,
};
use crate::FoodLib;

/// Authenticated user together with their resolved RBAC policy.
///
/// Performs a single DB lookup per request — pass `&AuthCtx` downstream
/// rather than re-extracting.
#[derive(Debug, Clone)]
pub struct AuthCtx {
    pub user: User,
    pub policy: UserPolicy,
    pool: Arc<PgPool>,
}

impl AuthCtx {
    /// Load an `AuthCtx` for an already-known `User` (useful after login/registration
    /// or inside background/CLI contexts).
    pub async fn load(foodlib: &FoodLib, user: User) -> Result<Self> {
        let policy = foodlib.users().get_user_policy(user.id).await?;
        Ok(Self {
            user,
            policy,
            pool: foodlib.pool_arc(),
        })
    }

    // ------------------------------------------------------------------------
    // Global & Group/Event Scope Checks
    // ------------------------------------------------------------------------

    pub fn can_global(&self, perm: GlobalPermission) -> bool {
        // admins must hold the admin:* permissions through explicit
        // `user_global_permissions` grants - isAdmin does not have any relevance
        self.policy.has_global_permission(perm)
    }

    pub fn assert_global(&self, perm: GlobalPermission) -> Result<()> {
        if self.can_global(perm) {
            Ok(())
        } else {
            Err(Error::Forbidden(format!(
                "Missing global permission: {perm}"
            )))
        }
    }

    /// True if the user holds any membership in `group_id`.
    pub fn can_access_group(&self, group_id: GroupId) -> bool {
        self.policy.group_permissions.contains_key(&group_id)
    }

    pub fn has_group_permission(&self, group_id: GroupId, perm: Permission) -> bool {
        self.policy.has_group_permission(group_id, perm)
    }

    pub fn has_event_permission(&self, event_id: EventId, group_id: GroupId, perm: Permission) -> bool {
        self.policy.has_event_permission(event_id, group_id, perm)
    }

    pub fn assert_group(&self, group_id: GroupId) -> Result<()> {
        if self.can_access_group(group_id) {
            Ok(())
        } else {
            Err(Error::Forbidden(
                "You don't have permission to access this group".into(),
            ))
        }
    }

    pub fn assert_group_perm(&self, group_id: GroupId, perm: Permission) -> Result<()> {
        if self.has_group_permission(group_id, perm) {
            Ok(())
        } else {
            Err(Error::Forbidden(format!(
                "Missing group permission: {perm}"
            )))
        }
    }

    pub fn assert_event_perm(
        &self,
        event_id: EventId,
        group_id: GroupId,
        perm: Permission,
    ) -> Result<()> {
        if self.has_event_permission(event_id, group_id, perm) {
            Ok(())
        } else {
            Err(Error::Forbidden(format!(
                "Missing event permission: {perm}"
            )))
        }
    }

    // ------------------------------------------------------------------------
    // Resource-Lookup Assertions (Resolves entity ID -> scope -> permission)
    // ------------------------------------------------------------------------

    pub async fn assert_recipe_perm(&self, recipe_id: RecipeId, perm: RecipePerm) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"SELECT group_id AS "group_id: GroupId" FROM recipes WHERE recipe_id = $1"#,
            recipe_id.get()
        )
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Recipe",
                id: recipe_id.to_string(),
            })?;

        self.assert_group_perm(group_id, Permission::Recipe(perm))
    }

    pub async fn assert_ingredient_perm(
        &self,
        ingredient_id: IngredientId,
        perm: IngredientPerm,
    ) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"SELECT group_id AS "group_id: GroupId" FROM ingredients WHERE ingredient_id = $1"#,
            ingredient_id.get()
        )
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Ingredient",
                id: ingredient_id.to_string(),
            })?;

        self.assert_group_perm(group_id, Permission::Ingredient(perm))
    }

    pub async fn assert_inventory_perm(
        &self,
        inventory_id: InventoryId,
        perm: InventoryPerm,
    ) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"SELECT group_id AS "group_id: GroupId" FROM inventories WHERE inventory_id = $1"#,
            inventory_id.get()
        )
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Inventory",
                id: inventory_id.to_string(),
            })?;

        self.assert_group_perm(group_id, Permission::Inventory(perm))
    }

    pub async fn assert_event_action(&self, event_id: EventId, perm: EventPerm) -> Result<()> {
        self.assert_event_actions(event_id, [perm]).await
    }

    pub async fn assert_event_actions(
        &self,
        event_id: EventId,
        perms: impl IntoIterator<Item = EventPerm>,
    ) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"SELECT group_id AS "group_id: GroupId" FROM events WHERE event_id = $1"#,
            event_id.get()
        )
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Event",
                id: event_id.to_string(),
            })?;

        for perm in perms {
            self.assert_event_perm(event_id, group_id, Permission::Event(perm))?;
        }
        Ok(())
    }

    pub async fn assert_meal_action(&self, meal_id: MealId, perm: EventPerm) -> Result<()> {
        let row = sqlx::query!(
            r#"
            SELECT e.event_id AS "event_id: EventId", e.group_id AS "group_id: GroupId"
            FROM event_meals m
            JOIN events e ON e.event_id = m.event_id
            WHERE m.meal_id = $1
            "#,
            meal_id.get()
        )
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "Meal",
                id: meal_id.to_string(),
            })?;

        self.assert_event_perm(row.event_id, row.group_id, Permission::Event(perm))
    }

    pub async fn assert_food_prep_action(&self, prep_id: FoodPrepId, perm: EventPerm) -> Result<()> {
        let row = sqlx::query!(
            r#"
            SELECT e.event_id AS "event_id: EventId", e.group_id AS "group_id: GroupId"
            FROM food_prep fp
            JOIN events e ON e.event_id = fp.event_id
            WHERE fp.prep_id = $1
            "#,
            prep_id.get()
        )
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "FoodPrep",
                id: prep_id.to_string(),
            })?;

        self.assert_event_perm(row.event_id, row.group_id, Permission::Event(perm))
    }

    pub async fn assert_shopping_tour_action(
        &self,
        tour_id: TourId,
        perm: EventPerm,
    ) -> Result<()> {
        let row = sqlx::query!(
            r#"
            SELECT
                e.event_id AS "event_id: EventId",
                e.group_id AS "group_id: GroupId",
                t.completed
            FROM shopping_tours t
            JOIN events e ON e.event_id = t.event_id
            WHERE t.tour_id = $1
            "#,
            tour_id.get()
        )
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| Error::NotFound {
                entity: "ShoppingTour",
                id: tour_id.to_string(),
            })?;

        if row.completed && perm == EventPerm::ShoppingTourEdit {
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
        let user = User::from_request_parts(parts, state).await?;
        let Extension(foodlib): Extension<FoodLib> = Extension::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::Misc("FoodLib extension missing".into()))?;

        Self::load(&foodlib, user).await
    }
}