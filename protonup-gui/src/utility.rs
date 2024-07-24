use core::fmt;
use std::{fmt::Display, ops::Deref};

use libprotonup::{apps::AppInstallations, github};

/// Wrapper around libprotonup::apps::AppInstallations
/// to modify the Display implementation, and track the release list per install
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct AppInstallWrapper {
    pub app_install: AppInstallations,
}

impl Display for AppInstallWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.app_install {
            AppInstallations::Steam => write!(
                f,
                "Steam \"Native\" ({})",
                AppInstallations::Steam.default_install_dir()
            ),
            AppInstallations::SteamFlatpak => write!(
                f,
                "Steam Flatpak ({})",
                AppInstallations::SteamFlatpak.default_install_dir()
            ),
            AppInstallations::Lutris => write!(
                f,
                "Lutris \"Native\" ({})",
                AppInstallations::Lutris.default_install_dir()
            ),
            AppInstallations::LutrisFlatpak => write!(
                f,
                "Lutris Flatpak ({})",
                AppInstallations::LutrisFlatpak.default_install_dir()
            ),
        }
    }
}

/// Wraps AppInstallations in a AppInstallWrapper Vec
pub async fn list_installed_apps() -> Vec<AppInstallWrapper> {
    libprotonup::apps::list_installed_apps()
        .await
        .into_iter()
        .map(|app| app.into())
        .collect()
}

impl From<AppInstallations> for AppInstallWrapper {
    fn from(app_install: AppInstallations) -> Self {
        Self { app_install }
    }
}

impl Deref for AppInstallWrapper {
    type Target = AppInstallations;

    fn deref(&self) -> &Self::Target {
        &self.app_install
    }
}

#[derive(Debug, Clone)]
/// Wrapper around the Wine/Proton GitHub data
///
/// Allows the GUI to keep track of which versions are selected for download.
pub struct ReleaseWrapper {
    release: github::Release,
    pub selected: bool,
}

impl From<github::Release> for ReleaseWrapper {
    fn from(release: github::Release) -> Self {
        Self {
            release,
            selected: false,
        }
    }
}

impl Deref for ReleaseWrapper {
    type Target = github::Release;

    fn deref(&self) -> &Self::Target {
        &self.release
    }
}

impl PartialEq for ReleaseWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.release == other.release
    }
}
