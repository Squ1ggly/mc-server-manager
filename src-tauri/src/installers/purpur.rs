//! Purpur installer (api.purpurmc.org). Purpur publishes only an MD5, so
//! downloads rely on HTTPS integrity.

use std::path::Path;

use serde::Deserialize;

use crate::error::AppResult;
use crate::installers::vanilla::{McVersion, SERVER_JAR_NAME};
use crate::installers::{download_file, fetch_json, ExpectedChecksum, ProgressCallback};

const PURPUR_API_BASE: &str = "https://api.purpurmc.org/v2/purpur";

#[derive(Debug, Deserialize)]
struct PurpurVersions {
    versions: Vec<String>,
}

/// Versions Purpur supports, newest first.
pub async fn list_versions(client: &reqwest::Client) -> AppResult<Vec<McVersion>> {
    let listing: PurpurVersions = fetch_json(client, PURPUR_API_BASE).await?;

    let versions = listing
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

/// Downloads the latest Purpur build for `version` as `server.jar`.
pub async fn install(
    client: &reqwest::Client,
    version: &str,
    server_dir: &Path,
    report_progress: &ProgressCallback,
) -> AppResult<()> {
    let download_url = format!("{PURPUR_API_BASE}/{version}/latest/download");
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
