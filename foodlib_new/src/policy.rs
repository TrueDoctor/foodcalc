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
    /// System configuration (e.g. global default member roles).
    ManageSettings,
    /// Reading the audit log (ISO 27001 A.8.15 / A.5.28).
    ViewAuditLog,
    ViewUserProfiles,
}

impl GlobalPermission {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ManageUsers => "admin:manage_users",
            Self::ManageMasterData => "admin:manage_master_data",
            Self::AllowedToCreateNewDomainObjects => "admin:allow_creating", // Positive permission for blocking users
            Self::ManageSettings => "admin:manage_settings",
            Self::ViewAuditLog => "admin:view_audit_log",
            Self::ViewUserProfiles => "admin:view_user_profiles",
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
            "admin:manage_settings" => Ok(Self::ManageSettings),
            "admin:view_audit_log" => Ok(Self::ViewAuditLog),
            "admin:view_user_profiles" => Ok(Self::ViewUserProfiles),
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
    /// Group configuration: rename, default member roles.
    ManageSettings,
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
            "group:manage_settings" => Ok(Self::Group(GroupPerm::ManageSettings)),
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
// 4. Built-in Role Catalog (R2)
//
// Single source of truth for role definitions, served to the SPA via
// `GET /api/roles` — the frontend never hardcodes role slugs. If roles move
// into the database later, only this section and `permissions_for_role_slug`
// change; stored `user_group_roles.role_slug` values remain valid.
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltInRole {
    /// Leser — read-only baseline.
    Viewer,
    /// Koch — meals, recipes, ingredients, inventory bookings, status board.
    Cook,
    /// Einkauf — shopping tours and related inventory bookings.
    Shopper,
    /// Lager — inventory administration.
    Stocker,
    /// Ausgabe — serving counter / status board operation.
    Serving,
    /// Teammitglied — composite allrounder without destructive rights.
    Team,
    /// Leitung — everything, including destructive and member management.
    Manager,
}

impl BuiltInRole {
    /// Legacy slugs map onto their R2 successor (`editor` → `team`).
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "viewer" => Some(Self::Viewer),
            "cook" => Some(Self::Cook),
            "shopper" => Some(Self::Shopper),
            "stocker" => Some(Self::Stocker),
            "serving" => Some(Self::Serving),
            "team" | "editor" => Some(Self::Team),
            "manager" => Some(Self::Manager),
            _ => None,
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Self::Viewer => "viewer",
            Self::Cook => "cook",
            Self::Shopper => "shopper",
            Self::Stocker => "stocker",
            Self::Serving => "serving",
            Self::Team => "team",
            Self::Manager => "manager",
        }
    }

    pub const fn permissions(self) -> &'static [Permission] {
        use EventPerm as Ev;
        use IngredientPerm as Ing;
        use InventoryPerm as Inv;
        use RecipePerm as Rec;

        match self {
            Self::Viewer => &[
                Permission::Event(Ev::View),
                Permission::Event(Ev::ViewCalculation),
                Permission::Event(Ev::Export),
                Permission::Recipe(Rec::View),
                Permission::Recipe(Rec::Export),
                Permission::Ingredient(Ing::View),
                Permission::Inventory(Inv::View),
            ],
            Self::Cook => &[
                Permission::Event(Ev::View),
                Permission::Event(Ev::ViewCalculation),
                Permission::Event(Ev::Export),
                Permission::Event(Ev::Edit),
                Permission::Event(Ev::ManageMeals),
                Permission::Event(Ev::OperateStatusBoard),
                Permission::Recipe(Rec::View),
                Permission::Recipe(Rec::Export),
                Permission::Recipe(Rec::Create),
                Permission::Recipe(Rec::Edit),
                Permission::Ingredient(Ing::View),
                Permission::Ingredient(Ing::Create),
                Permission::Ingredient(Ing::Edit),
                Permission::Inventory(Inv::View),
                Permission::Inventory(Inv::Edit),
            ],
            Self::Shopper => &[
                Permission::Event(Ev::View),
                Permission::Event(Ev::ViewCalculation),
                Permission::Event(Ev::Export),
                Permission::Event(Ev::ManageShopping),
                Permission::Event(Ev::ShoppingAddTour),
                Permission::Event(Ev::ShoppingEditTour),
                Permission::Recipe(Rec::View),
                Permission::Ingredient(Ing::View),
                Permission::Inventory(Inv::View),
                Permission::Inventory(Inv::Edit),
            ],
            Self::Stocker => &[
                Permission::Event(Ev::View),
                Permission::Event(Ev::ViewCalculation),
                Permission::Recipe(Rec::View),
                Permission::Ingredient(Ing::View),
                Permission::Ingredient(Ing::Edit),
                Permission::Inventory(Inv::View),
                Permission::Inventory(Inv::Create),
                Permission::Inventory(Inv::Edit),
            ],
            Self::Serving => &[
                Permission::Event(Ev::View),
                Permission::Event(Ev::OperateStatusBoard),
                Permission::Recipe(Rec::View),
                Permission::Ingredient(Ing::View),
                Permission::Inventory(Inv::View),
            ],
            Self::Team => &[
                Permission::Event(Ev::View),
                Permission::Event(Ev::ViewCalculation),
                Permission::Event(Ev::Export),
                Permission::Event(Ev::Create),
                Permission::Event(Ev::Edit),
                Permission::Event(Ev::Clone),
                Permission::Event(Ev::ManageMeals),
                Permission::Event(Ev::ManageShopping),
                Permission::Event(Ev::ShoppingAddTour),
                Permission::Event(Ev::ShoppingEditTour),
                Permission::Event(Ev::OperateStatusBoard),
                Permission::Recipe(Rec::View),
                Permission::Recipe(Rec::Export),
                Permission::Recipe(Rec::Create),
                Permission::Recipe(Rec::Edit),
                Permission::Ingredient(Ing::View),
                Permission::Ingredient(Ing::Create),
                Permission::Ingredient(Ing::Edit),
                Permission::Inventory(Inv::View),
                Permission::Inventory(Inv::Create),
                Permission::Inventory(Inv::Edit),
            ],
            Self::Manager => &[
                Permission::Event(Ev::View),
                Permission::Event(Ev::ViewCalculation),
                Permission::Event(Ev::Export),
                Permission::Event(Ev::Create),
                Permission::Event(Ev::Edit),
                Permission::Event(Ev::Clone),
                Permission::Event(Ev::ManageMeals),
                Permission::Event(Ev::ManageShopping),
                Permission::Event(Ev::ShoppingAddTour),
                Permission::Event(Ev::ShoppingEditTour),
                Permission::Event(Ev::OperateStatusBoard),
                Permission::Event(Ev::Delete),
                Permission::Event(Ev::TransferOwner),
                Permission::Recipe(Rec::View),
                Permission::Recipe(Rec::Export),
                Permission::Recipe(Rec::Create),
                Permission::Recipe(Rec::Edit),
                Permission::Recipe(Rec::Delete),
                Permission::Recipe(Rec::TransferOwner),
                Permission::Ingredient(Ing::View),
                Permission::Ingredient(Ing::Create),
                Permission::Ingredient(Ing::Edit),
                Permission::Ingredient(Ing::Delete),
                Permission::Ingredient(Ing::TransferOwner),
                Permission::Inventory(Inv::View),
                Permission::Inventory(Inv::Create),
                Permission::Inventory(Inv::Edit),
                Permission::Inventory(Inv::Delete),
                Permission::Inventory(Inv::TransferOwner),
                Permission::Group(GroupPerm::ManageMembers),
                Permission::Group(GroupPerm::ManageSettings),
                Permission::Group(GroupPerm::Delete),
            ],
        }
    }
}

