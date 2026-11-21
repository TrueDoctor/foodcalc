//! Shared application state for all transports (`api`, `frontend`, `mcp`).

use crate::app::exports::{default_pdf_exporter, PdfExporter};
use axum::Extension;
use std::ops::Deref;
use std::sync::Arc;

pub type FoodLib = Extension<foodlib_new::FoodLib>;

#[derive(Clone)]
pub struct MyAppState {
    pub db: foodlib_new::FoodLib,
    pub pdf: Arc<dyn PdfExporter>,
}

impl Deref for MyAppState {
    type Target = foodlib_new::FoodLib;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl MyAppState {
    pub fn new(db: foodlib_new::FoodLib) -> Self {
        Self {
            db,
            pdf: default_pdf_exporter(),
        }
    }

    pub fn pdf(&self) -> Arc<dyn PdfExporter> {
        Arc::clone(&self.pdf)
    }
}