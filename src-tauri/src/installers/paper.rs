//! Installers for the PaperMC family — Paper, Folia, and Velocity all share
//! the same download API (api.papermc.io v2).

use std::path::Path;

use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::installers::vanilla::{McVersion, SERVER_JAR_NAME};
use crate::installers::{download_file, ExpectedChecksum, ProgressCallback};
use crate::servers::Loader;

const PAPER_API_BASE: &str = "https://api.papermc.io/v2/projects";

#[derive(Debug, Deserialize)]
struct ProjectVersions {
    versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ProjectBuilds {
    builds: Vec<Build>,
}

#[derive(Debug, Deserialize)]
struct Build {
    build: u32,
    downloads: Downloads,
}

#[derive(Debug, Deserialize)]
struct Downloads {
    application: Application,
}

#[derive(Debug, Deserialize)]
struct Application {
    name: String,
    sha256: String,
}

/// The PaperMC project slug for a loader.
fn project_slug(loader: Loader) -> AppResult<&'static str> {
    match loader {
        Loader::Paper => Ok("paper"),
        Loader::Folia => Ok("folia"),
        Loader::Velocity => Ok("velocity"),
        other => {
            let message = format!("{other:?} is not a PaperMC project");
            Err(AppError::InvalidInput(message))
        }
    }
}

/// Versions the project supports, newest first.
pub async fn list_versions(client: &reqwest::Client, loader: Loader) -> AppResult<Vec<McVersion>> {
    let slug = project_slug(loader)?;
    let url = format!("{PAPER_API_BASE}/{slug}");
    let project: ProjectVersions = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let versions = project
        .versions
        .into_iter()
        .rev()
        .map(|id| McVersion {
            id,
            kind: "release".to_string(),
            release_time: String::new(),
        })
        .collect();
    Ok(versions)
}

/// Downloads the latest build of the project for `version` as `server.jar`.
pub async fn install(
    client: &reqwest::Client,
    loader: Loader,
    version: &str,
    server_dir: &Path,
    report_progress: &ProgressCallback,
) -> AppResult<()> {
    let slug = project_slug(loader)?;
    let builds_url = format!("{PAPER_API_BASE}/{slug}/versions/{version}/builds");
    let project_builds: ProjectBuilds = client
        .get(&builds_url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let latest_build = project_builds
        .builds
        .last()
        .ok_or_else(|| AppError::UnknownMinecraftVersion(version.to_string()))?;

    let download_url = format!(
        "{PAPER_API_BASE}/{slug}/versions/{version}/builds/{}/downloads/{}",
        latest_build.build, latest_build.downloads.application.name,
    );
    let jar_path = server_dir.join(SERVER_JAR_NAME);
    download_file(
        client,
        &download_url,
        &jar_path,
        ExpectedChecksum::Sha256(&latest_build.downloads.application.sha256),
        report_progress,
    )
    .await
}
