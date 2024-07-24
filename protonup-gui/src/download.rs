use std::path::Path;

use futures::StreamExt;
use iced::{
    futures::channel::mpsc,
    subscription::{self, Subscription},
};
use libprotonup::{files, github};

use crate::utility::{self, AppInstallWrapper, ReleaseWrapper};

/// Gets the installed apps and their release data from github, returns info for GUI
// Todo, make the error message useful
pub async fn get_app_info() -> Result<Vec<(AppInstallWrapper, Vec<ReleaseWrapper>)>, ()> {
    let launchers = utility::list_installed_apps().await;

    let mut release_map: Vec<(AppInstallWrapper, Vec<ReleaseWrapper>)> = Vec::new();

    let mut future_set = tokio::task::JoinSet::new();

    for launcher in launchers {
        future_set.spawn(return_releases(launcher));
    }

    while let Some(res) = future_set.join_next().await {
        // So many results to deal with
        let (launcher, releases) = if let Ok(Ok(release)) = res {
                release
        } else {
            return Err(());
        };

        release_map.push((launcher, releases));
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
                                for (id, download) in &download_info.requested_downloads {
                                    // TODO, extract the download process into an async function to pass to different threads.
                                    // Also, implement a method to cancel downloads

                                    println!("Starting download for {} from {}", download.version, download.download_url);

                                    // Had to use a String because of some weird lifetime issue,
                                    // I'm guessing because the loop will continue on and download_info gets moved into the next iteration or something
                                    let install_str = String::from(download_info.selected_app.default_install_dir().as_str());

                                    let install_path = std::path::Path::new(&install_str);

                                    let file_name = format!(
                                        "{}.{}",
                                        &download.version,
                                        match &download.download_url {
                                            url if url.ends_with("tar.gz") => "tar.gz",
                                            url if url.ends_with("tar.xz") => "tar.xz",
                                            // Todo, send an error message back to the main thread. Have it try to refresh the GitHub data to fix the issue
                                            _ => {
                                                eprintln!("Requested download from GitHub not of expected type. Expecting tar.(gz|xz)");
                                                continue;
                                            }
                                        }
                                    );

                                    let temp_dir = match tempfile::tempdir() {
                                        Ok(dir) => {println!("Made a temporary folder at {:?}", dir.path()); Some(dir)},
                                        Err(e) => {
                                            eprintln!(
                                                "Error creating temporary directory: {:?}",
                                                e
                                            );
                                            None
                                            // Path::new("/tmp/protonup-rs")
                                        }
                                    };

                                    // Open the file to write to, replacing if it exists
                                    let mut temporary_file = match tokio::fs::OpenOptions::new()
                                        .write(true)
                                        .read(true)
                                        .truncate(true)
                                        .create(true)
                                        .open(
                                            if let Some(ref dir) = temp_dir {
                                                dir.path()
                                            } else {
                                                Path::new("/tmp/protonup-rs")
                                            }
                                            .join(file_name),
                                        )
                                        .await
                                    {
                                        Ok(file) => file,
                                        Err(e) => {
                                            eprintln!("Error opening attempting to create a temporary file: {:?}", e);
                                            continue;
                                        }
                                    };

                                    // Todo find a way to track the progress of the downloaded file, in the TUI ProgressBar, 
                                    // they wrap the AsyncWrite type in their own wrapper that adds progress with a usize tracker.
                                    match files::download_to_async_write(
                                        &download.download_url,
                                        &mut temporary_file,
                                    )
                                    .await
                                    {
                                        Ok(()) => {}
                                        // Todo, send an error back to the Iced application, possibly just restart the download.
                                        Err(e) => {
                                            eprintln!("Error downloading file: {:?}", e);
                                            continue;
                                        }
                                    }

                                    println!("Compressed file downloaded to temporary folder");

                                    // Verify the file was downloaded correctly, match the provided hash
                                    let hash = match files::download_file_into_memory(
                                        &download.sha512sum_url,
                                    )
                                    .await
                                    {
                                        Ok(hash) => hash,
                                        Err(e) => {
                                            eprintln!(
                                                "Error downloading SHA hash from GitHub: {e}"
                                            );
                                            continue;
                                        }
                                    };

                                    // If the file doesn't match the hash, assume the download failed.
                                    if !match files::hash_check_file(&mut temporary_file, &hash)
                                        .await
                                    {
                                        Ok(bool) => bool,
                                        Err(e) => {
                                            eprintln!("{e}");
                                            continue;
                                        }
                                    } {
                                        // Todo, send a message to the main thread and restart the download
                                        eprintln!(
                                            "Compressed file from GitHub failed to match Hash"
                                        );
                                        continue;
                                    }

                                    // Uncompress the successfully downloaded file into the install directory
                                    match files::decompress(temporary_file, install_path).await {
                                        Ok(_) => {},
                                        Err(e) => {eprintln!("Error decompressing {} files, {}", &download_info.selected_app.as_app().app_wine_version(), e);}
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
    release_data: &Option<Vec<(utility::AppInstallWrapper, Vec<utility::ReleaseWrapper>)>>,
    download_handler_tx: &mut Option<mpsc::Sender<HandlerMessage>>,
) {
    // Check that the download handler and release data are ready
    if let Some(h_tx) = download_handler_tx {
        if let Some(release_map) = release_data {
            // Get the GitHub download list for the currently selected app
            if let Some((_app, release_list)) = release_map.iter().find(|(app, _)| app == selected_app) {
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
