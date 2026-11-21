//! Shared application state for all transports (`api`, `frontend`, `mcp`).

use std::ops::Deref;

use axum::Extension;

pub type FoodLib = Extension<foodlib_new::FoodLib>;

#[derive(Clone)]
pub struct MyAppState {
    pub db: foodlib_new::FoodLib,
}

impl Deref for MyAppState {
    type Target = foodlib_new::FoodLib;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}
