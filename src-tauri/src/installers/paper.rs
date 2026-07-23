//! Installers for the PaperMC family — Paper, Folia, and Velocity — via the
//! Fill v3 API (the old api.papermc.io v2 was retired with 410 Gone).

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::installers::forgelike::sort_minecraft_versions_desc;
use crate::installers::vanilla::{McVersion, SERVER_JAR_NAME};
use crate::installers::{download_file, fetch_json, ExpectedChecksum, ProgressCallback};
use crate::servers::Loader;

const FILL_API_BASE: &str = "https://fill.papermc.io/v3/projects";

#[derive(Debug, Deserialize)]
struct FillProject {
    /// Version group -> version ids, e.g. "1.21" -> ["1.21.8", "1.21.7"].
    versions: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct FillBuild {
    channel: String,
    downloads: HashMap<String, FillDownload>,
}

#[derive(Debug, Deserialize)]
struct FillDownload {
    url: String,
    checksums: FillChecksums,
}

#[derive(Debug, Deserialize)]
struct FillChecksums {
    sha256: String,
}

/// The Fill project slug for a loader.
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

/// Versions the project supports, newest first. Pre-releases (rc/snapshot
/// ids containing a dash) are marked as snapshots.
pub async fn list_versions(client: &reqwest::Client, loader: Loader) -> AppResult<Vec<McVersion>> {
    let slug = project_slug(loader)?;
    let url = format!("{FILL_API_BASE}/{slug}");
    let project: FillProject = fetch_json(client, &url).await?;

    let mut ids: Vec<String> = project.versions.into_values().flatten().collect();
    sort_minecraft_versions_desc(&mut ids);

    let versions = ids
        .into_iter()
        .map(|id| {
            let kind = if id.contains('-') {
                "snapshot"
            } else {
                "release"
            };
            McVersion {
                id,
                kind: kind.to_string(),
                release_time: String::new(),
            }
        })
        .collect();
    Ok(versions)
}

/// Downloads the newest stable build of the project for `version`.
pub async fn install(
    client: &reqwest::Client,
    loader: Loader,
    version: &str,
    server_dir: &Path,
    report_progress: &ProgressCallback,
) -> AppResult<()> {
    let slug = project_slug(loader)?;
    let builds_url = format!("{FILL_API_BASE}/{slug}/versions/{version}/builds");
    let builds: Vec<FillBuild> = fetch_json(client, &builds_url).await?;

    // Builds are newest-first; prefer a stable/recommended channel.
    let chosen = builds
        .iter()
        .find(|build| {
            build.channel.eq_ignore_ascii_case("stable")
                || build.channel.eq_ignore_ascii_case("recommended")
        })
        .or_else(|| builds.first())
        .ok_or_else(|| AppError::UnknownMinecraftVersion(version.to_string()))?;

    let download = chosen
        .downloads
        .get("server:default")
        .or_else(|| chosen.downloads.values().next())
        .ok_or_else(|| AppError::Process("build has no downloadable server".to_string()))?;

    let jar_path = server_dir.join(SERVER_JAR_NAME);
    download_file(
        client,
        &download.url,
        &jar_path,
        ExpectedChecksum::Sha256(&download.checksums.sha256),
        report_progress,
    )
    .await
}
