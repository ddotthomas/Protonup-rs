use libprotonup::{github, variants::VariantGithubParameters};

use crate::utility::AppInstallWrapper;

pub async fn get_launcher_releases(
    launchers: Vec<AppInstallWrapper>,
) -> Result<Vec<LauncherReleases>, ()> {
    let mut release_list: Vec<LauncherReleases> = Vec::with_capacity(4);

    let mut future_set = tokio::task::JoinSet::new();

    for launcher in launchers {
        future_set.spawn(github::list_releases(
            launcher.app_install.get_github_parameters(),
        ));
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

        
    }

    todo!()
}

/// Wrapper labeling the launcher and its list of Releases
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherReleases {
    launcher: AppInstallWrapper,
    releases: Vec<github::Release>,
}
