// data dragon is riot's static data cdn. we fetch the champion table once on
// startup so the frontend can turn the numeric champion ids in champ select
// into names and square icon urls. it is a public cdn with a valid certificate,
// so this uses an ordinary https client (not the self-signed lcu client).

use std::collections::HashMap;
use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::error::{Error, Result};
use crate::models::{ChampionData, ChampionEntry, DataDragonChampions};
use crate::state::AppState;

const VERSIONS_URL: &str = "https://ddragon.leagueoflegends.com/api/versions.json";

// a launch at sign in can beat the network, so failed loads are retried, waiting
// a little longer each time up to this ceiling.
const RETRY_FIRST: Duration = Duration::from_secs(15);
const RETRY_MAX: Duration = Duration::from_secs(600);

// loads champion data into shared state, retrying until it succeeds. it runs in
// the background and never blocks startup.
pub async fn load(app: AppHandle) {
    let mut wait = RETRY_FIRST;
    loop {
        match fetch().await {
            Ok(data) => {
                *app.state::<AppState>().champions.write() = Some(data);
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

async fn fetch() -> Result<ChampionData> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()?;

    // the first entry is always the newest patch.
    let versions: Vec<String> = client.get(VERSIONS_URL).send().await?.json().await?;
    let version = versions
        .into_iter()
        .next()
        .ok_or_else(|| Error::Message("data dragon returned no versions".to_string()))?;

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
