use std::collections::HashMap;

use iced::executor;
use iced::widget::{button, column, container, pick_list, row};
use iced::{
    Application,
    Command,
    Element,
    Length,
    Subscription,
    Theme,
    // Background,
    // Color,
};
use libprotonup::apps;
mod helpers;

use crate::{download, utility};

//use std::{cmp, path::PathBuf};

#[derive(Debug)]
pub struct Gui {
    selected_launcher: utility::AppInstallWrapper,
    launchers: Vec<utility::AppInstallWrapper>,
    release_data: Option<HashMap<utility::AppInstallWrapper, Vec<utility::ReleaseWrapper>>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    QuickUpdate,
    LauncherSelected(utility::AppInstallWrapper),
    AddReleases(Result<HashMap<utility::AppInstallWrapper, Vec<utility::ReleaseWrapper>>, ()>),
    /// Toggle the release being downloaded or not
    SelectVersion(utility::AppInstallWrapper, utility::ReleaseWrapper, bool),
}

impl Application for Gui {
    type Message = Message;
    type Theme = Theme;
    // Using the tokio async executor
    type Executor = executor::Default;
    type Flags = ();

    /// Sets up the Gui and its needed variables
    fn new(_flags: ()) -> (Self, Command<Message>) {
        // Find the currently installed apps to show in the GUI dropdown
        let installed_apps = utility::list_installed_apps();
        (
            Self {
                // If there were any apps found, 
                // use the first one as the currently selected
                selected_launcher: if installed_apps.len() > 0 {
                    installed_apps[0].clone()
                } else {
                    // If no installed apps were found, default to Steam
                    apps::AppInstallations::Steam.into()
                },
                launchers: installed_apps.clone(),
                release_data: None,
            },
            Command::perform(
                download::get_launcher_releases(installed_apps),
                Message::AddReleases,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("Protonup-rs")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            // TODO
            Message::QuickUpdate => { },
            Message::LauncherSelected(app) => {
                self.selected_launcher = app;
            }
            Message::AddReleases(releases) => match releases {
                Ok(releases) => {
                    self.release_data = Some(releases);
                }
                Err(()) => { /* TODO display an Error message of some kind in the GUI */ }
            },
            Message::SelectVersion(app, release, selection) => {
                if let Some(release_map) = &mut self.release_data {
                    if let Some(releases) = release_map.get_mut(&app) {
                        if let Some(rel) = releases.iter_mut().find(|rel| **rel == release) {
                            rel.selected = selection;
                        }
                    }
                }
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn view(&self) -> Element<Message> {
        let controls: Element<Message> = column(vec![button("TODO: Quick Update")
            .on_press(Message::QuickUpdate)
            .width(Length::Fill)
            .into()])
        .width(Length::FillPortion(1))
        .padding(5)
        .into();

        let list = Element::from(
            column(helpers::download_list(&self))
                .width(Length::FillPortion(3))
                .padding(5),
        );

        let content = column(vec![
            container(
                pick_list(
                    self.launchers.clone(),
                    Some(self.selected_launcher.clone()),
                    Message::LauncherSelected,
                )
                .width(Length::Fill),
            )
            .height(Length::Fixed(40.))
            .width(Length::Fill)
            .into(),
            container(row(vec![controls, list]))
                .height(Length::Fill)
                .into(),
        ]);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(10)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
