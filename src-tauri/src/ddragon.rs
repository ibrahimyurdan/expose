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

// loads champion data into shared state. a failure is logged and left for the
// frontend to retry; it never blocks startup.
pub async fn load(app: AppHandle) {
    match fetch().await {
        Ok(data) => {
            *app.state::<AppState>().champions.write() = Some(data);
        }
        Err(error) => {
            eprintln!("expose: could not load champion data: {error}");
        }
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
