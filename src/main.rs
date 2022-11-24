use fern::colors::{Color, ColoredLevelConfig};
use iced::{Application, Settings};

fn main() -> iced::Result {
    let settings = Settings {
        antialiasing: true,
        text_multithreading: true,
        ..Settings::default()
    };
    foodcalc::app::FoodCalc::run(settings)
}
