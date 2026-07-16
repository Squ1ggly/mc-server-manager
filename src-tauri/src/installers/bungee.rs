//! BungeeCord installer — the Spigot CI publishes one rolling build with no
//! per-version selection and no checksum.

use std::path::Path;

use crate::error::AppResult;
use crate::installers::vanilla::{McVersion, SERVER_JAR_NAME};
use crate::installers::{download_file, ExpectedChecksum, ProgressCallback};

const BUNGEE_JAR_URL: &str =
    "https://ci.md-5.net/job/BungeeCord/lastSuccessfulBuild/artifact/bootstrap/target/BungeeCord.jar";

/// BungeeCord has a single rolling build.
pub fn list_versions() -> Vec<McVersion> {
    vec![McVersion {
        id: "latest".to_string(),
        kind: "release".to_string(),
        release_time: String::new(),
    }]
}

pub async fn install(
    client: &reqwest::Client,
    server_dir: &Path,
    report_progress: &ProgressCallback,
) -> AppResult<()> {
    let jar_path = server_dir.join(SERVER_JAR_NAME);
    download_file(
        client,
        BUNGEE_JAR_URL,
        &jar_path,
        ExpectedChecksum::None,
        report_progress,
    )
    .await
}
