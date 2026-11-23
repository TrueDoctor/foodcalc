use std::sync::Arc;

use crate::exports::{default_pdf_exporter, PdfExporter};

#[derive(Clone)]
pub struct MyAppState {
    pub db: foodlib_new::FoodLib,
    pub pdf: Arc<dyn PdfExporter>,
}

impl std::ops::Deref for MyAppState {
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