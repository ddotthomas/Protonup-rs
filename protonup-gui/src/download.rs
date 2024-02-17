use std::collections::HashMap;

use libprotonup::github;

use crate::utility::{AppInstallWrapper, ReleaseWrapper};

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