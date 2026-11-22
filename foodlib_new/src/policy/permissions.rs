use crate::policy::{GlobalPermission, UnknownPermissionError};
use serde::{Deserialize, Deserializer, Serializer};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    Event(EventPerm),
    Recipe(RecipePerm),
    Ingredient(IngredientPerm),
    Inventory(InventoryPerm),
    Group(GroupPerm),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventPerm {
    // --- Lifecycle & Frame Data ---
    /// View event overview, schedule, meals, and basic shopping lists.
    ViewEvent,
    /// Create new events in the group (also requires global `CreateResources`).
    CreateEvent,
    /// Edit event frame data (name, dates, budget, comment) and general settings.
    EditEvent,
    /// Permanently delete the event and all dependent records.
    DeleteEvent,
    /// Duplicate an existing event into a new event.
    CloneEvent,
    /// Reassign event ownership to another member of the group.
    TransferOwner,
    // --- Event Sub-Domains ---
    /// View financial calculations, cost breakdowns, and budget comparisons.
    ViewCalculation,
    /// Add, edit, reorder, and delete meals and portion counts on the event.
    ManageMeals,
    /// Create new shopping tours on the event.
    ShoppingTourCreate,
    /// Edit items and quantities on open (uncompleted) shopping tours.
    /// Also edit the tour date and store assignment.
    ShoppingTourEdit,
    /// Mark an open shopping tour as completed (locks further item edits).
    ShoppingTourComplete,
    /// Operate the live kitchen/serving status board (reorders and meal availability).
    StatusBoardOperate,
    /// Generate and download event exports (PDF/CSV shopping lists, meal plans).
    ExportMealPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecipePerm {
    /// View recipe steps, ingredients, and energy/nutrition calculations.
    View,
    /// Create new recipes in the group (also requires global `CreateResources`).
    Create,
    /// Edit existing recipe instructions, yields, and ingredient amounts.
    Edit,
    /// Permanently delete recipes from the group.
    Delete,
    /// Reassign the owning group of a recipe.
    TransferOwner,
    /// Export recipe sheets (e.g. Typst PDF).
    Export,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IngredientPerm {
    /// View ingredient details, units, prices, and allergen/property metadata.
    View,
    /// Create new ingredients in the group (also requires global `CreateResources`).
    Create,
    /// Edit ingredient metadata, unit conversions, and properties.
    Edit,
    /// Permanently delete unused ingredients from the group.
    Delete,
    /// Reassign the owning group of an ingredient.
    TransferOwner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InventoryPerm {
    /// View inventory locations and current stock levels.
    View,
    /// Create new inventories in the group (also requires global `CreateResources`).
    Create,
    /// Edit inventory metadata/settings (name, description).
    EditMetadata,
    /// Book stock movements (purchases in, meal consumption out, stock corrections).
    EditStock,
    /// Permanently delete an inventory and its stock history.
    Delete,
    /// Reassign the owning group of an inventory.
    TransferOwner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupPerm {
    /// Add/remove group members and assign or change their group roles.
    ManageMembers,
    /// Configure group settings (rename group, set group default member roles).
    ManageSettings,
    /// Permanently delete a non-personal, empty group.
    Delete,
}

impl Permission {
    pub const fn as_str(self) -> &'static str {
        match self {
            // Events
            Self::Event(EventPerm::ViewEvent) => "events:view",
            Self::Event(EventPerm::CreateEvent) => "events:create",
            Self::Event(EventPerm::EditEvent) => "events:edit",
            Self::Event(EventPerm::DeleteEvent) => "events:delete",
            Self::Event(EventPerm::CloneEvent) => "events:clone",
            Self::Event(EventPerm::TransferOwner) => "events:transfer_owner",
            Self::Event(EventPerm::ManageMeals) => "events:manage_meals",
            Self::Event(EventPerm::ViewCalculation) => "events:view_calculation",
            Self::Event(EventPerm::ShoppingTourCreate) => "events:shopping:add_tour",
            Self::Event(EventPerm::ShoppingTourEdit) => "events:shopping:edit_tour",
            Self::Event(EventPerm::ShoppingTourComplete) => "events:shopping:complete_tour",
            Self::Event(EventPerm::StatusBoardOperate) => "status_board:operate",
            Self::Event(EventPerm::ExportMealPlan) => "events:export",
            // Recipes
            Self::Recipe(RecipePerm::View) => "recipes:view",
            Self::Recipe(RecipePerm::Create) => "recipes:create",
            Self::Recipe(RecipePerm::Edit) => "recipes:edit",
            Self::Recipe(RecipePerm::Delete) => "recipes:delete",
            Self::Recipe(RecipePerm::TransferOwner) => "recipes:transfer_owner",
            Self::Recipe(RecipePerm::Export) => "recipes:export",
            // Ingredients
            Self::Ingredient(IngredientPerm::View) => "ingredients:view",
            Self::Ingredient(IngredientPerm::Create) => "ingredients:create",
            Self::Ingredient(IngredientPerm::Edit) => "ingredients:edit",
            Self::Ingredient(IngredientPerm::Delete) => "ingredients:delete",
            Self::Ingredient(IngredientPerm::TransferOwner) => "ingredients:transfer_owner",
            // Inventories
            Self::Inventory(InventoryPerm::View) => "inventories:view",
            Self::Inventory(InventoryPerm::Create) => "inventories:create",
            Self::Inventory(InventoryPerm::EditMetadata) => "inventories:edit",
            Self::Inventory(InventoryPerm::Delete) => "inventories:delete",
            Self::Inventory(InventoryPerm::EditStock) => "inventories:edit_stock",
            Self::Inventory(InventoryPerm::TransferOwner) => "inventories:transfer_owner",
            // Group
            Self::Group(GroupPerm::ManageMembers) => "group:manage_members",
            Self::Group(GroupPerm::ManageSettings) => "group:manage_settings",
            Self::Group(GroupPerm::Delete) => "group:delete",
        }
    }
}

impl FromStr for Permission {
    type Err = UnknownPermissionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // Events
            "events:view" => Ok(Self::Event(EventPerm::ViewEvent)),
            "events:create" => Ok(Self::Event(EventPerm::CreateEvent)),
            "events:edit" => Ok(Self::Event(EventPerm::EditEvent)),
            "events:delete" => Ok(Self::Event(EventPerm::DeleteEvent)),
            "events:clone" => Ok(Self::Event(EventPerm::CloneEvent)),
            "events:transfer_owner" => Ok(Self::Event(EventPerm::TransferOwner)),
            "events:manage_meals" => Ok(Self::Event(EventPerm::ManageMeals)),
            "events:manage_shopping" => Ok(Self::Event(EventPerm::ShoppingTourEdit)), // Legacy alias
            "events:view_calculation" => Ok(Self::Event(EventPerm::ViewCalculation)),
            "events:shopping:add_tour" => Ok(Self::Event(EventPerm::ShoppingTourCreate)),
            "events:shopping:edit_tour" => Ok(Self::Event(EventPerm::ShoppingTourEdit)),
            "events:shopping:complete_tour" => Ok(Self::Event(EventPerm::ShoppingTourComplete)),
            "events:export" => Ok(Self::Event(EventPerm::ExportMealPlan)),
            "status_board:operate" => Ok(Self::Event(EventPerm::StatusBoardOperate)),
            // Recipes
            "recipes:view" => Ok(Self::Recipe(RecipePerm::View)),
            "recipes:create" => Ok(Self::Recipe(RecipePerm::Create)),
            "recipes:edit" => Ok(Self::Recipe(RecipePerm::Edit)),
            "recipes:delete" => Ok(Self::Recipe(RecipePerm::Delete)),
            "recipes:export" => Ok(Self::Recipe(RecipePerm::Export)),
            "recipes:transfer_owner" => Ok(Self::Recipe(RecipePerm::TransferOwner)),
            // Ingredients
            "ingredients:view" => Ok(Self::Ingredient(IngredientPerm::View)),
            "ingredients:create" => Ok(Self::Ingredient(IngredientPerm::Create)),
            "ingredients:edit" => Ok(Self::Ingredient(IngredientPerm::Edit)),
            "ingredients:delete" => Ok(Self::Ingredient(IngredientPerm::Delete)),
            "ingredients:transfer_owner" => Ok(Self::Ingredient(IngredientPerm::TransferOwner)),
            // Inventories
            "inventories:view" => Ok(Self::Inventory(InventoryPerm::View)),
            "inventories:create" => Ok(Self::Inventory(InventoryPerm::Create)),
            "inventories:edit" => Ok(Self::Inventory(InventoryPerm::EditMetadata)),
            "inventories:edit_stock" => Ok(Self::Inventory(InventoryPerm::EditStock)),
            "inventories:delete" => Ok(Self::Inventory(InventoryPerm::Delete)),
            "inventories:transfer_owner" => Ok(Self::Inventory(InventoryPerm::TransferOwner)),
            // Group
            "group:manage_members" => Ok(Self::Group(GroupPerm::ManageMembers)),
            "group:manage_settings" => Ok(Self::Group(GroupPerm::ManageSettings)),
            "group:delete" => Ok(Self::Group(GroupPerm::Delete)),
            _ => Err(UnknownPermissionError(s.to_owned())),
        }
    }
}



// ============================================================================
// Serde & Display Wiring (Serializes directly as "events:edit", etc.)
// ============================================================================

impl_str_serde!(Permission);
impl_str_serde!(GlobalPermission);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_str_roundtrip() {
        let all = [
            Permission::Event(EventPerm::ViewEvent),
            Permission::Event(EventPerm::CreateEvent),
            Permission::Event(EventPerm::EditEvent),
            Permission::Event(EventPerm::DeleteEvent),
            Permission::Event(EventPerm::CloneEvent),
            Permission::Event(EventPerm::TransferOwner),
            Permission::Event(EventPerm::ViewCalculation),
            Permission::Event(EventPerm::ManageMeals),
            Permission::Event(EventPerm::ShoppingTourCreate),
            Permission::Event(EventPerm::ShoppingTourEdit),
            Permission::Event(EventPerm::ShoppingTourComplete),
            Permission::Event(EventPerm::StatusBoardOperate),
            Permission::Event(EventPerm::ExportMealPlan),
            Permission::Recipe(RecipePerm::View),
            Permission::Recipe(RecipePerm::Create),
            Permission::Recipe(RecipePerm::Edit),
            Permission::Recipe(RecipePerm::Delete),
            Permission::Recipe(RecipePerm::TransferOwner),
            Permission::Recipe(RecipePerm::Export),
            Permission::Ingredient(IngredientPerm::View),
            Permission::Ingredient(IngredientPerm::Create),
            Permission::Ingredient(IngredientPerm::Edit),
            Permission::Ingredient(IngredientPerm::Delete),
            Permission::Ingredient(IngredientPerm::TransferOwner),
            Permission::Inventory(InventoryPerm::View),
            Permission::Inventory(InventoryPerm::Create),
            Permission::Inventory(InventoryPerm::EditMetadata),
            Permission::Inventory(InventoryPerm::EditStock),
            Permission::Inventory(InventoryPerm::Delete),
            Permission::Inventory(InventoryPerm::TransferOwner),
            Permission::Group(GroupPerm::ManageMembers),
            Permission::Group(GroupPerm::ManageSettings),
            Permission::Group(GroupPerm::Delete),
        ];

        for perm in all {
            assert_eq!(Permission::from_str(perm.as_str()), Ok(perm));
        }
    }
}