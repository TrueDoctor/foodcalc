use crate::entities::user::User;
use crate::types::ids::{
    EventId, FoodPrepId, GroupId, IngredientId, InventoryId, MealId, RecipeId, ShoppingTourId,
};
use crate::{Error, Result};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    /// HTMX/Maud frontend.
    WebUi,
    /// JSON REST transport (SPA).
    RestApi,
    /// MCP transport — treated as automation.
    McpAgent,
    /// Public read-only status dashboard service.
    StatusApi,
}

impl Channel {
    pub fn as_str(self) -> &'static str {
        match self {
            Channel::WebUi => "web-ui",
            Channel::RestApi => "rest-api",
            Channel::McpAgent => "mcp-agent",
            Channel::StatusApi => "status-api",
        }
    }
}

/// Authenticated actor context carrying the resolved user, their group
/// memberships, and the transport channel the request arrived on.
///
/// Construct once per request/session via [`crate::AppState::build_ctx`] and
/// pass by reference (`&AppCtx`) into use-case functions.
#[derive(Clone)]
pub struct AppCtx {
    pub user: User,
    pub group_ids: Vec<GroupId>,
    pub channel: Channel,
    pool: Arc<PgPool>,
}

impl AppCtx {
    pub fn new(
        user: User,
        group_ids: Vec<GroupId>,
        channel: Channel,
        pool: Arc<PgPool>,
    ) -> Self {
        Self {
            user,
            group_ids,
            channel,
            pool,
        }
    }

    /// Rejects automation channels (MCP) for sensitive use-cases.
    pub fn require_human(&self) -> Result<()> {
        if self.channel == Channel::McpAgent {
            return Err(Error::Forbidden(
                "this action is not available to automation (MCP); use the web UI".into(),
            ));
        }
        Ok(())
    }

    /// Returns Forbidden unless the user has global admin privileges.
    pub fn require_admin(&self) -> Result<()> {
        if self.user.is_admin {
            Ok(())
        } else {
            Err(Error::Forbidden("Admin privileges required".into()))
        }
    }

    /// True if the user is an admin or belongs to the given group.
    pub fn can_access_group(&self, group_id: GroupId) -> bool {
        self.user.is_admin || self.group_ids.contains(&group_id)
    }

    /// Returns Forbidden unless the user has access to the given group.
    pub fn assert_group(&self, group_id: GroupId) -> Result<()> {
        if self.can_access_group(group_id) {
            Ok(())
        } else {
            Err(Error::Forbidden(
                "You don't have permission to modify this resource".into(),
            ))
        }
    }

    /// Authorizes moving a resource into `target_group_id`: admins may move
    /// anywhere; non-admins may only move into groups they belong to.
    pub fn assert_can_move_to(&self, target_group_id: GroupId) -> Result<()> {
        if self.can_access_group(target_group_id) {
            Ok(())
        } else {
            Err(Error::Forbidden(
                "You can only move resources into groups you belong to".into(),
            ))
        }
    }

    pub async fn assert_can_edit_recipe(&self, recipe_id: RecipeId) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"
            SELECT group_id as "group_id: GroupId" FROM recipes
            WHERE recipe_id = $1
            "#,
            recipe_id.get()
        )
            .fetch_optional(&*self.pool)
            .await?
            .ok_or(Error::NotFound {
                entity: "Recipe",
                id: recipe_id.to_string(),
            })?;
        self.assert_group(group_id)
    }

    pub async fn assert_can_edit_event(&self, event_id: EventId) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"SELECT group_id as "group_id: GroupId" FROM events WHERE event_id = $1"#,
            event_id.get()
        )
            .fetch_optional(&*self.pool)
            .await?
            .ok_or(Error::NotFound {
                entity: "Event",
                id: event_id.to_string(),
            })?;
        self.assert_group(group_id)
    }

    pub async fn assert_can_edit_ingredient(&self, ingredient_id: IngredientId) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"SELECT group_id as "group_id: GroupId" FROM ingredients WHERE ingredient_id = $1"#,
            ingredient_id.get()
        )
            .fetch_optional(&*self.pool)
            .await?
            .ok_or(Error::NotFound {
                entity: "Ingredient",
                id: ingredient_id.to_string(),
            })?;
        self.assert_group(group_id)
    }

    pub async fn assert_can_edit_inventory(&self, inventory_id: InventoryId) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"SELECT group_id as "group_id: GroupId" FROM inventories WHERE inventory_id = $1"#,
            inventory_id.get()
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
    pub async fn assert_can_edit_meal(&self, meal_id: MealId) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"
            SELECT e.group_id as "group_id: GroupId"
            FROM event_meals m
            JOIN events e ON e.event_id = m.event_id
            WHERE m.meal_id = $1
            "#,
            meal_id.get()
        )
            .fetch_optional(&*self.pool)
            .await?
            .ok_or(Error::NotFound {
                entity: "Meal",
                id: meal_id.to_string(),
            })?;
        self.assert_group(group_id)
    }

    pub async fn assert_can_edit_food_prep(&self, prep_id: FoodPrepId) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"
            SELECT e.group_id as "group_id: GroupId"
            FROM food_prep fp
            JOIN events e ON e.event_id = fp.event_id
            WHERE fp.prep_id = $1
            "#,
            prep_id.get()
        )
            .fetch_optional(&*self.pool)
            .await?
            .ok_or(Error::NotFound {
                entity: "FoodPrep",
                id: prep_id.to_string(),
            })?;
        self.assert_group(group_id)
    }

    pub async fn assert_can_edit_shopping_tour(&self, tour_id: ShoppingTourId) -> Result<()> {
        let group_id = sqlx::query_scalar!(
            r#"
            SELECT e.group_id as "group_id: GroupId"
            FROM shopping_tours t
            JOIN events e ON e.event_id = t.event_id
            WHERE t.tour_id = $1
            "#,
            tour_id.get()
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

impl std::fmt::Debug for AppCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppCtx")
            .field("user", &self.user.username)
            .field("group_ids", &self.group_ids)
            .field("channel", &self.channel)
            .finish_non_exhaustive()
    }
}

/// Convenience wrapper for existing call sites using `ctx::assert_can_move_to(actor, group_id)`.
pub fn assert_can_move_to(actor: &AppCtx, target_group_id: GroupId) -> Result<()> {
    actor.assert_can_move_to(target_group_id)
}