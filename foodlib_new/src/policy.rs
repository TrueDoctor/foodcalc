use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownPermissionError(pub String);

impl fmt::Display for UnknownPermissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown permission identifier: {}", self.0)
    }
}
impl std::error::Error for UnknownPermissionError {}

macro_rules! impl_str_serde {
    ($ty:ty) => {
        impl std::fmt::Display for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
        impl serde::Serialize for $ty {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }
        impl<'de> serde::Deserialize<'de> for $ty {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let s = String::deserialize(deserializer)?;
                s.parse().map_err(serde::de::Error::custom)
            }
        }
    };
}

mod evaluator;
mod global;
mod permissions;
mod roles;

pub use evaluator::{RoleSlug, UserPolicy};
pub use global::GlobalPermission;
pub use permissions::{
    EventPerm, GroupPerm, IngredientPerm, InventoryPerm, Permission, RecipePerm,
};
pub use roles::{
    all_definitions, permissions_for_role_slug, resolve_role_slugs, BuiltInRole, RoleCategory,
    RoleDefinition,
};