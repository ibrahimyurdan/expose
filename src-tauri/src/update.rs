// a lightweight update check: ask github for the latest release and compare its
// tag to the running version. no auto-download, no signing keys — it just emits
// a notice the settings panel shows. it no-ops quietly until the repo is
// published and REPO is set to the real owner/name.

use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

const REPO: &str = "ibrahimyurdan/expose";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current: String,
    pub latest: String,
    pub available: bool,
    pub url: String,
}

pub async fn check(app: AppHandle) {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let info = match fetch_latest().await {
        Some((latest, url)) => UpdateInfo {
            available: is_newer(&latest, &current),
            current,
            latest,
            url,
        },
        None => UpdateInfo {
            latest: current.clone(),
            current,
            available: false,
            url: String::new(),
        },
    };
    let _ = app.emit("expose://update", info);
}

async fn fetch_latest() -> Option<(String, String)> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .ok()?;
    let response = client
        .get(format!(
            "https://api.github.com/repos/{REPO}/releases/latest"
        ))
        .header("User-Agent", "expose-app")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let value: serde_json::Value = response.json().await.ok()?;
    let tag = value.get("tag_name")?.as_str()?.to_string();
    let url = value
        .get("html_url")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    Some((tag, url))
}

fn is_newer(latest: &str, current: &str) -> bool {
    match (parse(latest), parse(current)) {
        (Some(latest), Some(current)) => latest > current,
        _ => false,
    }
}

// parses a "v1.2.3" or "1.2.3" tag into a comparable tuple.
fn parse(version: &str) -> Option<(u32, u32, u32)> {
    let mut parts = version.trim().trim_start_matches('v').split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_a_newer_release() {
        assert!(is_newer("v0.2.0", "0.1.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(is_newer("v0.1.1", "v0.1.0"));
    }

    #[test]
    fn ignores_same_or_older_releases() {
        assert!(!is_newer("v0.1.0", "0.1.0"));
        assert!(!is_newer("v0.1.0", "0.2.0"));
    }

    #[test]
    fn malformed_versions_are_not_newer() {
        assert!(!is_newer("nightly", "0.1.0"));
        assert!(!is_newer("v0.1.0", "garbage"));
    }
}
