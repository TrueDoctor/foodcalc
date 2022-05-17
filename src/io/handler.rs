use std::sync::Arc;
use std::time::Duration;

use eyre::Result;
use log::{error, info};

use super::IoEvent;
use crate::app::App;

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

    /// Just take a little break
    async fn do_update_data(&mut self) -> Result<()> {
        info!("Fetching ingredients");
        self.app.lock().await.update_data().await;
        // Notify the app for having slept
        let mut app = self.app.lock().await;
        app.updated_data();

        Ok(())
    }
}
