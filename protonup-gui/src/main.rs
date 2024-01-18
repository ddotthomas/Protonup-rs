use iced::{window, Application, Settings};

mod gui;
mod utility;
use gui::Gui;

pub fn main() -> iced::Result {
    Gui::run(Settings {
        window: window::Settings {
            size: (800, 450),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}
