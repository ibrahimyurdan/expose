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
use crate::models::{ChampSelectPhase, ChampSelectSession, ConnectionStatus, Teammate};
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

    // only pop the window forward on the transition into champ select, not on
    // every subsequent session update, so it does not repeatedly steal focus.
    let entering = *state.status.read() != ConnectionStatus::ChampSelect;
    set_status(app, ConnectionStatus::ChampSelect);
    if entering {
        show_window(app);
        // a fresh champ select: allow auto-open to fire once more.
        state.auto_opened.store(false, Ordering::Relaxed);
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
        resolve_from_chat(app, client, region.as_deref()).await
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
            opgg_url: build_opgg_url(region, &summoner.game_name, &summoner.tag_line),
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
        return Vec::new();
    };

    let participants = match riot.chat_participants().await {
        Ok(participants) => participants,
        Err(error) => {
            eprintln!("expose: could not read chat participants: {error}");
            return Vec::new();
        }
    };

    let mut teammates = Vec::new();
    for participant in participants {
        if !participant.cid.contains("champ-select") || participant.game_name.is_empty() {
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
            opgg_url: build_opgg_url(region, &participant.game_name, &participant.game_tag),
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

// builds a region-correct op.gg account link. region is the op.gg web slug, for
// example "na" or "euw"; it falls back to "na" only if the client did not
// report one.
fn build_opgg_url(region: Option<&str>, game_name: &str, tag_line: &str) -> String {
    let region = region.filter(|slug| !slug.is_empty()).unwrap_or("na");
    format!(
        "https://www.op.gg/summoners/{region}/{}-{}",
        percent_encode(game_name),
        percent_encode(tag_line)
    )
}

// percent-encodes a single url path segment so riot ids with spaces or other
// characters produce a valid link.
fn percent_encode(segment: &str) -> String {
    let mut encoded = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
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
    fn builds_a_region_correct_opgg_url() {
        assert_eq!(
            build_opgg_url(Some("euw"), "Faker", "KR1"),
            "https://www.op.gg/summoners/euw/Faker-KR1"
        );
    }

    #[test]
    fn falls_back_to_na_when_the_region_is_unknown() {
        assert!(build_opgg_url(None, "Name", "NA1").starts_with("https://www.op.gg/summoners/na/"));
        assert!(
            build_opgg_url(Some(""), "Name", "NA1").starts_with("https://www.op.gg/summoners/na/")
        );
    }

    #[test]
    fn percent_encodes_spaces_and_special_characters() {
        assert_eq!(
            build_opgg_url(Some("na"), "Hide on bush", "NA 1"),
            "https://www.op.gg/summoners/na/Hide%20on%20bush-NA%201"
        );
    }

    #[test]
    fn recognizes_hidden_or_zeroed_puuids() {
        assert!(is_hidden_puuid(""));
        assert!(is_hidden_puuid("00000000-0000-0000-0000-000000000000"));
        assert!(!is_hidden_puuid("8f3c1a2b-real-puuid"));
    }
}
