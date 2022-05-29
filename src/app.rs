use std::env;

use log::{debug, error, warn};
use sqlx::postgres::types::PgMoney;
use sqlx::types::BigDecimal;
use sqlx::PgPool;

use self::actions::Actions;
use self::state::{AppState, PopUp};
use crate::app::actions::Action;
use crate::inputs::key::Key;
use crate::io::IoEvent;

pub mod actions;
pub mod db;
#[cfg(feature = "scraping")]
pub mod scraping;
pub mod state;
pub mod ui;

static PRICE_PLACEHOLDER: PgMoney = PgMoney(-100i64);

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

/// The main application, containing the state
pub struct App {
    /// We could dispatch an IO event
    io_tx: tokio::sync::mpsc::Sender<IoEvent>,
    /// Contextual actions
    actions: Actions,
    /// State
    is_loading: bool,
    state: AppState,
    database: db::FoodBase,
}

impl App {
    pub async fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>) -> eyre::Result<Self> {
        let actions = vec![Action::Quit].into();
        let is_loading = false;
        let state = AppState::default();

        dotenv::dotenv().ok();
        let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;
        let database = db::FoodBase::new(pool);

        Ok(Self {
            io_tx,
            actions,
            is_loading,
            state,
            database,
        })
    }

    /// Handle a user action
    pub async fn do_action(&mut self, key: Key) -> AppReturn {
        if self.state.input().is_some() {
            match key {
                Key::Enter => match self.state.popup() {
                    None => self.state.add_ingredient_source_url(),
                    Some(PopUp::AddSourceUrl { .. }) => {
                        self.state.add_ingredient_source_weight();
                    }
                    Some(PopUp::AddSourceWeight {
                        ingredient,
                        url,
                        weight,
                    }) => {
                        if let Some((num, unit)) = db::parse_package_size(weight) {
                            let add_source_event = IoEvent::AddIngredientSource {
                                ingredient_id: ingredient.ingredient_id,
                                store_id: db::METRO,
                                url: url.clone(),
                                weight: num,
                                unit,
                                price: PRICE_PLACEHOLDER,
                            };
                            self.dispatch(add_source_event).await;
                        }
                        self.state.close_popup();
                    }
                    Some(_) => {
                        self.state.close_popup();
                    }
                },
                Key::Char(c) => {
                    if let Some(input) = self.state.input() {
                        input.push(c)
                    }
                }
                Key::Backspace => {
                    if let Some(input) = self.state.input() {
                        input.pop();
                    }
                }
                Key::Esc => {
                    self.state.close_popup();
                }
                _ => {}
            };
            AppReturn::Continue
        } else if let Some(action) = self.actions.find(key) {
            debug!("Run action [{:?}]", action);
            match action {
                Action::Quit => AppReturn::Exit,
                Action::Refresh => {
                    // Refresh is an I/O action, we dispatch on the IO channel that's run on another thread
                    self.dispatch(IoEvent::UpdateData).await;
                    AppReturn::Continue
                }
                Action::MoveDown => {
                    self.state.next_item();
                    AppReturn::Continue
                }
                Action::MoveUp => {
                    self.state.previous_item();
                    AppReturn::Continue
                }
                Action::AddSource => {
                    self.state.add_ingredient_source_url();
                    AppReturn::Continue
                }
                Action::FetchMetroPrice => {
                    let ingredient = self.state.ingredient();
                    let ingredient_id = ingredient.map(|ingredient| ingredient.ingredient_id);
                    self.dispatch(IoEvent::FetchMetroPrice { ingredient_id })
                        .await;
                    AppReturn::Continue
                }
            }
        } else {
            warn!("No action accociated to {}", key);
            AppReturn::Continue
        }
    }

    /// We could update the app or dispatch event on tick
    pub async fn update_on_tick(&mut self) -> AppReturn {
        // here we just increment a counter
        AppReturn::Continue
    }

    /// We could update the app or dispatch event on tick
    pub async fn update_data(&mut self) -> AppReturn {
        // here we just increment a counter
        self.state.update(&self.database).await;
        AppReturn::Continue
    }

    pub async fn add_ingredient_source(
        &mut self,
        ingredient_id: i32,
        store_id: i32,
        weight: BigDecimal,
        price: PgMoney,
        url: String,
    ) {
        log::debug!("Ingredients");
        match self
            .database
            .add_ingredient_source(ingredient_id, store_id, weight, price, Some(url), 0)
            .await
        {
            Ok(id) => {
                self.state.next_item();
                log::debug!("Added source for ingredient {id}")
            }
            Err(error) => {
                log::error!("failed to add ingredient source to database, {error:?}")
            }
        }
    }

    pub async fn fetch_ingredient_price(&mut self, ingredient_id: Option<i32>) {
        match self.database.fetch_metro_prices(ingredient_id).await {
            Ok(id) => {
                self.state.next_item();
                log::debug!("Updated price for {id:?} ingredients")
            }
            Err(error) => {
                log::error!("failed to updete metro prices, {error:?}")
            }
        }
    }

    /// Send a network event to the IO thread
    pub async fn dispatch(&mut self, action: IoEvent) {
        // `is_loading` will be set to false again after the async action has finished in io/handler.rs
        self.is_loading = true;
        if let Err(e) = self.io_tx.send(action).await {
            self.is_loading = false;
            error!("Error from dispatch {}", e);
        };
    }

    pub fn actions(&self) -> &Actions {
        &self.actions
    }
    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    pub fn initialized(&mut self) {
        // Update contextual actions
        self.actions = vec![
            Action::Quit,
            Action::Refresh,
            Action::MoveDown,
            Action::MoveUp,
            Action::AddSource,
            #[cfg(feature = "scraping")]
            Action::FetchMetroPrice,
        ]
        .into();
        self.state = AppState::initialized()
    }

    pub fn loaded(&mut self) {
        self.is_loading = false;
    }

    pub fn updated_data(&mut self) {
        //self.state.incr_sleep();
    }
}
