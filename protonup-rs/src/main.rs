use clap::Parser;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use inquire::{Confirm, MultiSelect, Select, Text};
use std::fmt;
use std::fs;
use std::fs::create_dir_all;
use std::sync::atomic::Ordering;
use std::thread;
use std::{sync::Arc, time::Duration};
mod file_path;

use libprotonup::{constants, file, github, parameters, utils};

#[derive(Debug, Parser)]
struct Opt {
    /// Skip Menu and download latest directly
    #[arg(short, long)]
    quick_download: bool,
    #[arg(short = 'f', long)]
    quick_download_flatpak: bool,
    /// Download latest Wine GE for Lutris
    #[arg(short, long)]
    lutris_quick_download: bool,
    #[arg(short = 'L', long)]
    lutris_quick_download_flatpak: bool,
}

#[derive(Debug, Copy, Clone)]
#[allow(clippy::upper_case_acronyms)]
enum Menu {
    QuickUpdate,
    QuickUpdateFlatpak,
    QuickUpdateLutris,
    QuickUpdateLutrisFlatpak,
    ChoseReleases,
    ChoseReleasesFlatpak,
    ChoseReleasesCustomDir,
    ChoseReleasesLutris,
    ChoseReleasesLutrisFlatpak,
}

impl Menu {
    // could be generated by macro
    const VARIANTS: &'static [Menu] = &[
        Self::QuickUpdate,
        Self::QuickUpdateFlatpak,
        Self::QuickUpdateLutris,
        Self::QuickUpdateLutrisFlatpak,
        Self::ChoseReleases,
        Self::ChoseReleasesFlatpak,
        Self::ChoseReleasesCustomDir,
        Self::ChoseReleasesLutris,
        Self::ChoseReleasesLutrisFlatpak,
    ];
}

impl fmt::Display for Menu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::QuickUpdate => write!(f, "Quick Update Steam (download latest GE Proton)"),
            Self::QuickUpdateFlatpak => write!(
                f,
                "Quick Update (download latest GE Proton) for Flatpak Steam"
            ),
            Self::QuickUpdateLutris => write!(f, "Quick Update Lutris (download latest Wine GE)"),
            Self::QuickUpdateLutrisFlatpak => {
                write!(f, "Quick Update Lutris Flatpak (download latest Wine GE)")
            }
            Self::ChoseReleases => write!(f, "Choose GE Proton releases from list"),
            Self::ChoseReleasesFlatpak => {
                write!(f, "Choose GE Proton releases from list for Flatpak Steam")
            }
            Self::ChoseReleasesCustomDir => write!(
                f,
                "Choose GE Proton releases and install to custom directory"
            ),
            Self::ChoseReleasesLutris => write!(f, "Choose Wine GE releases for Lutris"),
            Self::ChoseReleasesLutrisFlatpak => {
                write!(f, "Choose Wine GE releases for Flatpak Lutris")
            }
        }
    }
}

fn tag_menu(options: Vec<String>) -> Vec<String> {
    let answer = MultiSelect::new("Select the versions you want to download :", options)
        .with_default(&[0_usize])
        .prompt();

    match answer {
        Ok(list) => list,

        Err(_) => {
            println!("The tag list could not be processed");
            vec![]
        }
    }
}

fn confirm_menu(text: String) -> bool {
    let answer = Confirm::new(&text)
        .with_default(false)
        .with_help_message("If you choose yes, we will re-install it.")
        .prompt();

    answer.unwrap_or(false)
}

