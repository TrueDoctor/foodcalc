use fern::colors::{Color, ColoredLevelConfig};
use iced::{Application, Settings};

fn main() -> iced::Result {
    let colors = ColoredLevelConfig::new().debug(Color::Magenta);

    fern::Dispatch::new()
        .chain(std::io::stdout())
        .level_for("foodcalc", log::LevelFilter::Trace)
        .level_for("sqlx", log::LevelFilter::Trace)
        .level(log::LevelFilter::Warn)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}]{} {}",
                // This will color the log level only, not the whole line. Just a touch.
                colors.color(record.level()),
                chrono::Utc::now().format("[%Y-%m-%d %H:%M:%S]"),
                message
            ))
        })
        .apply()
        .unwrap();

    let settings = Settings {
        antialiasing: true,
        text_multithreading: true,
        ..Settings::default()
    };
    foodcalc::app::FoodCalc::run(settings)
}
