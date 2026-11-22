// data dragon is riot's static data cdn. we fetch the champion table on
// startup, and again when league connects after a patch, so the frontend can
// turn the numeric champion ids in champ select into names and square icon
// urls. it is a public cdn with a valid certificate, so this uses an ordinary
// https client (not the self-signed lcu client).

use std::collections::HashMap;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use crate::error::{Error, Result};
use crate::models::{ChampionData, ChampionEntry, DataDragonChampions};
use crate::state::AppState;

const VERSIONS_URL: &str = "https://ddragon.leagueoflegends.com/api/versions.json";

// sent with the champion table whenever it loads, so the frontend needs no poll.
pub const EVENT_CHAMPIONS: &str = "expose://champions";

// a launch at sign in can beat the network, so failed loads are retried, waiting
// a little longer each time up to this ceiling.
const RETRY_FIRST: Duration = Duration::from_secs(15);
const RETRY_MAX: Duration = Duration::from_secs(600);

// loads champion data into shared state, retrying until it succeeds or a
// refresh got there first. it runs in the background and never blocks startup.
pub async fn load(app: AppHandle) {
    let mut wait = RETRY_FIRST;
    loop {
        if app.state::<AppState>().champions.read().is_some() {
            return;
        }
        match fetch_latest().await {
            Ok(data) => {
                publish(&app, data);
                return;
            }
            Err(error) => {
                log::warn!("could not load champion data, retrying in {wait:?}: {error}");
            }
        }
        tokio::time::sleep(wait).await;
        wait = (wait * 2).min(RETRY_MAX);
    }
}

// checks for a newer patch when league connects (patch day) and reloads the
// table only when there is one. failures keep the table expose already has.
pub async fn refresh(app: AppHandle) {
    let current = app
        .state::<AppState>()
        .champions
        .read()
        .as_ref()
        .map(|data| data.version.clone());
    let result = async {
        let client = http_client()?;
        let latest = latest_version(&client).await?;
        if !needs_reload(current.as_deref(), &latest) {
            return Ok(None);
        }
        champions(&client, latest).await.map(Some)
    }
    .await;
    match result {
        Ok(Some(data)) => {
            log::info!("loaded champion data for patch {}", data.version);
            publish(&app, data);
        }
        Ok(None) => {}
        Err(error) => log::warn!("could not refresh champion data: {error}"),
    }
}

// a different newest patch than the one loaded (or none loaded yet).
fn needs_reload(current: Option<&str>, latest: &str) -> bool {
    current != Some(latest)
}

fn publish(app: &AppHandle, data: ChampionData) {
    *app.state::<AppState>().champions.write() = Some(data.clone());
    if let Err(error) = app.emit(EVENT_CHAMPIONS, &data) {
        log::warn!("could not emit champion data: {error}");
    }
}

fn http_client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?)
}

async fn fetch_latest() -> Result<ChampionData> {
    let client = http_client()?;
    let version = latest_version(&client).await?;
    champions(&client, version).await
}

// the first entry is always the newest patch.
async fn latest_version(client: &reqwest::Client) -> Result<String> {
    let versions: Vec<String> = client.get(VERSIONS_URL).send().await?.json().await?;
    versions
        .into_iter()
        .next()
        .ok_or_else(|| Error::Message("data dragon returned no versions".to_string()))
}

async fn champions(client: &reqwest::Client, version: String) -> Result<ChampionData> {
    let url = format!("https://ddragon.leagueoflegends.com/cdn/{version}/data/en_US/champion.json");
    let file: DataDragonChampions = client.get(url).send().await?.json().await?;

    let mut champions = HashMap::with_capacity(file.data.len());
    for champion in file.data.into_values() {
        if let Ok(key) = champion.key.parse::<i64>() {
            champions.insert(
                key,
                ChampionEntry {
                    id: champion.id,
                    name: champion.name,
                },
            );
        }
    }

    Ok(ChampionData { version, champions })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reloads_only_for_a_newer_patch() {
        assert!(needs_reload(None, "16.23.1"));
        assert!(!needs_reload(Some("16.23.1"), "16.23.1"));
        assert!(needs_reload(Some("16.23.1"), "16.24.1"));
    }
}