#[tokio::main]
async fn main() {
    // run quick downloads and skip menu
    if run_quick_downloads().await {
        return;
    }

    // Default Parameters
    let source: parameters::VariantParameters;
    let mut install_dir = constants::DEFAULT_STEAM_INSTALL_DIR.to_string();
    let mut tags: Vec<String> = vec![String::from("latest")];

    let mut should_open_tag_selector = false;
    let mut should_open_dir_selector = false;

    let answer: Menu = Select::new("ProtonUp Menu: Chose your action:", Menu::VARIANTS.to_vec())
        .with_page_size(9)
        .prompt()
        .unwrap_or_else(|_| std::process::exit(0));

    // Set parameters based on users choice
    match answer {
        Menu::QuickUpdate => {
            //            let tag = match github::fetch_data_from_tag("latest", false).await {
            //                Ok(data) => data,
            //                Err(e) => {
            //                    eprintln!("Failed to fetch Github data, make sure you're connected to the internet.\nError: {}", e);
            //                    std::process::exit(1)
            //                }
            //            };
            //
            //            if file::check_if_exists(
            //                constants::DEFAULT_STEAM_INSTALL_DIR.to_owned(),
            //                tag.version.clone(),
            //            ) && !confirm_menu(format!(
            //                "Version {} exists in installation path. Overwrite?",
            //                tag.version
            //            )) {
            //                return;
            //            }
            //
            //            download_file(
            //                "latest",
            //                constants::DEFAULT_STEAM_INSTALL_DIR.to_string(),
            //                false,
            //            )
            //            .await
            //            .unwrap();
            //        }
            //        Menu::QuickUpdateFlatpak => {
            //            let tag = match github::fetch_data_from_tag("latest", false).await {
            //                Ok(data) => data,
            //                Err(e) => {
            //                    eprintln!("Failed to fetch Github data, make sure you're connected to the internet.\nError: {}", e);
            //                    std::process::exit(1)
            //                }
            //            };
            //
            //            if file::check_if_exists(
            //                constants::DEFAULT_STEAM_INSTALL_DIR_FLATPAK.to_owned(),
            //                tag.version.clone(),
            //            ) && !confirm_menu(format!(
            //                "Version {} exists in installation path. Overwrite?",
            //                tag.version
            //            )) {
            //                return;
            //            }
            //
            //            download_file(
            //                "latest",
            //                constants::DEFAULT_STEAM_INSTALL_DIR_FLATPAK.to_string(),
            //                false,
            //            )
            //            .await
            //            .unwrap();
            source = parameters::Variant::GEProton.parameters();
            install_dir = constants::DEFAULT_STEAM_INSTALL_DIR.to_owned();
        }
        Menu::QuickUpdateFlatpak => {
            source = parameters::Variant::GEProton.parameters();
            install_dir = constants::DEFAULT_STEAM_INSTALL_DIR_FLATPAK.to_owned();
        }
        Menu::QuickUpdateLutris => {
            source = parameters::Variant::WineGE.parameters();
            install_dir = constants::DEFAULT_LUTRIS_INSTALL_DIR.to_owned();
        }
        Menu::QuickUpdateLutrisFlatpak => {
            source = parameters::Variant::WineGE.parameters();
            install_dir = constants::DEFAULT_LUTRIS_INSTALL_DIR_FLATPAK.to_owned();
        }
        Menu::ChoseReleases => {
            //            let release_list = match github::list_releases(false).await {
            //                Ok(data) => data,
            //                Err(e) => {
            //                    eprintln!("Failed to fetch Github data, make sure you're connected to the internet.\nError: {}", e);
            //                    std::process::exit(1)
            //                }
            //            };
            //            let tag_list: Vec<String> = release_list.into_iter().map(|r| (r.tag_name)).collect();
            //            let list = tag_menu(tag_list);
            //            for tag in list.iter() {
            //                if file::check_if_exists(
            //                    constants::DEFAULT_STEAM_INSTALL_DIR.to_owned(),
            //                    tag.to_owned(),
            //                ) && !confirm_menu(format!(
            //                    "Version {tag} exists in installation path. Overwrite?"
            //                )) {
            //                    return;
            //                }
            //                download_file(tag, constants::DEFAULT_STEAM_INSTALL_DIR.to_string(), false)
            //                    .await
            //                    .unwrap();
            //            }
            //        }
            //        Menu::ChoseReleasesFlatpak => {
            //            let release_list = match github::list_releases(false).await {
            //                Ok(data) => data,
            //                Err(e) => {
            //                    eprintln!("Failed to fetch Github data, make sure you're connected to the internet.\nError: {}", e);
            //                    std::process::exit(1)
            //                }
            //            };
            //            let tag_list: Vec<String> = release_list.into_iter().map(|r| (r.tag_name)).collect();
            //            let list = tag_menu(tag_list);
            //            for tag in list.iter() {
            //                if file::check_if_exists(
            //                    constants::DEFAULT_STEAM_INSTALL_DIR_FLATPAK.to_owned(),
            //                    tag.to_owned(),
            //                ) && !confirm_menu(format!(
            //                    "Version {tag} exists in installation path. Overwrite?"
            //                )) {
            //                    return;
            //                }
            //                download_file(
            //                    tag,
            //                    constants::DEFAULT_STEAM_INSTALL_DIR_FLATPAK.to_string(),
            //                    false,
            //                )
            //                .await
            //                .unwrap();
            //            }
            //        }
            //        Menu::ChoseReleasesCustomDir => {
            //            let current_dir = std::env::current_dir().unwrap();
            //            let help_message = format!("Current directory: {}", current_dir.to_string_lossy());
            //            let answer = Text::new("Installation path:")
            //                .with_autocomplete(file_path::FilePathCompleter::default())
            //                .with_help_message(&help_message)
            //                .prompt();
            //
            //            let chosen_path = match answer {
            //                Ok(path) => path,
            //                Err(error) => {
            //                    println!("Error choosing custom path. Using the default. Error: {error:?}");
            //                    constants::DEFAULT_STEAM_INSTALL_DIR.to_string()
            //                }
            //            };
            //            let release_list = match github::list_releases(false).await {
            //                Ok(data) => data,
            //                Err(e) => {
            //                    eprintln!("Failed to fetch Github data, make sure you're connected to the internet.\nError: {}", e);
            //                    std::process::exit(1)
            //                }
            //            };
            //            let tag_list: Vec<String> = release_list.into_iter().map(|r| (r.tag_name)).collect();
            //            let list = tag_menu(tag_list);
            //            for tag in list.iter() {
            //                if file::check_if_exists(
            //                    constants::DEFAULT_STEAM_INSTALL_DIR.to_owned(),
            //                    tag.to_owned(),
            //                ) && !confirm_menu(format!(
            //                    "Version {tag} exists in installation path. Overwrite?"
            //                )) {
            //                    return;
            //                }
            //
            //                download_file(tag, chosen_path.clone(), false)
            //                    .await
            //                    .unwrap();
            //            }
            source = parameters::Variant::GEProton.parameters();
            install_dir = constants::DEFAULT_STEAM_INSTALL_DIR.to_owned();
            should_open_tag_selector = true;
        }
        Menu::ChoseReleasesFlatpak => {
            source = parameters::Variant::GEProton.parameters();
            install_dir = constants::DEFAULT_STEAM_INSTALL_DIR_FLATPAK.to_owned();
            should_open_tag_selector = true;
        }
        Menu::ChoseReleasesCustomDir => {
            source = parameters::Variant::GEProton.parameters();
            should_open_dir_selector = true;
            should_open_tag_selector = true;
        }
        Menu::ChoseReleasesLutris => {
            source = parameters::Variant::WineGE.parameters();
            install_dir = constants::DEFAULT_LUTRIS_INSTALL_DIR.to_owned();
            should_open_tag_selector = true;
        }
        Menu::ChoseReleasesLutrisFlatpak => {
            source = parameters::Variant::WineGE.parameters();
            install_dir = constants::DEFAULT_LUTRIS_INSTALL_DIR_FLATPAK.to_owned();
            should_open_tag_selector = true;
        }
    }

    // This is where the execution happens

    if should_open_dir_selector {
        let current_dir = std::env::current_dir().unwrap();
        let help_message = format!("Current directory: {}", current_dir.to_string_lossy());
        let answer = Text::new("Installation path:")
            .with_autocomplete(file_path::FilePathCompleter::default())
            .with_help_message(&help_message)
            .prompt();

        match answer {
            Ok(path) => install_dir = path,
            Err(error) => {
                println!("Error choosing custom path. Using the default. Error: {error:?}");
            }
        };
    }

    if should_open_tag_selector {
        tags = vec![];
        let release_list = match github::list_releases(&source).await {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to fetch Github data, make sure you're connected to the internet.\nError: {}", e);
                std::process::exit(1)
            }
        };
        let tag_list: Vec<String> = release_list.into_iter().map(|r| (r.tag_name)).collect();
        let list = tag_menu(tag_list);
        for tag_iter in list.iter() {
            let tag = String::from(tag_iter);
            tags.push(tag);
        }
    }

    tags.retain(|tag_name| {
        // Check if versions exist in disk.
        // If they do, ask the user if it should be overwritten
        !(file::check_if_exists(&install_dir, &tag_name)
            && !confirm_menu(format!(
                "Version {tag_name} exists in installation path. Overwrite?"
            )))
    });

    // install the versions that are in the tags array
    for tag_name in tags {
        let tag = match github::fetch_data_from_tag(&tag_name, &source).await {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to fetch Github data, make sure you're connected to the internet.\nError: {}", e);
                std::process::exit(1)
            }
        };

        match download_file(&tag_name, &install_dir, &source).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "Error downloading {}, make sure you're connected to the internet\nError: {}",
                    tag.version, e
                )
            }
        }
    }
}

