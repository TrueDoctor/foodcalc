use std::env;

use log::{debug, error, warn};
use num::Num;
//use scraper::{Html, Selector};
use sqlx::postgres::types::PgMoney;
use sqlx::types::BigDecimal;
use sqlx::PgPool;
use tokio::io::split;

use self::actions::Actions;
use self::state::{AppState, PopUp};
use crate::app::actions::Action;
use crate::inputs::key::Key;
use crate::io::IoEvent;

pub mod actions;
pub mod db;
pub mod state;
pub mod ui;

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

enum InputMode {
    Normal,
    Editing,
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
    input_mode: InputMode,
    database: db::FoodBase,
}
/*async fn fetch_metro_price_reqwest(url: &str) -> PgMoney {
    let mut resp = reqwest::get(url).await.unwrap();
    assert!(resp.status().is_success());

    let body = resp.text().await.unwrap();
    // parses string of HTML as a document
    let fragment = Html::parse_document(&body);
    // parses based on a CSS selector
    let prices =
        Selector::parse("div+ div .mfcss_article-detail--price-breakdown span span").unwrap();

    // iterate over elements matching our selector
    panic!(
        "at the disco {:?}",
        fragment.select(&prices).into_iter().size_hint()
    );
    for story in fragment.select(&prices) {
        // grab the headline text and place into a vector
        let story_txt = story.text().collect::<Vec<_>>();
        panic!("{:?}", story_txt);
    }

    PgMoney::from_bigdecimal(BigDecimal::new(1.into(), 1), 2).unwrap()
}*/

#[cfg(feature = "scraping")]
fn fetch_metro_price_python(url: &str) -> PgMoney {
    use inline_python::{python, Context};

    let c = Context::new();
    c.run(python! {
        from selenium import webdriver
        from selenium.webdriver.common.by import By
        from selenium.webdriver.common.keys import Keys
        import time

        browser = webdriver.Firefox()
    });

    c.run(python! {
        browser.get("https://produkte.metro.de/shop/pv/BTY-Z112/0032/0021/Champignon-weiss-fein-3kg")

    });
    std::thread::sleep_ms(5000);

    c.run(python! {
        elem = browser.find_element_by_css_selector(".mfcss_article-detail--price-breakdown")
        elem = elem.text
    });

    let value = c.get::<String>("elem");
    println!("value: {}", value);
    let price = value.split_whitespace().find(|s| s.contains(',')).unwrap();
    use num::Num;

    PgMoney::from_bigdecimal(BigDecimal::from_str_radix(price, 10).unwrap(), 2).unwrap()
}

impl App {
    pub async fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>) -> eyre::Result<Self> {
        let actions = vec![Action::Quit].into();
        let is_loading = false;
        let state = AppState::default();

        dotenv::dotenv().ok();
        let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;
        let database = db::FoodBase::new(pool);
        let input_mode = InputMode::Normal;

        Ok(Self {
            io_tx,
            actions,
            is_loading,
            input_mode,
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
                        {
                            let (num, unit) = db::parse_package_size(weight).unwrap();
                            self.dispatch(IoEvent::AddIngredientSource {
                                ingredient_id: ingredient.ingredient_id,
                                store_id: 0,
                                url: url.clone(),
                                weight: num,
                                unit,
                                price: PgMoney::from_bigdecimal(
                                    BigDecimal::from_str_radix("-1", 10).unwrap(),
                                    2,
                                )
                                .unwrap(),
                            })
                            .await;
                        }
                        self.state.close_popup();
                        self.input_mode = InputMode::Normal;
                    }
                    Some(_) => {
                        self.state.close_popup();
                        self.input_mode = InputMode::Normal;
                    }
                },
                Key::Char(c) => {
                    self.state.input().unwrap().push(c);
                }
                Key::Backspace => {
                    self.state.input().unwrap().pop();
                }
                Key::Esc => {
                    self.input_mode = InputMode::Normal;
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
                    // Sleep is an I/O action, we dispatch on the IO channel that's run on another thread
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
                    self.input_mode = InputMode::Editing;
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
