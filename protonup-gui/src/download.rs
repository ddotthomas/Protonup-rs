use std::collections::HashMap;

use futures::StreamExt;
use iced::{
    futures::channel::mpsc,
    subscription::{self, Subscription},
};
use libprotonup::github;

use crate::{
    gui::Message,
    utility::{AppInstallWrapper, ReleaseWrapper},
};

pub async fn get_launcher_releases(
    launchers: Vec<AppInstallWrapper>,
) -> Result<HashMap<AppInstallWrapper, Vec<ReleaseWrapper>>, ()> {
    let mut release_map: HashMap<AppInstallWrapper, Vec<ReleaseWrapper>> = HashMap::new();

    let mut future_set = tokio::task::JoinSet::new();

    for launcher in launchers {
        future_set.spawn(return_releases(launcher));
    }

    while let Some(res) = future_set.join_next().await {
        // So many results to deal with
        let release = if let Ok(res) = res {
            if let Ok(release) = res {
                release
            } else {
                return Err(());
            }
        } else {
            return Err(());
        };

        release_map.insert(release.0, release.1);
    }

    Ok(release_map)
}

async fn return_releases(
    launcher: AppInstallWrapper,
) -> Result<(AppInstallWrapper, Vec<ReleaseWrapper>), ()> {
    let releases = if let Ok(releases) =
        github::list_releases(launcher.app_install.get_github_parameters()).await
    {
        releases.into_iter().map(|release| release.into()).collect()
    } else {
        return Err(());
    };

    Ok((launcher, releases))
}

/// State tracker for the download handler iced subscription
enum State {
    Starting,
    Ready(mpsc::Receiver<HandlerMessage>),
}

pub fn handle_downloads() -> Subscription<DownloadInfo> {
    struct Handler;

    subscription::channel(
        std::any::TypeId::of::<Handler>(),
        100,
        |mut output| async move {
            let mut state = State::Starting;
            // channel() takes a Future with a Never return type
            // we loop forever in order to never return
            loop {
                match &mut state {
                    // When the app is starting, set up the mpsc tx and rx channels
                    State::Starting => {
                        // Create the mpsc channels to communicate to the subscription
                        let (mut h_tx, h_rx) = mpsc::channel(100);

                        // Send the transmitter channel to the main/gui
                        let _ = output.try_send(DownloadInfo::Connected(h_tx));

                        // Set the subsciption state to ready with the reciever
                        state = State::Ready(h_rx);
                    }
                    // After the reciever and transmitter channels are ready, start listening for messages on the reciever
                    State::Ready(h_rx) => {
                        // Check if there's any messages from the gui and handle them
                        match h_rx.select_next_some().await {
                            HandlerMessage::Download(data) => { /* TODO */ }
                        }
                    }
                }
            }
        },
    )
}

#[derive(Debug, Clone)]
/// Download info context switch, handled by the gui::Message::DownloadInfo
pub enum DownloadInfo {
    Connected(mpsc::Sender<HandlerMessage>),
}

pub enum HandlerMessage {
    Download(/* TODO DownloadData */ ()),
}