pub async fn download_file(
    tag: &str,
    install_path: &str,
    source: &parameters::VariantParameters,
) -> Result<(), String> {
    let install_dir = utils::expand_tilde(install_path).unwrap();
    let mut temp_dir = utils::expand_tilde(constants::TEMP_DIR).unwrap();

    let download = match github::fetch_data_from_tag(tag, &source).await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to fetch GitHub data, make sure you're connected to the internet\nError: {}", e);
            std::process::exit(1)
        }
    };

    temp_dir.push(if download.download_url.ends_with("tar.gz") {
        format!("{}.tar.gz", &download.version)
    } else if download.download_url.ends_with("tar.xz") {
        format!("{}.tar.xz", &download.version)
    } else {
        eprintln!("Downloaded file wasn't of the expected type. tar.(gz/xz)");
        std::process::exit(1)
    });

    // install_dir
    create_dir_all(&install_dir).unwrap();

    let git_hash = file::download_sha512_into_memory(&download.sha512sum_url)
        .await
        .unwrap();

    if temp_dir.exists() {
        fs::remove_file(&temp_dir).unwrap();
    }

    let (progress, done) = file::create_progress_trackers();
    let progress_read = Arc::clone(&progress);
    let done_read = Arc::clone(&done);
    let url = String::from(&download.download_url);
    let tmp_dir = String::from(temp_dir.to_str().unwrap());

    // start ProgressBar in another thread
    thread::spawn(move || {
        let pb = ProgressBar::with_draw_target(
            Some(download.size),
            ProgressDrawTarget::stderr_with_hz(20),
        );
        pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec})").unwrap()
        .progress_chars("#>-"));
        pb.set_message(format!("Downloading {}", url.split('/').last().unwrap()));
        let wait_time = Duration::from_millis(50); // 50ms wait is about 20Hz
        loop {
            let newpos = progress_read.load(Ordering::Relaxed);
            pb.set_position(newpos as u64);
            if done_read.load(Ordering::Relaxed) {
                break;
            }
            thread::sleep(wait_time);
        }
        pb.set_message(format!("Downloaded {url} to {tmp_dir}"));
        pb.abandon(); // closes progress bar without blanking terminal

        println!("Checking file integrity"); // This is being printed here because the progress bar needs to be closed before printing.
    });

    file::download_file_progress(
        download.download_url,
        download.size,
        temp_dir.clone().as_path(),
        progress,
        done,
    )
    .await
    .unwrap();
    if !file::hash_check_file(temp_dir.to_str().unwrap().to_string(), git_hash).unwrap() {
        return Err("Failed checking file hash".to_string());
    }
    println!("Unpacking files into install location. Please wait");
    file::decompress(temp_dir.as_path(), install_dir.clone().as_path()).unwrap();
    let source_type = source.variant_type();
    println!(
        "Done! Restart {}. {} installed in {}",
        source_type.intended_application(),
        source_type.to_string(),
        install_dir.to_string_lossy(),
    );
    Ok(())
}

