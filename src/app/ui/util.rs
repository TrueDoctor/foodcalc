#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub state: iced::text_input::State,
    pub valid: bool,
    pub value: String,
}