/// Category used to group roles in the assignment UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleCategory {
    /// General membership presets (Leser, Teammitglied, Leitung).
    Basic,
    /// Domain roles for departments (Küche, Einkauf, Lager, Ausgabe).
    Domain,
}

/// Role definition as served to the SPA via `GET /api/roles`.
/// The frontend renders roles purely from this catalog — never hardcoded.
#[derive(Debug, Clone, Serialize)]
pub struct RoleDefinition {
    pub slug: &'static str,
    /// German display name; the SPA overrides it via `common:roles.<slug>`.
    pub name: &'static str,
    pub description: &'static str,
    pub category: RoleCategory,
    pub is_system: bool,
    pub permissions: Vec<Permission>,
}

impl BuiltInRole {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Viewer => "Leser",
            Self::Cook => "Koch",
            Self::Shopper => "Einkauf",
            Self::Stocker => "Lager",
            Self::Serving => "Ausgabe",
            Self::Team => "Teammitglied",
            Self::Manager => "Leitung",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::Viewer => "Darf Veranstaltungen, Rezepte, Zutaten und Inventare ansehen und Exporte herunterladen.",
            Self::Cook => "Plant Mahlzeiten, pflegt Rezepte und Zutaten und bucht verbrauchte Mengen aus Inventaren ab.",
            Self::Shopper => "Plant und bearbeitet Einkaufstouren und bucht eingekaufte Mengen in Inventare.",
            Self::Stocker => "Verwaltet Inventare und deren Bestände sowie Zutatenstammdaten.",
            Self::Serving => "Bedient das Statusboard (Küche/Ausgabe) während der Veranstaltung.",
            Self::Team => "Allrounder ohne Destruktives: kann planen und bearbeiten, aber nichts löschen, übertragen oder Mitglieder verwalten.",
            Self::Manager => "Volle Kontrolle: inklusive Löschen, Eigentumsübertragung und Mitgliederverwaltung der Gruppe.",
        }
    }

    pub const fn category(self) -> RoleCategory {
        match self {
            Self::Viewer | Self::Team | Self::Manager => RoleCategory::Basic,
            Self::Cook | Self::Shopper | Self::Stocker | Self::Serving => RoleCategory::Domain,
        }
    }

    pub const fn all() -> [BuiltInRole; 7] {
        [
            Self::Viewer,
            Self::Cook,
            Self::Shopper,
            Self::Stocker,
            Self::Serving,
            Self::Team,
            Self::Manager,
        ]
    }

    pub fn definition(self) -> RoleDefinition {
        RoleDefinition {
            slug: self.slug(),
            name: self.name(),
            description: self.description(),
            category: self.category(),
            is_system: true,
            permissions: self.permissions().to_vec(),
        }
    }
}

/// The complete role catalog for `GET /api/roles`.
pub fn all_definitions() -> Vec<RoleDefinition> {
    BuiltInRole::all()
        .iter()
        .map(|role| role.definition())
        .collect()
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