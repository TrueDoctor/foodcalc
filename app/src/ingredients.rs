//! Ingredient use-cases.

use foodlib_new::entities::types::amount::Amount;
use foodlib_new::entities::types::ids::{
    GroupId, IngredientId, IngredientSourceId, PropertyId, StoreId, UnitId,
};
use foodlib_new::entities::types::money::Money;
use foodlib_new::ingredient::{CreateIngredient, CreateIngredientSource, Ingredient, IngredientSource};
use foodlib_new::property::IngredientProperties;
use foodlib_new::recipe::Recipe;
use foodlib_new::store::Store;
use foodlib_new::types::ids::UserId;
use foodlib_new::user::User;

use crate::app_ctx::{assert_can_move_to, AppCtx};
use crate::{Error, MyAppState, Result};

/// True if the user may edit an entity owned by `group_id`: admins may edit
/// anything; members may edit entities of groups they belong to.
pub fn can_edit(group_id: GroupId, user_group_ids: &[GroupId], user: &User) -> bool {
    user.is_admin || user_group_ids.contains(&group_id)
}

async fn user_group_ids(state: &MyAppState, user_id: UserId) -> Result<Vec<GroupId>> {
    Ok(state
        .users()
        .get_user_groups(user_id)
        .await?
        .into_iter()
        .map(|g| g.id)
        .collect())
}

async fn user_group_ids_opt(
    state: &MyAppState,
    user: Option<&User>,
) -> Result<Vec<GroupId>> {
    match user {
        Some(u) => user_group_ids(state, u.id).await,
        None => Ok(vec![]),
    }
}

/// Creates a new ingredient owned by the actor's personal group.
pub async fn create_ingredient(
    state: &MyAppState,
    actor: &AppCtx,
    name: String,
    energy: Amount,
    comment: Option<String>,
) -> Result<()> {
    let group = state.users().get_personal_group(actor.ctx.user.id).await?;
    state
        .ingredients()
        .create(CreateIngredient {
            name,
            energy,
            comment,
            group_id: group.id,
        })
        .await?;
    Ok(())
}

/// Updates an existing ingredient and optionally transfers it to another
/// group. Requires edit rights on the ingredient's owner group; moving to
/// another group additionally requires membership in (or admin over) the
/// target group. Returns the refreshed ingredient.
pub async fn update_ingredient(
    state: &MyAppState,
    actor: &AppCtx,
    mut ingredient: Ingredient,
    target_group: GroupId,
) -> Result<Ingredient> {
    let existing = state.ingredients().get(ingredient.id).await?;
    if !can_edit(existing.group_id, &actor.ctx.group_ids, &actor.ctx.user) {
        return Err(Error::Forbidden(
            "You don't have permission to edit this ingredient".into(),
        ));
    }

    ingredient.group_id = existing.group_id;
    if existing.group_id != target_group {
        assert_can_move_to(actor, target_group)?;
    }
    let updated = state.ingredients().update(ingredient).await?;
    if existing.group_id != target_group {
        state
            .ingredients()
            .set_group(updated.id, target_group)
            .await?;
    }
    Ok(state.ingredients().get(updated.id).await?)
}

/// Deletes an ingredient. Only administrators may delete ingredients.
pub async fn delete_ingredient(
    state: &MyAppState,
    user: &User,
    ingredient_id: IngredientId,
) -> Result<()> {
    if !user.is_admin {
        return Err(Error::Forbidden(
            "You don't have permission to delete ingredients".into(),
        ));
    }
    state.ingredients().delete(ingredient_id).await?;
    Ok(())
}

/// Lists the recipes that use an ingredient (shown before confirming a
/// delete). Only administrators may delete ingredients.
pub async fn gather_usages(
    state: &MyAppState,
    user: &User,
    ingredient_id: IngredientId,
) -> Result<Vec<Recipe>> {
    if !user.is_admin {
        return Err(Error::Forbidden(
            "You don't have permission to delete ingredients".into(),
        ));
    }
    Ok(state.ingredients().usages(ingredient_id).await?)
}

/// Everything the sources dialog renders, fetched in one use case so
/// transports only have to deal with presentation.
pub struct SourcesViewData {
    pub sources: Vec<IngredientSource>,
    pub stores: Vec<Store>,
    pub ingredient: Ingredient,
    pub user_group_ids: Vec<GroupId>,
}

/// Collects all data needed to render an ingredient's sources dialog.
pub async fn gather_sources(
    state: &MyAppState,
    user: Option<&User>,
    ingredient_id: IngredientId,
) -> Result<SourcesViewData> {
    let sources = state.ingredients().get_sources(ingredient_id).await?;
    let stores = state.stores().list().await?;
    let ingredient = state.ingredients().get(ingredient_id).await?;
    let user_group_ids = user_group_ids_opt(state, user)
        .await
        .unwrap_or_default();

    Ok(SourcesViewData {
        sources,
        stores,
        ingredient,
        user_group_ids,
    })
}

