use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc,
};

use iced::executor;
use iced::futures::channel::mpsc::Sender;
use iced::widget::{button, column, container, pick_list, row, scrollable};
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

use crate::download::{DownloadThreadMessage, HandlerMessage};
use crate::{download, utility};

#[derive(Debug)]
pub struct Gui {
    selected_launcher: utility::AppInstallWrapper,
    launchers: Vec<utility::AppInstallWrapper>,
    release_data: Option<HashMap<utility::AppInstallWrapper, Vec<utility::ReleaseWrapper>>>,
    download_status: DownloadStatus,
}

#[derive(Default, Debug)]
pub struct DownloadStatus {
    /// Channel to send messages to the download threads.
    download_handler_tx: Option<Sender<HandlerMessage>>,
    download_trackers: Vec<(usize, (Arc<AtomicUsize>, Arc<AtomicBool>))>,
}

#[derive(Debug, Clone)]
pub enum Message {
    QuickUpdate,
    LauncherSelected(utility::AppInstallWrapper),
    AddReleases(Result<HashMap<utility::AppInstallWrapper, Vec<utility::ReleaseWrapper>>, ()>),
    /// Toggle the release being downloaded or not
    SelectVersion(utility::AppInstallWrapper, utility::ReleaseWrapper, bool),
    /// Wrapper around any messages or events from the download thread/subscription
    DownloadInfo(download::DownloadThreadMessage),
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
                    installed_apps[0]
                } else {
                    // If no installed apps were found, default to Steam
                    apps::AppInstallations::Steam.into()
                },
                launchers: installed_apps.clone(),
                release_data: None,
                download_status: Default::default(),
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
            Message::QuickUpdate => {
                download::quick_update(
                    &self.selected_launcher,
                    &self.release_data,
                    &mut self.download_status.download_handler_tx,
                );
            }
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
            Message::DownloadInfo(info) => match info {
                DownloadThreadMessage::Ready(h_tx) => {
                    self.download_status.download_handler_tx = Some(h_tx);
                }
                DownloadThreadMessage::Trackers(id, (progress, download_done)) => {
                    self.download_status.download_trackers.push((id, (progress, download_done)));
                }
            },
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        download::handle_downloads().map(Message::DownloadInfo)
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
            scrollable(column(helpers::download_list(&self)).padding(5))
                .width(Length::FillPortion(3)),
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
