use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;
// ============================================================================
// 1. Global Permissions (User-scoped, independent of any Group or Event)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalPermission {
    ManageUsers,
    ManageMasterData,
    AllowedToCreateNewDomainObjects, // Groups, Events, Recipes, Ingredients, Inventories // Lock-down a user
}

impl GlobalPermission {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ManageUsers => "admin:manage_users",
            Self::ManageMasterData => "admin:manage_master_data",
            Self::AllowedToCreateNewDomainObjects => "admin:allow_creating", // Positive permission for blocking users
        }
    }
}

impl FromStr for GlobalPermission {
    type Err = UnknownPermissionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin:manage_users" => Ok(Self::ManageUsers),
            "admin:manage_master_data" => Ok(Self::ManageMasterData),
            "admin:allow_creating" => Ok(Self::AllowedToCreateNewDomainObjects),
            _ => Err(UnknownPermissionError(s.to_owned())),
        }
    }
}

// ============================================================================
// 2. Scoped Permissions (Namespaced by Domain)
// ============================================================================

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
    View,
    Create,
    Edit,
    Delete,
    Clone,
    TransferOwner,
    ManageMeals,
    ManageShopping,
    ViewCalculation,
    ShoppingAddTour,
    ShoppingEditTour,
    OperateStatusBoard,
    Export,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecipePerm {
    View,
    Create,
    Edit,
    Delete,
    TransferOwner,
    Export,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IngredientPerm {
    View,
    Create,
    Edit,
    Delete,
    TransferOwner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InventoryPerm {
    View,
    Create,
    Edit,
    Delete,
    TransferOwner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupPerm {
    ManageMembers,
    Delete,
}

impl Permission {
    pub const fn as_str(self) -> &'static str {
        match self {
            // Events
            Self::Event(EventPerm::View) => "events:view",
            Self::Event(EventPerm::Create) => "events:create",
            Self::Event(EventPerm::Edit) => "events:edit",
            Self::Event(EventPerm::Delete) => "events:delete",
            Self::Event(EventPerm::Clone) => "events:clone",
            Self::Event(EventPerm::TransferOwner) => "events:transfer_owner",
            Self::Event(EventPerm::ManageMeals) => "events:manage_meals",
            Self::Event(EventPerm::ManageShopping) => "events:manage_shopping",
            Self::Event(EventPerm::ViewCalculation) => "events:view_calculation",
            Self::Event(EventPerm::ShoppingAddTour) => "events:shopping:add_tour",
            Self::Event(EventPerm::ShoppingEditTour) => "events:shopping:edit_tour",
            Self::Event(EventPerm::OperateStatusBoard) => "status_board:operate",
            Self::Event(EventPerm::Export) => "events:export",
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
            Self::Inventory(InventoryPerm::Edit) => "inventories:edit",
            Self::Inventory(InventoryPerm::Delete) => "inventories:delete",
            Self::Inventory(InventoryPerm::TransferOwner) => "inventories:transfer_owner",
            // Group
            Self::Group(GroupPerm::ManageMembers) => "group:manage_members",
            Self::Group(GroupPerm::Delete) => "group:delete",
        }
    }
}

impl FromStr for Permission {
    type Err = UnknownPermissionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // Events
            "events:view" => Ok(Self::Event(EventPerm::View)),
            "events:create" => Ok(Self::Event(EventPerm::Create)),
            "events:edit" => Ok(Self::Event(EventPerm::Edit)),
            "events:delete" => Ok(Self::Event(EventPerm::Delete)),
            "events:clone" => Ok(Self::Event(EventPerm::Clone)),
            "events:transfer_owner" => Ok(Self::Event(EventPerm::TransferOwner)),
            "events:manage_meals" => Ok(Self::Event(EventPerm::ManageMeals)),
            "events:manage_shopping" => Ok(Self::Event(EventPerm::ManageShopping)),
            "events:view_calculation" => Ok(Self::Event(EventPerm::ViewCalculation)),
            "events:shopping:add_tour" => Ok(Self::Event(EventPerm::ShoppingAddTour)),
            "events:shopping:edit_tour" => Ok(Self::Event(EventPerm::ShoppingEditTour)),
            "events:export" => Ok(Self::Event(EventPerm::Export)),
            "status_board:operate" => Ok(Self::Event(EventPerm::OperateStatusBoard)),
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
            "inventories:edit" => Ok(Self::Inventory(InventoryPerm::Edit)),
            "inventories:delete" => Ok(Self::Inventory(InventoryPerm::Delete)),
            "inventories:transfer_owner" => Ok(Self::Inventory(InventoryPerm::TransferOwner)),
            // Group
            "group:manage_members" => Ok(Self::Group(GroupPerm::ManageMembers)),
            "group:delete" => Ok(Self::Group(GroupPerm::Delete)),
            _ => Err(UnknownPermissionError(s.to_owned())),
        }
    }
}

// ============================================================================
// 3. Serde & Display Wiring (Serializes directly as "events:edit", etc.)
// ============================================================================

#[derive(Debug, Clone)]
pub struct UnknownPermissionError(pub String);

impl fmt::Display for UnknownPermissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown permission identifier: {}", self.0)
    }
}
impl std::error::Error for UnknownPermissionError {}

macro_rules! impl_str_serde {
    ($ty:ty) => {
        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
        impl Serialize for $ty {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }
        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let s = String::deserialize(deserializer)?;
                s.parse().map_err(serde::de::Error::custom)
            }
        }
    };
}

impl_str_serde!(Permission);
impl_str_serde!(GlobalPermission);