/// Creates or updates one ingredient source; an invalid `source_id` means
/// create (the sources dialog submits a sentinel id for the inline add row).
/// Requires edit rights on the ingredient's owner group.
pub async fn save_source(
    state: &MyAppState,
    user: &User,
    ingredient_id: IngredientId,
    source_id: IngredientSourceId,
    store_id: StoreId,
    package_size: Amount,
    price: Money,
    url: Option<String>,
    comment: Option<String>,
) -> Result<()> {
    let ingredient = state.ingredients().get(ingredient_id).await?;
    let group_ids = user_group_ids(state, user.id).await?;
    if !can_edit(ingredient.group_id, &group_ids, user) {
        return Err(Error::Forbidden(
            "You don't have permission to edit sources for this ingredient".into(),
        ));
    }

    if !source_id.is_valid() {
        state
            .ingredients()
            .add_source(CreateIngredientSource {
                ingredient_id,
                store_id,
                package_size,
                price,
                unit_id: UnitId(0),
                url,
                comment,
            })
            .await?;
    } else {
        state
            .ingredients()
            .update_source(IngredientSource {
                id: source_id,
                ingredient_id,
                store_id,
                package_size,
                price,
                unit_id: UnitId(0),
                url,
                comment,
            })
            .await?;
    }
    Ok(())
}

/// Deletes one ingredient source.
/// Requires edit rights on the ingredient's owner group.
pub async fn delete_source(
    state: &MyAppState,
    user: &User,
    ingredient_id: IngredientId,
    source_id: IngredientSourceId,
) -> Result<()> {
    let ingredient = state.ingredients().get(ingredient_id).await?;
    let group_ids = user_group_ids(state, user.id).await?;
    if !can_edit(ingredient.group_id, &group_ids, user) {
        return Err(Error::Forbidden(
            "You don't have permission to delete sources for this ingredient".into(),
        ));
    }
    state.ingredients().delete_source(source_id).await?;
    Ok(())
}

/// Everything the allergen (properties) dialog renders, fetched in one use
/// case so transports only have to deal with presentation.
pub struct PropertiesViewData {
    pub properties: IngredientProperties,
    pub ingredient: Ingredient,
    pub user_group_ids: Vec<GroupId>,
}

/// Collects all data needed to render an ingredient's allergen dialog.
pub async fn gather_properties(
    state: &MyAppState,
    user: Option<&User>,
    ingredient_id: IngredientId,
) -> Result<PropertiesViewData> {
    let properties = state
        .properties()
        .get_ingredient_properties(ingredient_id)
        .await?;
    let ingredient = state.ingredients().get(ingredient_id).await?;
    let user_group_ids = user_group_ids_opt(state, user)
        .await
        .unwrap_or_default();

    Ok(PropertiesViewData {
        properties,
        ingredient,
        user_group_ids,
    })
}

/// Removes one property (e.g. allergen tag) from an ingredient.
/// Requires edit rights on the ingredient's owner group.
pub async fn delete_property(
    state: &MyAppState,
    user: &User,
    ingredient_id: IngredientId,
    property_id: PropertyId,
) -> Result<()> {
    let ingredient = state.ingredients().get(ingredient_id).await?;
    let group_ids = user_group_ids(state, user.id).await?;
    if !can_edit(ingredient.group_id, &group_ids, user) {
        return Err(Error::Forbidden(
            "You don't have permission to edit this ingredient".into(),
        ));
    }
    state
        .properties()
        .remove_from_ingredient(ingredient_id, property_id)
        .await?;
    Ok(())
}

/// Adds one canonical allergen tag to an ingredient. `kind` is either
/// `"contains"` or `"may_contain"`.
/// Requires edit rights on the ingredient's owner group.
pub async fn add_property(
    state: &MyAppState,
    user: &User,
    ingredient_id: IngredientId,
    canonical: String,
    kind: String,
) -> Result<()> {
    let ingredient = state.ingredients().get(ingredient_id).await?;
    let group_ids = user_group_ids(state, user.id).await?;
    if !can_edit(ingredient.group_id, &group_ids, user) {
        return Err(Error::Forbidden(
            "You don't have permission to edit this ingredient".into(),
        ));
    }

    use foodlib_new::allergen::{canonical, ClassifiedProperty, PropertySource};
    if !canonical::ALL.contains(&canonical.as_str()) {
        return Err(Error::Validation {
            message: format!("Unknown allergen: {}", canonical),
        });
    }
    let source = match kind.as_str() {
        "contains" => PropertySource::Contains,
        "may_contain" => PropertySource::MayContain,
        _ => {
            return Err(Error::Validation {
                message: format!("Invalid kind: {}", kind),
            })
        }
    };

    state
        .allergens()
        .apply_to_ingredient(
            ingredient_id,
            &[ClassifiedProperty {
                canonical_name: canonical,
                source,
            }],
        )
        .await?;
    Ok(())
}
