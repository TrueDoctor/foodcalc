use std::str::FromStr;

use chrono::Duration;
use sqlx::postgres::types::PgInterval;
use sqlx::types::time::PrimitiveDateTime;
use sqlx::types::BigDecimal;

#[derive(Debug, Clone)]
pub struct InputState<T> {
    pub state: iced::text_input::State,
    pub value: String,
    pub value_type: Option<T>,
}
impl<T> Default for InputState<T> {
    fn default() -> Self {
        Self {
            state: Default::default(),
            value: String::new(),
            value_type: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct OptionString(Option<String>);

impl FromStr for OptionString {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self(None))
        } else {
            Ok(Self(Some(s.to_string())))
        }
    }
}

#[derive(Debug, Clone)]
pub struct DateInput(pub PrimitiveDateTime);

impl FromStr for DateInput {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let date = PrimitiveDateTime::parse(s, "%Y-%m-%d %H:%M")?;
        Ok(Self(date))
    }
}

#[derive(Debug, Clone)]
pub struct DurationInput(pub PgInterval);

impl FromStr for DurationInput {
    type Err = eyre::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let duration = parse_duration::parse(s)?;
        let interval = PgInterval::try_from(duration);
        let interval = interval.map_err(|_| eyre::eyre!("Invalid duration"))?;
        Ok(Self(interval))
    }
}

impl Into<String> for DurationInput {
    fn into(self) -> String {
        let duration = chrono::Duration::microseconds(self.0.microseconds);
        format!(
            "{}:{}:{}",
            duration.num_hours(),
            duration.num_minutes(),
            duration.num_seconds()
        )
    }
}

impl<T> InputState<T>
where
    T: FromStr,
{
    pub fn new(value: impl Into<String>) -> Self {
        let mut state = Self::default();
        state.update(value);
        state
    }
    pub fn parsed_value(&self) -> &Option<T> {
        &self.value_type
    }

    pub fn valid(&self) -> bool {
        self.value_type.is_some()
    }

    pub fn input(&self) -> &str {
        self.value.as_str()
    }

    pub fn update(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.value_type = self.input().parse().ok();
    }

    pub fn text_color(&self) -> super::style::TextInput {
        if self.valid() {
            super::style::TextInput::Normal
        } else {
            super::style::TextInput::Error
        }
    }
}
