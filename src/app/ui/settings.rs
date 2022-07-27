use iced::{Column, Command, Container, Element, Radio, Text};
use iced_aw::TabLabel;

use crate::app::ui::theme::Theme;
use crate::app::ui::{Icon, Tab, TabMessage};

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
            SettingsMessage::ThemeSelected(theme) => self.settings().tab_bar_theme = Some(theme),
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

    fn content(&mut self, theme: impl iced_aw::modal::StyleSheet + 'static) -> Element<'_, Self::Message> {
        let content: Element<'_, SettingsMessage> = Container::new(
            Column::new()
                .spacing(20)
                .push(Text::new("TabBar position:").size(20))
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
                            .size(16),
                        )
                    },
                ))
                .push(Text::new("TabBar color:").size(20))
                .push(
                    Theme::ALL
                        .iter()
                        .cloned()
                        .fold(Column::new().padding(10).spacing(10), |column, theme| {
                            column.push(
                                Radio::new(
                                    theme,
                                    theme,
                                    self.settings().tab_bar_theme,
                                    SettingsMessage::ThemeSelected,
                                )
                                .size(16),
                            )
                        }),
                ),
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
