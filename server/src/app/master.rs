//! Master data use-cases: stores, units, properties and places. Used by the
//! pickers in the UI; groups live in `app::users` (admin-aware).

use foodlib_new::entities::event::Place;
use foodlib_new::entities::property::Property;
use foodlib_new::entities::store::Store;
use foodlib_new::entities::unit::Unit;

use super::{MyAppState, Result};

pub async fn get_available_stores(state: &MyAppState) -> Result<Vec<Store>> {
    state.stores().list().await
}

pub async fn get_available_units(state: &MyAppState) -> Result<Vec<Unit>> {
    state.units().list().await
}

pub async fn get_food_properties(state: &MyAppState) -> Result<Vec<Property>> {
    state.properties().list().await
}

/// Places where meals are served (event meal picker).
pub async fn get_available_event_places(state: &MyAppState) -> Result<Vec<Place>> {
    state.events().get_places().await
}
