//! Fabric installer (meta.fabricmc.net) — the meta service serves a ready
//! launcher jar for any game/loader/installer combination.

use std::path::Path;

use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::installers::vanilla::{McVersion, SERVER_JAR_NAME};
use crate::installers::{download_file, ExpectedChecksum, ProgressCallback};

const FABRIC_META_BASE: &str = "https://meta.fabricmc.net/v2/versions";

#[derive(Debug, Deserialize)]
struct GameVersion {
    version: String,
    stable: bool,
}

#[derive(Debug, Deserialize)]
struct ToolVersion {
    version: String,
    stable: bool,
}

/// Game versions Fabric supports, newest first (snapshots marked as such).
pub async fn list_versions(client: &reqwest::Client) -> AppResult<Vec<McVersion>> {
    let game_versions: Vec<GameVersion> = client
        .get(format!("{FABRIC_META_BASE}/game"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let versions = game_versions
        .into_iter()
        .map(|game| McVersion {
            id: game.version,
            kind: if game.stable { "release" } else { "snapshot" }.to_string(),
            release_time: String::new(),
        })
        .collect();
    Ok(versions)
}

/// Downloads the Fabric server launcher for `mc_version` as `server.jar`,
/// using the newest stable loader and installer.
pub async fn install(
    client: &reqwest::Client,
    mc_version: &str,
    server_dir: &Path,
    report_progress: &ProgressCallback,
) -> AppResult<()> {
    let loader_version = latest_stable(client, "loader").await?;
    let installer_version = latest_stable(client, "installer").await?;

    let download_url = format!(
        "{FABRIC_META_BASE}/loader/{mc_version}/{loader_version}/{installer_version}/server/jar"
    );
    let jar_path = server_dir.join(SERVER_JAR_NAME);
    download_file(
        client,
        &download_url,
        &jar_path,
        ExpectedChecksum::None,
        report_progress,
    )
    .await
}

async fn latest_stable(client: &reqwest::Client, tool: &str) -> AppResult<String> {
    let tools: Vec<ToolVersion> = client
        .get(format!("{FABRIC_META_BASE}/{tool}"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let newest_stable = tools.into_iter().find(|entry| entry.stable);
    let found = newest_stable
        .ok_or_else(|| AppError::Process(format!("no stable Fabric {tool} available")))?;
    Ok(found.version)
}
