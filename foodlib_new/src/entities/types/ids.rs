//! Domain ID newtypes.
//!
//! Every database identifier is a distinct zero-cost newtype instead of a bare
//! `i32`/`i64`. The compiler then rejects passing an `EventId` where a
//! `RecipeId` is expected (and vice versa), while the runtime cost is zero.
//!
//! Each wrapper is `#[sqlx(transparent)]`, so sqlx encodes and decodes it
//! exactly like the inner integer — `query_as!`/`query!` fetch rows into these
//! types and bind them back as parameters without any SQL changes. Changing
//! the underlying database type later means changing the inner type here.
//!
//! JSON serialization is a plain number, so REST/SPA clients are unaffected.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! make_id {
    (
        $(#[$meta:meta])*
        $name:ident : $inner:ty
    ) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Default,
            Serialize,
            Deserialize,
            sqlx::Type,
        )]
        #[sqlx(transparent)]
        #[serde(transparent)]
        $(#[$meta])*
        pub struct $name(pub $inner);

        impl $name {
            /// The raw database value.
            #[inline]
            pub fn get(self) -> $inner {
                self.0
            }
        }

        impl From<$inner> for $name {
            #[inline]
            fn from(v: $inner) -> Self {
                Self(v)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

make_id!(/// Primary key of the `users` table.
    UserId: i64);
make_id!(/// Primary key of the `groups` table.
    GroupId: i32);
make_id!(/// Primary key of the `events` table.
    EventId: i32);
make_id!(/// Primary key of the `recipes` table.
    RecipeId: i32);
make_id!(/// Primary key of the `recipe_steps` table.
    RecipeStepId: i32);
make_id!(/// Primary key of the `ingredients` table.
    IngredientId: i32);
make_id!(/// Primary key of the `stores` table.
    StoreId: i32);
make_id!(/// Primary key of the `units` table.
    UnitId: i32);
make_id!(/// Primary key of the `inventories` table.
    InventoryId: i32);
make_id!(/// Primary key of the `food_properties` table.
    PropertyId: i32);
make_id!(/// Primary key of the `meals` table.
    MealId: i32);
make_id!(/// Primary key of the `places` table.
    PlaceId: i32);
make_id!(/// Primary key of the `shopping_tours` table.
    TourId: i32);
make_id!(/// Primary key of the `food_preps` table.
    FoodPrepId: i32);
make_id!(/// Primary key of the `ingredient_sources` table.
    IngredientSourceId: i32);
make_id!(/// Primary key of the `audit_log` table.
    AuditId: i64);

make_id!( /// Id of a reorder request currently not in database
    ReorderId: i32 
);

/// Polymorphic entity ID used with `AuditRecord::entity_type`.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
    sqlx::Type,
)]
#[sqlx(transparent)]
#[serde(transparent)]
pub struct EntityId(Box<str>);
impl EntityId {
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}


macro_rules! impl_entity_id_from {
    ($($source:ty),* $(,)?) => {
        $(
            impl From<$source> for EntityId {
                fn from(id: $source) -> Self {
                    EntityId(id.get().to_string().into_boxed_str())
                }
            }
        )*
    };
}

impl_entity_id_from!(
    UserId,
    GroupId,
    EventId,
    RecipeId,
    RecipeStepId,
    IngredientId,
    StoreId,
    UnitId,
    InventoryId,
    PropertyId,
    MealId,
    PlaceId,
    TourId,
    FoodPrepId,
    IngredientSourceId,
);



#[cfg(test)]
mod tests {
    use super::*;

    /// IDs are Copy, comparable, hashable and serialize as plain numbers.
    #[test]
    fn ids_are_zero_cost_values() {
        let user = UserId(1);
        let copied = user;
        assert_eq!(user, copied);
        assert_eq!(user.get(), 1i64);
        assert_eq!(user.to_string(), "1");

        let event: EventId = 7.into();
        assert_eq!(event, EventId(7));
        assert_ne!(event, EventId::default());

        // Wrong-ID mix-ups don't compile, so no runtime test possible —
        // this is exactly what the type system buys us.
    }

    #[test]
    fn ids_roundtrip_through_json() {
        assert_eq!(serde_json::to_string(&RecipeId(42)).unwrap(), "42");
        let back: RecipeId = serde_json::from_str("42").unwrap();
        assert_eq!(back, RecipeId(42));
    }
}