// ============================================================================
// 4. Hard-coded Built-in Roles (Viewer, Editor, Manager)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltInRole {
    Viewer,
    Editor,
    Manager,
}

impl BuiltInRole {
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "viewer" => Some(Self::Viewer),
            "editor" => Some(Self::Editor),
            "manager" => Some(Self::Manager),
            _ => None,
        }
    }

    pub const fn permissions(self) -> &'static [Permission] {
        use IngredientPerm as Ing;
        use InventoryPerm as Inv;
        use RecipePerm as Rec;

        match self {
            Self::Viewer => &[
                Permission::Event(EventPerm::View),
                Permission::Event(EventPerm::ViewCalculation),
                Permission::Recipe(Rec::View),
                Permission::Ingredient(Ing::View),
                Permission::Inventory(Inv::View),
            ],
            Self::Editor => &[
                Permission::Event(EventPerm::View),
                Permission::Event(EventPerm::ViewCalculation),
                Permission::Recipe(Rec::View),
                Permission::Ingredient(Ing::View),
                Permission::Inventory(Inv::View),
                Permission::Event(EventPerm::Create),
                Permission::Event(EventPerm::Edit),
                Permission::Event(EventPerm::Clone),
                Permission::Event(EventPerm::ManageMeals),
                Permission::Event(EventPerm::ManageShopping),
                Permission::Event(EventPerm::ShoppingAddTour),
                Permission::Event(EventPerm::ShoppingEditTour),
                Permission::Event(EventPerm::OperateStatusBoard),
                Permission::Recipe(Rec::Create),
                Permission::Recipe(Rec::Edit),
                Permission::Ingredient(Ing::Create),
                Permission::Ingredient(Ing::Edit),
                Permission::Inventory(Inv::Create),
                Permission::Inventory(Inv::Edit),
            ],
            Self::Manager => &[
                Permission::Event(EventPerm::View),
                Permission::Event(EventPerm::ViewCalculation),
                Permission::Recipe(Rec::View),
                Permission::Ingredient(Ing::View),
                Permission::Inventory(Inv::View),
                Permission::Event(EventPerm::Create),
                Permission::Event(EventPerm::Edit),
                Permission::Event(EventPerm::Clone),
                Permission::Event(EventPerm::ManageMeals),
                Permission::Event(EventPerm::ManageShopping),
                Permission::Event(EventPerm::ShoppingAddTour),
                Permission::Event(EventPerm::ShoppingEditTour),
                Permission::Event(EventPerm::OperateStatusBoard),
                Permission::Recipe(Rec::Create),
                Permission::Recipe(Rec::Edit),
                Permission::Ingredient(Ing::Create),
                Permission::Ingredient(Ing::Edit),
                Permission::Inventory(Inv::Create),
                Permission::Inventory(Inv::Edit),
                Permission::Event(EventPerm::Delete),
                Permission::Event(EventPerm::TransferOwner),
                Permission::Recipe(Rec::Delete),
                Permission::Recipe(Rec::TransferOwner),
                Permission::Ingredient(Ing::Delete),
                Permission::Ingredient(Ing::TransferOwner),
                Permission::Inventory(Inv::Delete),
                Permission::Inventory(Inv::TransferOwner),
                Permission::Group(GroupPerm::ManageMembers),
                Permission::Group(GroupPerm::Delete),
            ],
        }
    }
}

/// Single seam to replace later if roles ever move into the database.
pub fn permissions_for_role_slug(slug: &str) -> &'static [Permission] {
    BuiltInRole::from_slug(slug)
        .map(BuiltInRole::permissions)
        .unwrap_or(&[])
}

// ============================================================================
// 5. Dynamic Policy Evaluator (Loaded per-request into AuthContext)
// ============================================================================

/// Dynamic role slug (e.g., "viewer", "editor", "manager", or a custom DB role).
pub type RoleSlug = String;

#[derive(Debug, Default, Clone)]
pub struct UserPolicy {
    pub user_id: i32,
    pub global_permissions: HashSet<GlobalPermission>,
    /// Map of `group_id -> resolved set of permissions` (union of all roles in that group)
    pub group_permissions: HashMap<i32, HashSet<Permission>>,
    /// Map of `event_id -> direct event-level permissions` (union of direct event roles)
    pub event_overrides: HashMap<i32, HashSet<Permission>>,
}

impl UserPolicy {
    pub fn has_global_permission(&self, perm: GlobalPermission) -> bool {
        self.global_permissions.contains(&perm)
    }

    pub fn has_group_permission(&self, group_id: i32, perm: Permission) -> bool {
        self.group_permissions
            .get(&group_id)
            .is_some_and(|perms| perms.contains(&perm))
    }

    /// Evaluates `Group Permissions ∪ Direct Event Override Permissions`.
    pub fn has_event_permission(&self, event_id: i32, group_id: i32, perm: Permission) -> bool {
        self.has_group_permission(group_id, perm)
            || self
            .event_overrides
            .get(&event_id)
            .is_some_and(|perms| perms.contains(&perm))
    }

    /// Returns the complete resolved permission set for serialization into `EventDetail`.
    pub fn resolved_event_permissions(&self, event_id: i32, group_id: i32) -> Vec<Permission> {
        let mut combined = self
            .group_permissions
            .get(&group_id)
            .cloned()
            .unwrap_or_default();

        if let Some(overrides) = self.event_overrides.get(&event_id) {
            combined.extend(overrides.iter().copied());
        }

        combined.into_iter().collect()
    }
}