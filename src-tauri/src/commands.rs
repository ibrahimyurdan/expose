// the commands the frontend can invoke. status and teammates also arrive as
// push events; these commands cover the initial read on mount and the toggle.

use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;

use crate::models::{ChampionData, ConnectionStatus};
use crate::scout;
use crate::state::AppState;

pub const STORE_FILE: &str = "settings.json";
pub const AUTO_ACCEPT_KEY: &str = "autoAccept";
pub const SCOUT_PROVIDER_KEY: &str = "scoutProvider";
pub const AUTO_OPEN_KEY: &str = "autoOpen";
pub const REVEAL_RANKED_KEY: &str = "revealRanked";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub auto_accept: bool,
    pub scout_provider: String,
    pub auto_open: bool,
    pub reveal_ranked: bool,
    pub launch_at_login: bool,
}

#[tauri::command]
pub fn get_status(state: State<AppState>) -> ConnectionStatus {
    *state.status.read()
}

#[tauri::command]
pub fn get_auto_accept(state: State<AppState>) -> bool {
    state.auto_accept.load(Ordering::Relaxed)
}

#[tauri::command]
pub fn set_auto_accept(app: AppHandle, state: State<AppState>, enabled: bool) {
    state.auto_accept.store(enabled, Ordering::Relaxed);
    if let Ok(store) = app.store(STORE_FILE) {
        store.set(AUTO_ACCEPT_KEY, enabled);
        if let Err(error) = store.save() {
            eprintln!("expose: could not persist settings: {error}");
        }
    }
}

#[tauri::command]
pub fn get_champions(state: State<AppState>) -> Option<ChampionData> {
    state.champions.read().clone()
}

// opens a link (the op.gg button) in the system browser. routing this through a
// command keeps the opener plugin out of the frontend capability surface. the
// scheme guard makes sure only web links are ever handed to the os.
#[tauri::command]
pub fn open_external(app: AppHandle, url: String) {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return;
    }
    if let Err(error) = app.opener().open_url(url, None::<&str>) {
        eprintln!("expose: could not open external url: {error}");
    }
}

// leaves the current champ select (a dodge) without closing the client. only
// works while connected; the server-side dodge penalty still applies.
#[tauri::command]
pub async fn dodge(state: State<'_, AppState>) -> Result<(), String> {
    let client = state.client.read().clone();
    match client {
        Some(client) => client.dodge().await.map_err(|error| error.to_string()),
        None => Err("not connected to the league client".to_string()),
    }
}

#[tauri::command]
pub fn get_settings(app: AppHandle, state: State<AppState>) -> Settings {
    let provider = state.scout_provider.read().clone();
    Settings {
        auto_accept: state.auto_accept.load(Ordering::Relaxed),
        // only op.gg and u.gg support a shareable multi-search; coerce any other
        // stored value (empty, or a removed "deeplol"/"tracker") to op.gg so the
        // picker never shows an invalid selection.
        scout_provider: match provider.as_str() {
            "ugg" => "ugg".to_string(),
            _ => "opgg".to_string(),
        },
        auto_open: state.auto_open.load(Ordering::Relaxed),
        reveal_ranked: state.reveal_ranked.load(Ordering::Relaxed),
        launch_at_login: app.autolaunch().is_enabled().unwrap_or(false),
    }
}

// toggles revealing teammate names in anonymous ranked champ select. defaults on;
// turning it off makes expose respect riot's anonymity in ranked.
#[tauri::command]
pub fn set_reveal_ranked(app: AppHandle, state: State<AppState>, enabled: bool) {
    state.reveal_ranked.store(enabled, Ordering::Relaxed);
    if let Ok(store) = app.store(STORE_FILE) {
        store.set(REVEAL_RANKED_KEY, enabled);
        let _ = store.save();
    }
}

#[tauri::command]
pub fn set_scout_provider(app: AppHandle, state: State<AppState>, provider: String) {
    *state.scout_provider.write() = provider.clone();
    if let Ok(store) = app.store(STORE_FILE) {
        store.set(SCOUT_PROVIDER_KEY, provider.clone());
        let _ = store.save();
    }

    // rebuild the per-row links for any already-resolved team so switching site
    // takes effect immediately, not only on the next champ-select update.
    let label = scout::provider_label(&provider).to_string();
    let region = state.region.read().clone();
    let snapshot = {
        let mut teammates = state.teammates.write();
        for teammate in teammates.iter_mut() {
            teammate.scout_url = scout::profile_url(
                &provider,
                region.as_deref(),
                &teammate.game_name,
                &teammate.tag_line,
            );
            teammate.scout_label = label.clone();
        }
        teammates.clone()
    };
    let _ = app.emit(crate::features::EVENT_TEAMMATES, &snapshot);
}

#[tauri::command]
pub fn set_auto_open(app: AppHandle, state: State<AppState>, enabled: bool) {
    state.auto_open.store(enabled, Ordering::Relaxed);
    if let Ok(store) = app.store(STORE_FILE) {
        store.set(AUTO_OPEN_KEY, enabled);
        let _ = store.save();
    }
}

#[tauri::command]
pub fn set_launch_at_login(app: AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    result.map_err(|error| error.to_string())
}

// opens the chosen scout site with the whole resolved team in one tab.
#[tauri::command]
pub fn open_scout(app: AppHandle, state: State<AppState>) {
    let teammates = state.teammates.read().clone();
    let provider = state.scout_provider.read().clone();
    let region = state.region.read().clone();
    if let Some(url) = scout::scout_url(&provider, &teammates, region.as_deref()) {
        if let Err(error) = app.opener().open_url(url, None::<&str>) {
            eprintln!("expose: could not open scout url: {error}");
        }
    }
}
