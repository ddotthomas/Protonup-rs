use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc,
};

use futures::StreamExt;
use iced::{
    futures::channel::mpsc,
    subscription::{self, Subscription},
};
use libprotonup::{files, github};

use crate::utility::{self, AppInstallWrapper, ReleaseWrapper};

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
        let (launcher, releases) = if let Ok(res) = res {
            if let Ok(release) = res {
                release
            } else {
                return Err(());
            }
        } else {
            return Err(());
        };

        release_map.insert(launcher, releases);
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

pub fn handle_downloads() -> Subscription<DownloadThreadMessage> {
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
                        let (h_tx, h_rx) = mpsc::channel(100);

                        // Send the transmitter channel to the main/gui
                        let _ = output.try_send(DownloadThreadMessage::Ready(h_tx));

                        // Set the subsciption state to ready with the reciever
                        state = State::Ready(h_rx);
                    }
                    // After the reciever and transmitter channels are ready, start listening for messages on the reciever
                    State::Ready(h_rx) => {
                        // Check if there's any messages from the gui and handle them
                        match h_rx.select_next_some().await {
                            HandlerMessage::Download(download_info) => {
                                for (id, download) in download_info.requested_downloads {
                                    // Create the progress trackers for the GUI progress bar and download function
                                    let (progress, download_done) =
                                        files::create_progress_trackers();
                                    let _ = output.try_send(DownloadThreadMessage::Trackers(
                                        id,
                                        (progress.clone(), download_done.clone()),
                                    ));

                                    let install_path = std::path::Path::new(
                                        download_info
                                            .selected_app
                                            .app_install
                                            .default_install_dir(),
                                    );

                                    // Read the sent download info and start the requested downloads
                                    match files::download_file_progress(
                                        download.download_url,
                                        download.size,
                                        install_path,
                                        progress,
                                        download_done,
                                    )
                                    .await
                                    {
                                        Ok(_) => {
                                            todo!()
                                        }
                                        Err(_) => {
                                            todo!()
                                        }
                                    }
                                }
                            }
                        };
                    }
                }
            }
        },
    )
}

#[derive(Debug, Clone)]
/// Download thread info organizer, handled by the gui::Message::DownloadInfo
pub enum DownloadThreadMessage {
    Ready(mpsc::Sender<HandlerMessage>),
    Trackers(usize, (Arc<AtomicUsize>, Arc<AtomicBool>)),
}

/// Messages to send to the download thread
pub enum HandlerMessage {
    Download(DownloadInfo),
}

/// All the information needed by the download thread to start the download(s)
pub struct DownloadInfo {
    selected_app: AppInstallWrapper,
    requested_downloads: Vec<(usize, github::Download)>,
}

/// Quick download the currently selected app's most recent wine version
pub fn quick_update(
    selected_app: &AppInstallWrapper,
    release_data: &Option<HashMap<utility::AppInstallWrapper, Vec<utility::ReleaseWrapper>>>,
    download_handler_tx: &mut Option<mpsc::Sender<HandlerMessage>>,
) {
    // Check that the download handler and release data are ready
    if let Some(h_tx) = download_handler_tx {
        if let Some(release_map) = release_data {
            // Get the GitHub download list for the currently selected app
            if let Some(release_list) = release_map.get(selected_app) {
                // Grab the download info for the most recent version
                let download_data = DownloadInfo {
                    selected_app: *selected_app,
                    requested_downloads: vec![(1, release_list[0].get_download_info())],
                };

                // Send that in the Sender channel to the download handler thread
                let _ = h_tx.try_send(HandlerMessage::Download(download_data));
            }
        }
    }
}
