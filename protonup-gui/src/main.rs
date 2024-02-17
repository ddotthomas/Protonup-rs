use iced::{window, Application, Settings};

mod download;
mod gui;
mod utility;
use gui::Gui;

pub fn main() -> iced::Result {
    Gui::run(Settings {
        window: window::Settings {
            size: [600, 400].into(),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}
