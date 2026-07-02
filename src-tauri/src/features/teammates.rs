// feature 1: surface champ-select teammates.
//
// during champ select riot can hide teammate names. in normal, draft, and bot
// games the champ-select session carries real puuids, so we resolve them
// directly through the league client. in ranked, "anonymous champ select"
// strips identities from the session, so we fall back to the riot client chat
// service, which still reports the real names (chat has to know who is who).

use std::sync::atomic::Ordering;

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

use crate::features::{set_status, EVENT_PHASE, EVENT_TEAMMATES};
use crate::lcu::client::LcuClient;
use crate::models::{
    ChampSelectPhase, ChampSelectSession, ConnectionStatus, NoticeLevel, Teammate,
};
use crate::state::AppState;

pub async fn handle_session(app: &AppHandle, client: &LcuClient, event_type: &str, data: Value) {
    let state = app.state::<AppState>();

    // a Delete event means champ select ended: reset and hide.
    if event_type == "Delete" {
        state.summoner_cache.write().clear();
        state.teammates.write().clear();
        set_status(app, ConnectionStatus::Connected);
        let _ = app.emit(EVENT_TEAMMATES, Vec::<Teammate>::new());
        hide_window(app);
        return;
    }

    let Ok(session) = serde_json::from_value::<ChampSelectSession>(data) else {
        return;
    };

    // a transitional create/update can arrive with an empty team and no phase
    // before champ select is really underway. ignore it so the window does not
    // pop forward on an empty session.
    if session.my_team.is_empty() && session.timer.phase.is_empty() {
        return;
    }

    // only pop the window forward on the transition into champ select, not on
    // every subsequent session update, so it does not repeatedly steal focus.
    let entering = *state.status.read() != ConnectionStatus::ChampSelect;
    set_status(app, ConnectionStatus::ChampSelect);
    if entering {
        show_window(app);
        // a fresh champ select: allow auto-open and the resolution notice to
        // fire once more.
        state.auto_opened.store(false, Ordering::Relaxed);
        state.notified.store(false, Ordering::Relaxed);
    }

    let _ = app.emit(
        EVENT_PHASE,
        ChampSelectPhase {
            phase: session.timer.phase.clone(),
            time_left_ms: session.timer.adjusted_time_left_in_phase,
        },
    );

    let hidden = session
        .my_team
        .iter()
        .any(|player| player.name_visibility_type == "HIDDEN");
    let region = state.region.read().clone();

    let mut teammates = if hidden {
        if state.reveal_ranked.load(Ordering::Relaxed) {
            resolve_from_chat(app, client, region.as_deref()).await
        } else {
            notify_once(
                app,
                NoticeLevel::Info,
                "ranked reveal is off — turn it on in settings to see hidden teammates",
            );
            Vec::new()
        }
    } else {
        resolve_from_session(app, client, &session, region.as_deref()).await
    };

    // hidden teammates have no cell id, so fall back to ordering by name.
    teammates.sort_by(|a, b| {
        a.cell_id
            .cmp(&b.cell_id)
            .then_with(|| a.game_name.cmp(&b.game_name))
    });

    *state.teammates.write() = teammates.clone();
    maybe_auto_open(app, &teammates, region.as_deref());

    if let Err(error) = app.emit(EVENT_TEAMMATES, &teammates) {
        eprintln!("expose: could not emit teammates: {error}");
    }
}

// opens the scout link automatically the first time a champ select resolves
// teammates, when the auto-open setting is on.
fn maybe_auto_open(app: &AppHandle, teammates: &[Teammate], region: Option<&str>) {
    let state = app.state::<AppState>();
    if teammates.is_empty() || !state.auto_open.load(Ordering::Relaxed) {
        return;
    }
    if state.auto_opened.swap(true, Ordering::Relaxed) {
        return;
    }
    let provider = state.scout_provider.read().clone();
    if let Some(url) = crate::scout::scout_url(&provider, teammates, region) {
        use tauri_plugin_opener::OpenerExt;
        if let Err(error) = app.opener().open_url(url, None::<&str>) {
            eprintln!("expose: could not auto-open scout: {error}");
        }
    }
}

