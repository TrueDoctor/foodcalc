pub mod model_wrapper;
pub mod theme;

use std::sync::Arc;

use iced::alignment::{Horizontal, Vertical};
use iced::{Application, Column, Command, Container, Element, Font, Length, Sandbox, Settings, Text};
use iced_aw::{TabLabel, Tabs};

mod ingredient_tab;

use self::ingredient_tab::{IngredientTab, IngredientTabMessage};
use crate::app::Message;
use crate::db::FoodBase;

//mod recipes;
//use recipes::{RecipeMessage, RecipeTab};

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
    Edit,
    CogAlt,
    Delete,
}

impl From<Icon> for char {
    fn from(icon: Icon) -> Self {
        match icon {
            Icon::Apple => '\u{eb3b}',
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

#[derive(Clone, Debug)]
pub struct TabBarExample {
    active_tab: usize,
    database: Arc<FoodBase>,
    ingredient_tab: IngredientTab,
    settings_tab: SettingsTab,
}

#[derive(Clone, Debug)]
pub enum TabMessage {
    TabSelected(usize),
    IngredientTab(IngredientTabMessage),
    Settings(SettingsMessage),
}

impl TabBarExample {
    pub fn new(database: Arc<FoodBase>) -> (Self, Command<TabMessage>) {
        let (ingredient_tab, ingredient_command) = IngredientTab::new(database.clone());

        let tab_bar = TabBarExample {
            active_tab: 0,
            database,
            ingredient_tab,
            settings_tab: SettingsTab::new(),
        };
        (tab_bar, Command::batch([ingredient_command].into_iter()))
    }

    fn title(&self) -> String {
        String::from("TabBar Example")
    }

    pub fn update(&mut self, message: TabMessage) -> Command<TabMessage> {
        match message {
            TabMessage::TabSelected(selected) => {
                self.active_tab = selected;
                Command::none()
            },
            TabMessage::IngredientTab(message) => self.ingredient_tab.update(message),
            TabMessage::Settings(message) => self.settings_tab.update(message),
        }
    }

    pub fn view(&mut self) -> Element<'_, Message> {
        let position = self.settings_tab.settings().tab_bar_position.unwrap_or_default();
        let theme = self.settings_tab.settings().tab_bar_theme.unwrap_or_default();

        let element: Element<'_, TabMessage> = Tabs::new(self.active_tab, TabMessage::TabSelected)
            .push(self.ingredient_tab.tab_label(), self.ingredient_tab.view())
            .push(self.settings_tab.tab_label(), self.settings_tab.view())
            .tab_bar_style(theme)
            .icon_font(ICON_FONT)
            .tab_bar_position(match position {
                TabBarPosition::Top => iced_aw::TabBarPosition::Top,
                TabBarPosition::Bottom => iced_aw::TabBarPosition::Bottom,
            })
            .into();
        element.map(Message::MainMessage)
    }
}

trait Tab {
    type Message;

    fn title(&self) -> String;

    fn tab_label(&self) -> TabLabel;

    fn view(&mut self) -> Element<'_, Self::Message> {
        let title = Text::new(self.title())
            .width(Length::Fill)
            .size(HEADER_SIZE)
            .color([0.5, 0.5, 0.5])
            .horizontal_alignment(iced::alignment::Horizontal::Center);

        let column = Column::new().spacing(20).push(title).push(self.content());

        Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(TAB_PADDING)
            .into()
    }

    fn content(&mut self) -> Element<'_, Self::Message>;
}
