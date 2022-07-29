pub mod style;
pub mod theme;

use std::sync::Arc;

use iced::alignment::{Horizontal, Vertical};
use iced::{Column, Command, Container, Element, Font, Length, Text};
use iced_aw::{TabLabel, Tabs};

mod ingredient_tab;
use self::ingredient_tab::{IngredientTab, IngredientTabMessage};

mod recipe_tab;
use self::recipe_tab::{RecipeTab, RecipeTabMessage};

use crate::app::Message;
use crate::db::FoodBase;

//mod event_meals;
//use counter::{CounterMessage, CounterTab};

mod settings;
use settings::{SettingsMessage, SettingsTab, TabBarPosition};

const HEADER_SIZE: u16 = 100;
const TAB_PADDING: u16 = 16;

const ICON_FONT: Font = iced::Font::External {
    name: "Icons",
    bytes: include_bytes!("../../fonts/icons.ttf"),
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Apple,
    Burger,
    Edit,
    CogAlt,
    Delete,
}

impl From<Icon> for char {
    fn from(icon: Icon) -> Self {
        match icon {
            Icon::Apple => '\u{eb3b}',
            Icon::Burger => '\u{eb49}',
            Icon::CogAlt => '\u{ef3b}',
            Icon::Edit => '\u{ec55}',
            Icon::Delete => '\u{ec53}',
        }
    }
}

impl Icon {
    pub fn text(&self) -> Text {
        let c: char = (*self).into();
        Text::new(c.to_string())
            .font(ICON_FONT)
            .width(Length::Units(20))
            .height(Length::Shrink)
            .horizontal_alignment(iced::alignment::Horizontal::Center)
            .size(20)
    }
}

#[derive(Debug)]
pub struct TabBarExample {
    active_tab: usize,
    _database: Arc<FoodBase>,
    ingredient_tab: IngredientTab,
    recipe_tab: RecipeTab,
    settings_tab: SettingsTab,
}

#[derive(Clone, Debug)]
pub enum TabMessage {
    TabSelected(usize),
    IngredientTab(Box<IngredientTabMessage>),
    RecipeTab(Box<RecipeTabMessage>),
    Settings(SettingsMessage),
}

impl TabBarExample {
    pub fn new(database: Arc<FoodBase>) -> (Self, Command<TabMessage>) {
        let (ingredient_tab, ingredient_command) = IngredientTab::new(database.clone());
        let (recipe_tab, recipe_command) = RecipeTab::new(database.clone());

        let tab_bar = TabBarExample {
            active_tab: 0,
            _database: database,
            ingredient_tab,
            recipe_tab,
            settings_tab: SettingsTab::new(),
        };
        (
            tab_bar,
            Command::batch([ingredient_command, recipe_command].into_iter()),
        )
    }

    pub fn update(&mut self, message: TabMessage) -> Command<TabMessage> {
        match message {
            TabMessage::TabSelected(selected) => {
                self.active_tab = selected;
                Command::none()
            },
            TabMessage::IngredientTab(message) => self.ingredient_tab.update(*message),
            TabMessage::RecipeTab(message) => self.recipe_tab.update(*message),
            TabMessage::Settings(message) => self.settings_tab.update(message),
        }
    }

    pub fn view(&mut self) -> Element<'_, Message> {
        let position = self.settings_tab.settings().tab_bar_position.unwrap_or_default();
        let theme = crate::theme();

        let element: Element<'_, TabMessage> = Tabs::new(self.active_tab, TabMessage::TabSelected)
            .push(self.ingredient_tab.tab_label(), self.ingredient_tab.view())
            .push(self.recipe_tab.tab_label(), self.recipe_tab.view())
            .push(self.settings_tab.tab_label(), self.settings_tab.view())
            .tab_bar_style(theme)
            .icon_font(ICON_FONT)
            .tab_bar_position(match position {
                TabBarPosition::Top => iced_aw::TabBarPosition::Top,
                TabBarPosition::Bottom => iced_aw::TabBarPosition::Bottom,
            })
            .into();
        //let element = element.explain(iced::Color::from_rgb(1., 0., 0.));
        element.map(Message::MainMessage)
    }
}

trait Tab {
    type Message:'static;

    fn title(&self) -> String;

    fn tab_label(&self) -> TabLabel;

    fn view(&mut self) -> Element<'_, Self::Message> {

        let column = Column::new().spacing(20).push(self.content());

        let element: iced::Element<'_, Self::Message> = Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(TAB_PADDING)
            .into();
            element
            //element.explain(iced::Color::from_rgb(1.0,0.0,0.0))
    }

    fn content(&mut self) -> Element<'_, Self::Message>;
}