// non-anonymous path: the session has real puuids, so resolve each through the
// league client summoner endpoint, cached for the duration of the session.
async fn resolve_from_session(
    app: &AppHandle,
    client: &LcuClient,
    session: &ChampSelectSession,
    region: Option<&str>,
) -> Vec<Teammate> {
    let state = app.state::<AppState>();
    let provider = state.scout_provider.read().clone();
    let label = crate::scout::provider_label(&provider).to_string();
    let mut teammates = Vec::new();
    for player in &session.my_team {
        if is_hidden_puuid(&player.puuid) {
            continue;
        }

        // a clone out of the cache keeps the lock guard from crossing the await.
        let cached = state.summoner_cache.read().get(&player.puuid).cloned();
        let summoner = match cached {
            Some(summoner) => summoner,
            None => match client.summoner_by_puuid(&player.puuid).await {
                Ok(summoner) => {
                    state
                        .summoner_cache
                        .write()
                        .insert(player.puuid.clone(), summoner.clone());
                    summoner
                }
                Err(error) => {
                    eprintln!(
                        "expose: could not resolve summoner {}: {error}",
                        player.puuid
                    );
                    continue;
                }
            },
        };

        teammates.push(Teammate {
            cell_id: player.cell_id,
            puuid: player.puuid.clone(),
            scout_url: crate::scout::profile_url(
                &provider,
                region,
                &summoner.game_name,
                &summoner.tag_line,
            ),
            scout_label: label.clone(),
            game_name: summoner.game_name,
            tag_line: summoner.tag_line,
            summoner_level: summoner.summoner_level,
            profile_icon_id: summoner.profile_icon_id,
            assigned_position: player.assigned_position.clone(),
            champion_id: player.champion_id,
            champion_pick_intent: player.champion_pick_intent,
        });
    }
    teammates
}

// anonymous path: the session hides identities, so read the champ-select chat
// participants from the riot client, which still report real names. position
// and champion can't be mapped here (the chat has no cell id), so they are left
// empty and the ui simply shows the name.
async fn resolve_from_chat(
    app: &AppHandle,
    client: &LcuClient,
    region: Option<&str>,
) -> Vec<Teammate> {
    let state = app.state::<AppState>();
    let riot = state.riot_client.read().clone();
    let Some(riot) = riot else {
        eprintln!("expose: anonymous champ select but no riot client available");
        notify_once(
            app,
            NoticeLevel::Error,
            "couldn't read teammate names — make sure the Riot client is running",
        );
        return Vec::new();
    };

    let participants = match riot.chat_participants().await {
        Ok(participants) => participants,
        Err(error) => {
            eprintln!("expose: could not read chat participants: {error}");
            notify_once(
                app,
                NoticeLevel::Error,
                "couldn't read teammate names from the Riot client",
            );
            return Vec::new();
        }
    };

    let provider = state.scout_provider.read().clone();
    let label = crate::scout::provider_label(&provider).to_string();
    let mut teammates = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for participant in participants {
        if !participant.cid.contains("champ-select") || participant.game_name.is_empty() {
            continue;
        }

        // the chat reports participants across conversations, so the same person
        // can appear more than once. de-duplicate by puuid to avoid doubled rows
        // and colliding react keys in the ui.
        if !participant.puuid.is_empty() && !seen.insert(participant.puuid.clone()) {
            continue;
        }

        // best effort: the chat gives a real puuid, so try the league summoner
        // endpoint for level and icon. it may be blocked in anonymous mode, in
        // which case we just show the name. cached per session.
        let cached = state.summoner_cache.read().get(&participant.puuid).cloned();
        let summoner = match cached {
            Some(summoner) => Some(summoner),
            None => match client.summoner_by_puuid(&participant.puuid).await {
                Ok(summoner) => {
                    state
                        .summoner_cache
                        .write()
                        .insert(participant.puuid.clone(), summoner.clone());
                    Some(summoner)
                }
                Err(_) => None,
            },
        };
        let (summoner_level, profile_icon_id) = summoner
            .map(|summoner| (summoner.summoner_level, summoner.profile_icon_id))
            .unwrap_or((0, 0));

        teammates.push(Teammate {
            cell_id: 0,
            scout_url: crate::scout::profile_url(
                &provider,
                region,
                &participant.game_name,
                &participant.game_tag,
            ),
            scout_label: label.clone(),
            puuid: participant.puuid,
            game_name: participant.game_name,
            tag_line: participant.game_tag,
            summoner_level,
            profile_icon_id,
            assigned_position: String::new(),
            champion_id: 0,
            champion_pick_intent: 0,
        });
    }
    teammates
}

// riot zeroes the puuid for players whose identity is hidden, and leaves it
// empty for bot or not-yet-filled slots. either way there is nothing to resolve.
fn is_hidden_puuid(puuid: &str) -> bool {
    puuid.is_empty() || puuid.chars().all(|c| c == '0' || c == '-')
}

// emits a resolution notice at most once per champ select, so a missing riot
// client or a disabled reveal does not toast on every session update.
fn notify_once(app: &AppHandle, level: NoticeLevel, message: &str) {
    let state = app.state::<AppState>();
    if !state.notified.swap(true, Ordering::Relaxed) {
        crate::features::notice(app, level, message);
    }
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn hide_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_hidden_or_zeroed_puuids() {
        assert!(is_hidden_puuid(""));
        assert!(is_hidden_puuid("00000000-0000-0000-0000-000000000000"));
        assert!(!is_hidden_puuid("8f3c1a2b-real-puuid"));
    }
}
