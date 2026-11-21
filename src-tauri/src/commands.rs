// the commands the frontend can invoke. status and the lobby also arrive as
// push events; these commands cover the initial read on mount and the toggles.

use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;

use crate::features::dodge;
use crate::lobby;
use crate::lobby::view::LobbyView;
use crate::models::{ChampionData, ConnectionStatus};
use crate::state::AppState;
use crate::update::UpdateInfo;

pub const STORE_FILE: &str = "settings.json";
pub const AUTO_ACCEPT_KEY: &str = "autoAccept";
pub const SCOUT_PROVIDER_KEY: &str = "scoutProvider";
pub const AUTO_OPEN_KEY: &str = "autoOpen";
pub const REVEAL_RANKED_KEY: &str = "revealRanked";
pub const DODGE_LEAD_KEY: &str = "dodgeLeadMs";
pub const WELCOMED_KEY: &str = "welcomed";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub auto_accept: bool,
    pub scout_provider: String,
    pub auto_open: bool,
    pub reveal_ranked: bool,
    pub launch_at_login: bool,
    // how long before the lobby locks in the last-second dodge fires.
    pub dodge_lead_ms: i64,
    // the first-run welcome has been dismissed.
    pub welcomed: bool,
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
            log::warn!("could not persist settings: {error}");
        }
    }
}

#[tauri::command]
pub fn get_champions(state: State<AppState>) -> Option<ChampionData> {
    state.champions.read().clone()
}

// the startup update check, or none while it is still running.
#[tauri::command]
pub fn get_update(state: State<AppState>) -> Option<UpdateInfo> {
    state.update.read().clone()
}

// the latest lobby view, for a frontend that mounts mid champ select; none
// outside champ select.
#[tauri::command]
pub fn get_lobby(state: State<AppState>) -> Option<LobbyView> {
    state.lobby_view.read().clone()
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
        log::warn!("could not open external url: {error}");
    }
}

// leaves the current champ select (a dodge) without closing the client. only
// works while connected; the server-side dodge penalty still applies.
#[tauri::command]
pub async fn dodge(app: AppHandle) -> Result<(), String> {
    dodge::dodge_now(&app).await
}

// arms or disarms the last-second dodge for the current lobby. outside champ
// select there is nothing to arm.
#[tauri::command]
pub fn set_last_second_dodge(app: AppHandle, state: State<AppState>, armed: bool) {
    let game_id = state
        .lobby
        .lock()
        .session
        .as_ref()
        .map(|session| session.game_id);
    match game_id {
        Some(game_id) if armed => dodge::arm(&app, game_id),
        _ => dodge::disarm(&app),
    }
    lobby::refresh(&app);
}

// how long before the lobby locks in the last-second dodge fires, clamped to
// 1 to 3 seconds.
#[tauri::command]
pub fn set_dodge_lead(app: AppHandle, state: State<AppState>, ms: i64) {
    let lead = state.dodge.set_lead_ms(ms);
    if let Ok(store) = app.store(STORE_FILE) {
        store.set(DODGE_LEAD_KEY, lead);
        let _ = store.save();
    }
    // an armed dodge already counting down moves to the new lead.
    let session = state.lobby.lock().session.clone();
    if let Some(session) = session {
        dodge::on_session(&app, &session);
    }
    lobby::refresh(&app);
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
        dodge_lead_ms: state.dodge.lead_ms(),
        welcomed: app
            .store(STORE_FILE)
            .ok()
            .and_then(|store| store.get(WELCOMED_KEY))
            .and_then(|value| value.as_bool())
            .unwrap_or(false),
    }
}

// the first-run welcome is done with, for good.
#[tauri::command]
pub fn set_welcomed(app: AppHandle) {
    if let Ok(store) = app.store(STORE_FILE) {
        store.set(WELCOMED_KEY, true);
        if let Err(error) = store.save() {
            log::warn!("could not persist settings: {error}");
        }
    }
}

// opens the folder expose writes its log file to, so a problem can be
// reported with the log attached.
#[tauri::command]
pub fn open_logs_folder(app: AppHandle) -> Result<(), String> {
    let folder = app
        .path()
        .app_log_dir()
        .map_err(|error| error.to_string())?;
    // made if need be (nothing logged yet), so there is always one to open.
    std::fs::create_dir_all(&folder).map_err(|error| error.to_string())?;
    app.opener()
        .open_path(folder.to_string_lossy(), None::<&str>)
        .map_err(|error| error.to_string())
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
    // turned off, hidden teammates lose their names in the open lobby at once.
    lobby::refresh(&app);
}

#[tauri::command]
pub fn set_scout_provider(app: AppHandle, state: State<AppState>, provider: String) {
    *state.scout_provider.write() = provider.clone();
    if let Ok(store) = app.store(STORE_FILE) {
        store.set(SCOUT_PROVIDER_KEY, provider);
        let _ = store.save();
    }
    // rebuild the lobby's profile links so switching site takes effect
    // immediately, not only on the next champ-select update.
    lobby::refresh(&app);
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

// opens the chosen scout site with everyone in the lobby in one tab.
#[tauri::command]
pub fn open_scout(app: AppHandle) {
    lobby::open_scout(&app);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_serialize_like_the_frontend_expects() {
        let settings = Settings {
            auto_accept: true,
            scout_provider: "ugg".to_string(),
            auto_open: false,
            reveal_ranked: true,
            launch_at_login: false,
            dodge_lead_ms: 2_000,
            welcomed: true,
        };
        assert_eq!(
            serde_json::to_value(&settings).unwrap(),
            serde_json::json!({
                "autoAccept": true,
                "scoutProvider": "ugg",
                "autoOpen": false,
                "revealRanked": true,
                "launchAtLogin": false,
                "dodgeLeadMs": 2000,
                "welcomed": true,
            })
        );
    }
}
