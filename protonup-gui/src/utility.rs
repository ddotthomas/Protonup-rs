use core::fmt;
use std::fmt::Display;

use libprotonup::apps::AppInstallations;

/// Wrapper around libprotonup::apps::AppInstallations
/// to modify the Display implementation, and track the release list per install
#[derive(Clone, Debug, PartialEq, Eq)]
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
pub fn list_installed_apps() -> Vec<AppInstallWrapper> {
    libprotonup::apps::list_installed_apps()
        .into_iter()
        .map(|app| app.into())
        .collect()
}

impl From<AppInstallations> for AppInstallWrapper {
    fn from(app_install: AppInstallations) -> Self {
        Self { app_install }
    }
}
