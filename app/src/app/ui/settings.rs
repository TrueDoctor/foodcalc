use std::sync::RwLock;

use iced::{Column, Command, Container, Element, Radio, Text};
use iced_aw::TabLabel;

use crate::app::ui::theme::Theme;
use crate::app::ui::{Icon, Tab, TabMessage};

lazy_static::lazy_static! {
    pub static  ref CLOSE_ON_SAVE: RwLock<bool> = RwLock::new(true);
}

pub fn close_on_save() -> bool {
    CLOSE_ON_SAVE.read().as_deref().cloned().unwrap_or_default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabBarPosition {
    Top,
    Bottom,
}

impl TabBarPosition {
    pub const ALL: [TabBarPosition; 2] = [TabBarPosition::Top, TabBarPosition::Bottom];
}

impl From<TabBarPosition> for String {
    fn from(position: TabBarPosition) -> Self {
        String::from(match position {
            TabBarPosition::Top => "Top",
            TabBarPosition::Bottom => "Bottom",
        })
    }
}

impl Default for TabBarPosition {
    fn default() -> Self {
        TabBarPosition::Top
    }
}

//#[derive(Debug, Clone, Copy, PartialEq, Eq)]

#[derive(Debug, Clone)]
pub struct TabSettings {
    pub tab_bar_position: Option<TabBarPosition>,
    pub tab_bar_theme: Option<Theme>,
}

impl TabSettings {
    pub fn new() -> Self {
        TabSettings {
            tab_bar_position: Some(TabBarPosition::Top),
            tab_bar_theme: Some(Theme::default()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    PositionSelected(TabBarPosition),
    ThemeSelected(Theme),
    CloseOnSaveUpdated(bool),
}

#[derive(Debug, Clone)]
pub struct SettingsTab {
    settings: TabSettings,
}

impl SettingsTab {
    pub fn new() -> Self {
        SettingsTab {
            settings: TabSettings::new(),
        }
    }

    pub fn update(&mut self, message: SettingsMessage) -> Command<TabMessage> {
        match message {
            SettingsMessage::PositionSelected(position) => self.settings().tab_bar_position = Some(position),
            SettingsMessage::ThemeSelected(theme) => {
                self.settings().tab_bar_theme = Some(theme);
                match super::theme::THEME.write() {
                    Ok(mut t) => *t = theme,
                    Err(_) => log::error!("error setting theme"),
                }
            },
            SettingsMessage::CloseOnSaveUpdated(close_on_save) => {
                *CLOSE_ON_SAVE.write().unwrap() = close_on_save;
            },
        };
        Command::none()
    }

    pub fn settings(&mut self) -> &mut TabSettings {
        &mut self.settings
    }
}

impl Tab for SettingsTab {
    type Message = TabMessage;

    fn title(&self) -> String {
        String::from("Settings")
    }

    fn tab_label(&self) -> TabLabel {
        //TabLabel::Text(self.title())
        TabLabel::IconText(Icon::CogAlt.into(), self.title())
    }

    fn content(&mut self) -> Element<'_, Self::Message> {
        let theme = crate::theme();
        let content: Element<'_, SettingsMessage> = Container::new(
            Column::new()
                .spacing(20)
                .push(Text::new("TabBar position:").size(20).color(theme.foreground()))
                .push(TabBarPosition::ALL.iter().cloned().fold(
                    Column::new().padding(10).spacing(10),
                    |column, position| {
                        column.push(
                            Radio::new(
                                position,
                                position,
                                self.settings().tab_bar_position,
                                SettingsMessage::PositionSelected,
                            )
                            .style(theme)
                            .size(16),
                        )
                    },
                ))
                .push(Text::new("TabBar color:").size(20).color(theme.foreground()))
                .push(Theme::ALL.iter().cloned().fold(
                    Column::new().padding(10).spacing(10),
                    |column, selected_theme| {
                        column.push(
                            Radio::new(
                                selected_theme,
                                selected_theme,
                                Some(theme),
                                SettingsMessage::ThemeSelected,
                            )
                            .style(theme)
                            .size(16),
                        )
                    },
                ))
                .push(Text::new("Close on save:").size(20).color(theme.foreground()))
                .push(iced::widget::Checkbox::new(
                    close_on_save(),
                    "Close on save",
                    SettingsMessage::CloseOnSaveUpdated,
                )),
        )
        .into();
        let element: Element<'_, SettingsMessage> = Container::new(content)
            .height(iced::Length::Fill)
            .width(iced::Length::Fill)
            .center_x()
            .into();

        element.map(TabMessage::Settings)
    }
}
