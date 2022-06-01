use std::sync::Arc;

use eyre::Result;
use log::{error, info};
use sqlx::postgres::types::PgMoney;
use sqlx::types::BigDecimal;

use super::IoEvent;
use crate::app::{database, App};

/// In the IO thread, we handle IO event without blocking the UI thread
pub struct IoAsyncHandler {
    app: Arc<tokio::sync::Mutex<App>>,
}

impl IoAsyncHandler {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    /// We could be async here
    pub async fn handle_io_event(&mut self, io_event: IoEvent) {
        let result = match io_event {
            IoEvent::Initialize => self.do_initialize().await,
            IoEvent::UpdateData => self.do_update_data().await,
            IoEvent::AddIngredientSource {
                ingredient_id,
                store_id,
                url,
                price,
                weight,
                unit,
            } => {
                self.do_add_ingredient_source(ingredient_id, store_id, url, price, weight, unit)
                    .await
            },
            IoEvent::FetchMetroPrice { ingredient_id } => self.do_fetch_metro_price(ingredient_id).await,
        };

        if let Err(err) = result {
            error!("Oops, something wrong happen: {:?}", err);
        }

        let mut app = self.app.lock().await;
        app.loaded();
    }

    /// We use dummy implementation here, just wait 1s
    async fn do_initialize(&mut self) -> Result<()> {
        info!("🚀 Initialize the application");
        {
            let mut app = self.app.lock().await;
            //tokio::time::sleep(Duration::from_secs(1)).await;
            app.initialized(); // we could update the app state
        }
        self.do_update_data().await?;
        info!("👍 Application initialized");

        Ok(())
    }

    /// Update the current list view with data from the database
    async fn do_update_data(&mut self) -> Result<()> {
        info!("Fetching ingredients");
        self.app.lock().await.update_data().await;
        // Notify the app for having slept
        let mut app = self.app.lock().await;
        app.updated_data();

        Ok(())
    }

    /// Update the price for the given ingredient
    async fn do_fetch_metro_price(&mut self, ingredient_id: Option<i32>) -> Result<()> {
        info!("Fetching ingredient price for {ingredient_id:?}");

        match database().fetch_metro_prices(ingredient_id).await {
            Ok(id) => {
                self.app.lock().await.state.next_item();
                log::debug!("Updated price for {id:?} ingredients")
            },
            Err(error) => {
                log::error!("failed to updete metro prices, {error:?}")
            },
        };

        Ok(())
    }

    /// Add source for given ingredient to the database
    async fn do_add_ingredient_source(
        &mut self,
        ingredient_id: i32,
        store_id: i32,
        url: String,
        price: PgMoney,
        weight: BigDecimal,
        unit: i32,
    ) -> Result<()> {
        info!("Adding ingredient source");

        self.app
            .lock()
            .await
            .add_ingredient_source(ingredient_id, store_id, weight, price, url, unit)
            .await;
        // Notify the app for having slept
        //let mut app = self.app.lock().await;
        //app.updated_data();

        Ok(())
    }
}
