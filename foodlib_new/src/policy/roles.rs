use super::permissions::{
    EventPerm, GroupPerm, IngredientPerm, InventoryPerm, Permission, RecipePerm,
};
use serde::{Deserializer, Serialize, Serializer};
use std::collections::HashSet;

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
    /// Legacy slugs map onto their R2 successor (`editor` -> `team`).
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
                Permission::Event(Ev::ViewEvent),
                Permission::Event(Ev::ViewCalculation),
                Permission::Event(Ev::ExportMealPlan),
                Permission::Recipe(Rec::View),
                Permission::Recipe(Rec::Export),
                Permission::Ingredient(Ing::View),
                Permission::Inventory(Inv::View),
            ],
            Self::Cook => &[
                Permission::Event(Ev::ViewEvent),
                Permission::Event(Ev::ViewCalculation),
                Permission::Event(Ev::ExportMealPlan),
                Permission::Event(Ev::EditEvent),
                Permission::Event(Ev::ManageMeals),
                Permission::Event(Ev::StatusBoardOperate),
                Permission::Recipe(Rec::View),
                Permission::Recipe(Rec::Export),
                Permission::Recipe(Rec::Create),
                Permission::Recipe(Rec::Edit),
                Permission::Ingredient(Ing::View),
                Permission::Ingredient(Ing::Create),
                Permission::Ingredient(Ing::Edit),
                Permission::Inventory(Inv::View),
                Permission::Inventory(Inv::EditMetadata),
            ],
            Self::Shopper => &[
                Permission::Event(Ev::ViewEvent),
                Permission::Event(Ev::ViewCalculation),
                Permission::Event(Ev::ExportMealPlan),
                Permission::Event(Ev::ShoppingTourCreate),
                Permission::Event(Ev::ShoppingTourEdit),
                Permission::Event(Ev::ShoppingTourComplete),
                Permission::Recipe(Rec::View),
                Permission::Ingredient(Ing::View),
                Permission::Inventory(Inv::View),
                Permission::Inventory(Inv::EditMetadata),
            ],
            Self::Stocker => &[
                Permission::Event(Ev::ViewEvent),
                Permission::Event(Ev::ViewCalculation),
                Permission::Event(Ev::ShoppingTourComplete),
                Permission::Recipe(Rec::View),
                Permission::Ingredient(Ing::View),
                Permission::Ingredient(Ing::Edit),
                Permission::Inventory(Inv::View),
                Permission::Inventory(Inv::Create),
                Permission::Inventory(Inv::EditMetadata),
            ],
            Self::Serving => &[
                Permission::Event(Ev::ViewEvent),
                Permission::Event(Ev::StatusBoardOperate),
                Permission::Recipe(Rec::View),
                Permission::Ingredient(Ing::View),
                Permission::Inventory(Inv::View),
            ],
            Self::Team => &[
                Permission::Event(Ev::ViewEvent),
                Permission::Event(Ev::ViewCalculation),
                Permission::Event(Ev::ExportMealPlan),
                Permission::Event(Ev::CreateEvent),
                Permission::Event(Ev::EditEvent),
                Permission::Event(Ev::CloneEvent),
                Permission::Event(Ev::ManageMeals),
                Permission::Event(Ev::ShoppingTourCreate),
                Permission::Event(Ev::ShoppingTourEdit),
                Permission::Event(Ev::ShoppingTourComplete),
                Permission::Event(Ev::StatusBoardOperate),
                Permission::Recipe(Rec::View),
                Permission::Recipe(Rec::Export),
                Permission::Recipe(Rec::Create),
                Permission::Recipe(Rec::Edit),
                Permission::Ingredient(Ing::View),
                Permission::Ingredient(Ing::Create),
                Permission::Ingredient(Ing::Edit),
                Permission::Inventory(Inv::View),
                Permission::Inventory(Inv::Create),
                Permission::Inventory(Inv::EditMetadata),
            ],
            Self::Manager => &[
                Permission::Event(Ev::ViewEvent),
                Permission::Event(Ev::ViewCalculation),
                Permission::Event(Ev::ExportMealPlan),
                Permission::Event(Ev::CreateEvent),
                Permission::Event(Ev::EditEvent),
                Permission::Event(Ev::CloneEvent),
                Permission::Event(Ev::ManageMeals),
                Permission::Event(Ev::ShoppingTourCreate),
                Permission::Event(Ev::ShoppingTourEdit),
                Permission::Event(Ev::ShoppingTourComplete),
                Permission::Event(Ev::StatusBoardOperate),
                Permission::Event(Ev::DeleteEvent),
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
                Permission::Inventory(Inv::EditMetadata),
                Permission::Inventory(Inv::Delete),
                Permission::Inventory(Inv::TransferOwner),
                Permission::Group(GroupPerm::ManageMembers),
                Permission::Group(GroupPerm::ManageSettings),
                Permission::Group(GroupPerm::Delete),
            ],
        }
    }

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

/// Union of deduplicated permissions across multiple role slugs.
pub fn resolve_role_slugs(slugs: &[String]) -> Vec<Permission> {
    let mut set = HashSet::new();
    for slug in slugs {
        set.extend(permissions_for_role_slug(slug).iter().copied());
    }
    set.into_iter().collect()
}