async fn run_quick_downloads() -> bool {
    let Opt {
        quick_download,
        quick_download_flatpak,
        lutris_quick_download,
        lutris_quick_download_flatpak,
    } = Opt::parse();

    if quick_download {
        let source = parameters::Variant::GEProton;
        let destination = constants::DEFAULT_STEAM_INSTALL_DIR.to_string();
        println!(
            "\nQuick Download: {} / {} into -> {}\n",
            source.to_string(),
            source.intended_application(),
            destination
        );
        download_file("latest", &destination, &source.parameters())
            .await
            .unwrap();
    }

    if quick_download_flatpak {
        let source = parameters::Variant::GEProton;
        let destination = constants::DEFAULT_STEAM_INSTALL_DIR_FLATPAK.to_string();
        println!(
            "\nQuick Download: {} / {} into -> {}\n",
            source.to_string(),
            source.intended_application(),
            destination
        );
        download_file("latest", &destination, &source.parameters())
            .await
            .unwrap();
    }

    if lutris_quick_download {
        let source = parameters::Variant::WineGE;
        let destination = constants::DEFAULT_LUTRIS_INSTALL_DIR.to_string();
        println!(
            "\nQuick Download: {} / {} into -> {}\n",
            source.to_string(),
            source.intended_application(),
            destination
        );
        download_file("latest", &destination, &source.parameters())
            .await
            .unwrap();
    }

    if lutris_quick_download_flatpak {
        let source = parameters::Variant::WineGE;
        let destination = constants::DEFAULT_LUTRIS_INSTALL_DIR_FLATPAK.to_string();
        println!(
            "\nQuick Download: {} / {} into -> {}\n",
            source.to_string(),
            source.intended_application(),
            destination
        );
        download_file("latest", &destination, &source.parameters())
            .await
            .unwrap();
    }

    return quick_download
        || quick_download_flatpak
        || lutris_quick_download
        || lutris_quick_download_flatpak;
}
