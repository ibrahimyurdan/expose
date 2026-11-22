// the champ select lobby: who sits in each slot of your team, matched to a real
// account even when riot hides the names, with their recent games and the red
// flags in them. every champ select event from the league client goes
// through handle_session, which keeps one LobbyState per lobby and emits the
// LobbyView built from it. stats are fetched in the background, and the view
// goes out again each time a player's stats arrive.

pub mod aliases;
pub mod flags;
pub mod identity;
pub mod session;
pub mod stats;
pub mod view;

use std::collections::{HashMap, HashSet};
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_opener::OpenerExt;
use tokio::sync::Semaphore;

use crate::features::{self, dodge, set_status, EVENT_LOBBY};
use crate::lcu::client::LcuClient;
use crate::models::{ChatParticipant, ConnectionStatus, NoticeLevel};
use crate::scout;
use crate::state::AppState;
use aliases::AliasBook;
use identity::{
    format_uuid, guess, parse_uuid, resolve_exact, Candidate, Identity, MatchKind, Resolved,
    SlotHint,
};
use session::{is_blank, Acting, Session, SessionPlayer, Stage};
use stats::{fetch_quick, fetch_rest, Fetched, PlayerStats, Position, StatsCache};
use view::{build_view, LobbyView, ScoutContext};

// while a hidden teammate has no exact match, the riot client's chat is read
// again at most this often: quickly while the room fills at the start of the
// lobby, then every couple of seconds.
const CHAT_EARLY_EVERY: Duration = Duration::from_millis(500);
const CHAT_EARLY_FOR: Duration = Duration::from_secs(5);
const CHAT_EVERY: Duration = Duration::from_secs(2);
// how long after the first read an empty chat (the riot client joins the room
// a moment after champ select starts) still counts as filling, before a hidden
// slot stops showing as being matched.
const CHAT_PATIENCE: Duration = Duration::from_secs(10);
// any champ select chat room has this in its conversation id.
const CHAMP_SELECT_ROOM: &str = "champ-select";
// what the toast says when the names in a ranked lobby cannot be read.
const CHAT_UNAVAILABLE: &str = "Couldn't reach the Riot client, so ranked names may stay hidden.";
// stats requests to the league client in flight at once, across all players.
static STATS_REQUESTS: Semaphore = Semaphore::const_new(4);
// the last reason a session could not be read, so the log says it once rather
// than for every event of the lobby.
static SESSION_ERROR: Mutex<String> = Mutex::new(String::new());

// everything expose knows about the lobby it is in. reset whenever a session
// belongs to another lobby, so nothing of the last one leaks into the next.
#[derive(Debug, Default)]
pub struct LobbyState {
    pub game_id: i64,
    pub aliases: AliasBook,
    // who sits in each slot, keyed by player key (see SessionPlayer::key).
    pub identities: HashMap<String, Identity>,
    // the riot client's champ select chat, one entry per player.
    pub candidates: Vec<Candidate>,
    // keyed by puuid, written the one way canonical_puuid writes it.
    pub stats: HashMap<String, PlayerStats>,
    // puuids whose stats are being fetched.
    pub pending: HashSet<String>,
    // the quick half (summoner and rank) of stats still being fetched, keyed
    // like stats. kept apart so the slot guesses only ever see full stats.
    pub quick: HashMap<String, PlayerStats>,
    // the names came from this lobby's own chat room, not the champ select
    // fallback (see lobby_candidates).
    pub candidates_from_room: bool,
    // hidden players the scramble key rules out of the chat (see Resolved).
    pub absent: HashSet<String>,
    // hidden slots still being matched: the chat has not answered yet, or a
    // guess waits for a name's stats.
    pub matching: HashSet<String>,
    // when the chat was last read, and how it went.
    pub participants_fetched_at: Option<Instant>,
    // the first read that came back, for the read pace and the patience.
    pub chat_started: Option<Instant>,
    pub chat_failed: bool,
    // a chat reader runs for this lobby (see read_chat).
    pub chat_reading: bool,
    // the latest session, for the views rebuilt as stats arrive, and when it
    // arrived: the client sends no timer ticks, so its timer ages from then.
    pub session: Option<Session>,
    pub session_at: Option<Instant>,
    // the match counts last written to the log.
    pub logged_summary: String,
    // the stage and acting team last written to the log.
    pub logged_turn: Option<(Stage, Acting)>,
    // the scout site was opened for this lobby.
    pub auto_opened: bool,
    // the player heard that this lobby's names could not be read.
    pub notified: bool,
}

impl LobbyState {
    // forgets everything, for the lobby with this game id.
    pub fn reset(&mut self, game_id: i64) {
        *self = LobbyState {
            game_id,
            ..LobbyState::default()
        };
    }

    // the summoner and rank of a player still being fetched are in. they
    // show straight away; the player stays pending until the rest arrives.
    pub fn record_quick(&mut self, puuid: &str, stats: PlayerStats) {
        if self.pending.contains(puuid) {
            self.quick.insert(puuid.to_string(), stats);
        }
    }

    // a player's full stats are in.
    pub fn record_full(&mut self, puuid: &str, stats: PlayerStats) {
        self.pending.remove(puuid);
        self.quick.remove(puuid);
        self.stats.insert(puuid.to_string(), stats);
    }

    // follows a session to its lobby. a new game id (a requeue, or the first
    // session seen) starts over; true when it did.
    pub fn enter(&mut self, game_id: i64) -> bool {
        if self.game_id == game_id {
            return false;
        }
        self.reset(game_id);
        true
    }
}

pub async fn handle_session(app: &AppHandle, client: &LcuClient, event_type: &str, data: Value) {
    // a Delete event means champ select ended: reset and hide.
    if event_type == "Delete" {
        reset(app);
        set_status(app, ConnectionStatus::Connected);
        hide_window(app);
        return;
    }

    let session = match serde_json::from_value::<Session>(data) {
        Ok(session) => session,
        Err(error) => {
            session_unreadable(&error.to_string());
            return;
        }
    };
    // a transitional create/update can arrive with an empty team and no phase
    // before champ select is really underway. ignore it so the window does not
    // pop forward on an empty session.
    if session.my_team.is_empty() && session.timer.phase.is_empty() {
        return;
    }

    let state = app.state::<AppState>();
    // only pop the window forward on the transition into champ select, not on
    // every subsequent session update, so it does not repeatedly steal focus.
    let entering = *state.status.read() != ConnectionStatus::ChampSelect;
    set_status(app, ConnectionStatus::ChampSelect);
    if entering {
        show_window(app);
    }

    let reveal = state.reveal_ranked.load(Ordering::Relaxed);
    let start_reader = {
        let mut lobby = state.lobby.lock();
        lobby.enter(session.game_id);
        lobby.session = Some(session.clone());
        lobby.session_at = Some(Instant::now());
        resolve(&mut lobby, reveal);
        let start = reveal && !lobby.chat_reading && chat_due(&lobby, Instant::now());
        if start {
            lobby.chat_reading = true;
        }
        start
    };
    // an armed last-second dodge follows the lobby's timer.
    dodge::on_session(app, &session, Duration::ZERO);
    // slots, aliases and whatever is already known show straight away; the
    // chat can only add to them.
    update(app, client);
    if start_reader {
        let app = app.clone();
        let client = client.clone();
        let game_id = session.game_id;
        tauri::async_runtime::spawn(async move {
            read_chat(&app, &client, game_id).await;
        });
    }
    auto_open(app);
}

// the session could not be read, so this event changes nothing. said once per
// reason; serde's message names the expected type, never a player.
fn session_unreadable(reason: &str) {
    let mut last = SESSION_ERROR
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if *last != reason {
        log::warn!("could not read the champ select session: {reason}");
        reason.clone_into(&mut last);
    }
}

// the league client went away or champ select ended: disarm the last-second
// dodge, forget the lobby and tell the frontend it is gone.
pub fn reset(app: &AppHandle) {
    dodge::disarm(app);
    let state = app.state::<AppState>();
    let mut lobby = state.lobby.lock();
    lobby.reset(0);
    *state.lobby_view.write() = None;
    if let Err(error) = app.emit(EVENT_LOBBY, None::<LobbyView>) {
        log::warn!("could not emit the lobby: {error}");
    }
}

// shows a change made outside a session update (a setting, the last-second
// dodge) in the current lobby straight away.
pub fn refresh(app: &AppHandle) {
    let state = app.state::<AppState>();
    let reveal = state.reveal_ranked.load(Ordering::Relaxed);
    resolve(&mut state.lobby.lock(), reveal);
    publish(app);
}

// opens the chosen scout site with everyone in the lobby in one tab.
pub fn open_scout(app: &AppHandle) {
    let state = app.state::<AppState>();
    let Some(view) = state.lobby_view.read().clone() else {
        return;
    };
    let provider = state.scout_provider.read().clone();
    let region = state.region.read().clone();
    if let Some(url) = scout::scout_url(&provider, &view.names(), region.as_deref()) {
        if let Err(error) = app.opener().open_url(url, None::<&str>) {
            log::warn!("could not open the scout site: {error}");
        }
    }
}

// resolves the lobby again, starts fetching any stats it now needs and emits
// the view.
fn update(app: &AppHandle, client: &LcuClient) {
    let state = app.state::<AppState>();
    let reveal = state.reveal_ranked.load(Ordering::Relaxed);
    let (game_id, fetch) = {
        let mut lobby = state.lobby.lock();
        resolve(&mut lobby, reveal);
        let fetch = take_from_cache(&mut lobby, &state.stats_cache.lock(), Instant::now());
        // cached stats can complete the names a guess waits for.
        resolve(&mut lobby, reveal);
        (lobby.game_id, fetch)
    };
    for puuid in fetch {
        let app = app.clone();
        let client = client.clone();
        tauri::async_runtime::spawn(async move {
            // the summoner and rank answer fast, so they show first.
            let quick = fetch_quick(&client, &puuid, &STATS_REQUESTS).await;
            quick_arrived(&app, game_id, &puuid, quick.stats());
            let fetched = fetch_rest(&client, &puuid, &STATS_REQUESTS, quick).await;
            stats_arrived(&app, game_id, puuid, fetched);
        });
    }
    publish(app);
}

// one player's summoner and rank are in: show them while the rest loads.
fn quick_arrived(app: &AppHandle, game_id: i64, puuid: &str, stats: PlayerStats) {
    let state = app.state::<AppState>();
    {
        let mut lobby = state.lobby.lock();
        if lobby.game_id != game_id {
            return;
        }
        lobby.record_quick(puuid, stats);
    }
    publish(app);
}

// one player's stats are in: keep them, guess again if that was the last
// name a guess waited for, and show them. only a fetch where every request
// went through is reused in later lobbies; one with a gap stays in this lobby
// alone, so a passing failure is not shown for the next ten minutes.
fn stats_arrived(app: &AppHandle, game_id: i64, puuid: String, fetched: Fetched) {
    let Fetched { stats, complete } = fetched;
    let state = app.state::<AppState>();
    if complete {
        state
            .stats_cache
            .lock()
            .insert(puuid.clone(), stats.clone(), Instant::now());
    }
    let reveal = state.reveal_ranked.load(Ordering::Relaxed);
    {
        let mut lobby = state.lobby.lock();
        // the lobby they were fetched for is over; the cache keeps them.
        if lobby.game_id != game_id {
            return;
        }
        lobby.record_full(&puuid, stats);
        resolve(&mut lobby, reveal);
    }
    publish(app);
}

// builds the view from the lobby as it stands, keeps it for get_lobby and
// emits it. the lobby stays locked until the view is out, so two views
// built at nearly the same time cannot reach the frontend in the wrong order.
fn publish(app: &AppHandle) {
    let state = app.state::<AppState>();
    let scout = ScoutContext {
        provider: state.scout_provider.read().clone(),
        region: state.region.read().clone(),
    };
    let champion_name = |champion_id: i64| {
        state
            .champions
            .read()
            .as_ref()
            .and_then(|data| data.champions.get(&champion_id))
            .map(|entry| entry.name.clone())
    };

    let mut lobby = state.lobby.lock();
    let Some(session) = lobby.session.take() else {
        return;
    };
    let dodge = state.dodge.view(session.game_id);
    let view = build_view(
        &session,
        &mut lobby,
        &champion_name,
        &scout,
        dodge,
        now_ms(),
    );
    lobby.session = Some(session);
    log_matches(&mut lobby, view.unmatched.len());
    log_turn(&mut lobby, view.stage, view.acting);
    if !should_publish(state.lobby_view.read().as_ref(), &view) {
        return;
    }
    *state.lobby_view.write() = Some(view.clone());
    if let Err(error) = app.emit(EVENT_LOBBY, &view) {
        log::warn!("could not emit the lobby: {error}");
    }
}

// a view equal to the one the frontend already has changes nothing on screen,
// so it is not sent again (stats or settings often rebuild the same view).
fn should_publish(previous: Option<&LobbyView>, next: &LobbyView) -> bool {
    previous != Some(next)
}

// works out who sits in each slot: the exact steps first, then a guess for
// the hidden slots still open, once every name left over has stats to guess
// from. an exact match always wins, since the guess only gets what the exact
// steps leave. while a new name's stats are on the way the last guess stands,
// so rows do not blink as names trickle in. with ranked reveal off, the chat
// is forgotten and hidden players stay hidden.
fn resolve(lobby: &mut LobbyState, reveal: bool) {
    if !reveal {
        lobby.candidates.clear();
        lobby.candidates_from_room = false;
    }
    let Some(session) = &lobby.session else {
        lobby.identities.clear();
        lobby.absent.clear();
        lobby.matching.clear();
        return;
    };
    let Resolved {
        mut identities,
        absent,
    } = resolve_exact(session, &lobby.candidates);

    let used: HashSet<String> = identities
        .values()
        .filter_map(|identity| canonical_puuid(&identity.puuid))
        .collect();
    let open: Vec<SlotHint> = session
        .team_in_slot_order()
        .into_iter()
        .filter(|player| player.is_hidden())
        .filter(|player| {
            let key = player.key();
            !identities.contains_key(&key) && !absent.contains(&key)
        })
        .map(slot_hint)
        .collect();
    // none until every free name has its stats.
    let names: Option<Vec<(Candidate, PlayerStats)>> = lobby
        .candidates
        .iter()
        .filter(|candidate| !used.contains(&candidate.puuid))
        .map(|candidate| {
            let stats = lobby.stats.get(&candidate.puuid)?;
            Some((candidate.clone(), stats.clone()))
        })
        .collect();
    let waiting = names.is_none();
    match names {
        Some(names) => identities.extend(guess(&open, &names)),
        None => {
            let listed: HashSet<&str> = lobby
                .candidates
                .iter()
                .map(|candidate| candidate.puuid.as_str())
                .collect();
            for slot in &open {
                let last = lobby
                    .identities
                    .get(&slot.key)
                    .filter(|last| last.kind == MatchKind::Guess)
                    .filter(|last| {
                        canonical_puuid(&last.puuid).is_some_and(|puuid| {
                            listed.contains(puuid.as_str()) && !used.contains(&puuid)
                        })
                    });
                if let Some(last) = last {
                    identities.insert(slot.key.clone(), last.clone());
                }
            }
        }
    }
    // the chat has not named anyone yet but may still, or a guess is waiting.
    let names_due = lobby.candidates.is_empty()
        && !lobby.chat_failed
        && lobby
            .chat_started
            .is_none_or(|started| started.elapsed() < CHAT_PATIENCE);
    lobby.matching = if reveal && (names_due || waiting) {
        open.iter()
            .map(|slot| slot.key.clone())
            .filter(|key| !identities.contains_key(key))
            .collect()
    } else {
        HashSet::new()
    };
    lobby.identities = identities;
    lobby.absent = absent;
}

fn slot_hint(player: &SessionPlayer) -> SlotHint {
    SlotHint {
        key: player.key(),
        position: Position::from_assigned(&player.assigned_position),
        is_autofilled: player.is_autofilled,
        spells: (player.spell1_id, player.spell2_id),
        champion_id: player.champion(),
    }
}

// the chat is worth reading while a hidden teammate has no exact match (a
// guess may still turn exact), at the pace chat_interval sets.
fn chat_due(lobby: &LobbyState, now: Instant) -> bool {
    let every = chat_interval(chat_age(lobby, now));
    let rested = lobby
        .participants_fetched_at
        .is_none_or(|at| now.saturating_duration_since(at) >= every);
    needs_chat(lobby) && rested
}

// how long since the chat first answered in this lobby, zero before it has.
fn chat_age(lobby: &LobbyState, now: Instant) -> Duration {
    lobby.chat_started.map_or(Duration::ZERO, |started| {
        now.saturating_duration_since(started)
    })
}

// quick reads while the room fills at the start of the lobby, then slower.
fn chat_interval(since_first_read: Duration) -> Duration {
    if since_first_read < CHAT_EARLY_FOR {
        CHAT_EARLY_EVERY
    } else {
        CHAT_EVERY
    }
}

fn needs_chat(lobby: &LobbyState) -> bool {
    let Some(session) = &lobby.session else {
        return false;
    };
    session
        .my_team
        .iter()
        .filter(|player| player.is_hidden())
        .any(|player| {
            lobby
                .identities
                .get(&player.key())
                .is_none_or(|identity| identity.kind == MatchKind::Guess)
        })
}

// reads the lobby's chat names every couple of seconds while a hidden teammate
// has no exact match. one reader runs per lobby, off the league client's event
// loop, so a slow riot client never holds up champ select events.
async fn read_chat(app: &AppHandle, client: &LcuClient, game_id: i64) {
    loop {
        let chat_id = {
            let state = app.state::<AppState>();
            let lobby = state.lobby.lock();
            if lobby.game_id != game_id {
                return;
            }
            lobby
                .session
                .as_ref()
                .map(|session| session.chat_details.multi_user_chat_id.clone())
                .unwrap_or_default()
        };
        let read = read_chat_names(app, &chat_id).await;
        if !chat_arrived(app, client, game_id, read) {
            return;
        }
        let every = {
            let state = app.state::<AppState>();
            let lobby = state.lobby.lock();
            chat_interval(chat_age(&lobby, Instant::now()))
        };
        tokio::time::sleep(every).await;
    }
}

// one chat read is back: add its names to the lobby and show what changed.
// true while another read is worth it.
fn chat_arrived(
    app: &AppHandle,
    client: &LcuClient,
    game_id: i64,
    read: Option<(Vec<Candidate>, bool)>,
) -> bool {
    let state = app.state::<AppState>();
    let reveal = state.reveal_ranked.load(Ordering::Relaxed);
    let (changed, again) = {
        let mut lobby = state.lobby.lock();
        // the lobby is over; the next one runs its own reader.
        if lobby.game_id != game_id {
            return false;
        }
        lobby.participants_fetched_at = Some(Instant::now());
        let before = lobby.matching.clone();
        let mut changed = false;
        if reveal {
            match read {
                Some((names, from_room)) => {
                    lobby.chat_failed = false;
                    lobby.chat_started.get_or_insert_with(Instant::now);
                    changed = merge_candidates(&mut lobby, names, from_room);
                }
                None => lobby.chat_failed = true,
            }
        }
        resolve(&mut lobby, reveal);
        changed |= lobby.matching != before;
        let again = reveal && needs_chat(&lobby);
        if !again {
            lobby.chat_reading = false;
        }
        (changed, again)
    };
    if changed {
        update(app, client);
        auto_open(app);
    }
    again
}

// adds a read's names to the lobby's, and true when that changed them. names
// stay once seen: a player dropping out of one read (a presence flap) must not
// undo an exact match. the lobby's own room outranks the champ select fallback,
// whose names may come from an older lobby, so the first read that finds the
// room replaces them.
fn merge_candidates(lobby: &mut LobbyState, names: Vec<Candidate>, from_room: bool) -> bool {
    if names.is_empty() || (lobby.candidates_from_room && !from_room) {
        return false;
    }
    if from_room && !lobby.candidates_from_room {
        lobby.candidates_from_room = true;
        let changed = lobby.candidates != names;
        lobby.candidates = names;
        return changed;
    }
    let mut changed = false;
    for name in names {
        if !lobby
            .candidates
            .iter()
            .any(|known| known.puuid == name.puuid)
        {
            lobby.candidates.push(name);
            changed = true;
        }
    }
    changed
}

// the names in this lobby's chat as the riot client lists them, and whether
// they came from the lobby's own room; none when the chat could not be read
// (which the player hears about once per lobby).
async fn read_chat_names(app: &AppHandle, chat_id: &str) -> Option<(Vec<Candidate>, bool)> {
    let riot = app.state::<AppState>().riot_client.read().clone();
    let Some(riot) = riot else {
        chat_unavailable(app, "the riot client was not found");
        return None;
    };
    match riot.chat_participants().await {
        Ok(participants) => Some(lobby_candidates(&participants, chat_id)),
        Err(error) => {
            chat_unavailable(app, &error.to_string());
            None
        }
    }
}

fn chat_unavailable(app: &AppHandle, reason: &str) {
    let first = {
        let state = app.state::<AppState>();
        let mut lobby = state.lobby.lock();
        !std::mem::replace(&mut lobby.notified, true)
    };
    if first {
        log::warn!("could not read the champ select chat: {reason}");
        features::notice(app, NoticeLevel::Error, CHAT_UNAVAILABLE);
    }
}

// the chat participants in this lobby's champ select room, once each, and
// true when that room was found. the room is the conversation whose id holds
// the session's chat id, or any champ select room when none does. a
// participant without a name or a real puuid is left out, since the riot
// client is not showing who they are.
fn lobby_candidates(participants: &[ChatParticipant], chat_id: &str) -> (Vec<Candidate>, bool) {
    let lobby_room = !chat_id.is_empty()
        && participants
            .iter()
            .any(|participant| participant.cid.contains(chat_id));
    let room = if lobby_room {
        chat_id
    } else {
        CHAMP_SELECT_ROOM
    };
    let mut seen = HashSet::new();
    let names = participants
        .iter()
        .filter(|participant| participant.cid.contains(room))
        .filter(|participant| !participant.game_name.is_empty() && !is_blank(&participant.puuid))
        .filter_map(|participant| {
            Some(Candidate {
                puuid: canonical_puuid(&participant.puuid)?,
                game_name: participant.game_name.clone(),
                tag_line: participant.game_tag.clone(),
            })
        })
        .filter(|candidate| seen.insert(candidate.puuid.clone()))
        .collect();
    (names, lobby_room)
}

// moves cached stats into the lobby for every matched player and every chat
// name (a guess needs their stats too), and marks the rest pending. returns
// the puuids to fetch; one already stored or on the way is never fetched again.
fn take_from_cache(lobby: &mut LobbyState, cache: &StatsCache, now: Instant) -> Vec<String> {
    let matched = lobby
        .identities
        .values()
        .filter_map(|identity| canonical_puuid(&identity.puuid));
    let listed = lobby
        .candidates
        .iter()
        .filter_map(|candidate| canonical_puuid(&candidate.puuid));
    let mut seen = HashSet::new();
    let wanted: Vec<String> = matched
        .chain(listed)
        .filter(|puuid| seen.insert(puuid.clone()))
        .filter(|puuid| !lobby.stats.contains_key(puuid) && !lobby.pending.contains(puuid))
        .collect();

    let mut fetch = Vec::new();
    for puuid in wanted {
        match cache.get(&puuid, now) {
            Some(stats) => {
                lobby.stats.insert(puuid, stats);
            }
            None => {
                lobby.pending.insert(puuid.clone());
                fetch.push(puuid);
            }
        }
    }
    fetch
}

// opens the scout site once per lobby when the setting is on, as soon as a
// teammate has a name.
fn auto_open(app: &AppHandle) {
    let state = app.state::<AppState>();
    if !state.auto_open.load(Ordering::Relaxed) {
        return;
    }
    let named = state.lobby_view.read().as_ref().is_some_and(|view| {
        !view.unmatched.is_empty()
            || view.players.iter().any(|player| {
                !player.is_self
                    && player
                        .identity
                        .as_ref()
                        .is_some_and(|identity| !identity.game_name.is_empty())
            })
    });
    if !named || std::mem::replace(&mut state.lobby.lock().auto_opened, true) {
        return;
    }
    open_scout(app);
}

// writes how each player was matched to the log whenever that changes, so a
// real lobby shows which step worked. never names or ids.
fn log_matches(lobby: &mut LobbyState, unmatched: usize) {
    let line = match_counts(&lobby.identities, unmatched);
    if line != lobby.logged_summary {
        log::info!("{line}");
        lobby.logged_summary = line;
    }
}

// writes the champ select stage and which team is on a turn whenever they
// change, so a real lobby shows that the turns are read right.
fn log_turn(lobby: &mut LobbyState, stage: Stage, acting: Acting) {
    if lobby.logged_turn != Some((stage, acting)) {
        log::info!("lobby: stage={} acting={}", stage.as_str(), acting.as_str());
        lobby.logged_turn = Some((stage, acting));
    }
}

fn match_counts(identities: &HashMap<String, Identity>, unmatched: usize) -> String {
    let count = |kind: MatchKind| {
        identities
            .values()
            .filter(|identity| identity.kind == kind)
            .count()
    };
    format!(
        "lobby: own={} visible={} key={} elimination={} guess={} unmatched={unmatched}",
        count(MatchKind::Own),
        count(MatchKind::Visible),
        count(MatchKind::Key),
        count(MatchKind::Elimination),
        count(MatchKind::Guess),
    )
}

// one spelling per puuid, lowercase and dashed, so a player is stored and
// fetched once however a client capitalizes them. none for anything that is
// not a puuid.
fn canonical_puuid(puuid: &str) -> Option<String> {
    parse_uuid(puuid).map(format_uuid)
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| {
            i64::try_from(since.as_millis()).unwrap_or(i64::MAX)
        })
}

// pops the panel forward as champ select begins. windows only lets a background
// app take focus by faking an alt keypress, which would cut into typing in the
// client, so there it is only shown; the panel is always on top anyway.
fn show_window(app: &AppHandle) {
    crate::reveal_main_window(app, cfg!(not(target_os = "windows")));
}

fn hide_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
        #[cfg(target_os = "windows")]
        crate::window::set_memory_target(&window, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lobby::identity::Confidence;
    use crate::lobby::stats::{GameSummary, PlayerHistory};

    const RANKED: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/session_ranked.json"
    ));

    // the players in session_ranked.json by key: you by your puuid, the four
    // hidden teammates by their obfuscated puuids.
    const YOU: &str = "d45f9fb2-21da-4b64-954b-261f303c8054";
    const TOP: &str = "8c29b6f9-de1a-44c1-ab55-d6ac857a23e0";
    const JUNGLE: &str = "a1490aab-4a71-4b41-befd-50f2d7c07340";
    const BOTTOM: &str = "6dd19584-53a6-4f75-948c-88ab38d40f21";
    const SUPPORT: &str = "f355dab7-17a7-41a5-908d-73bcf648cecb";
    // made-up real puuids for names in the chat, and one for a stranger.
    const REAL: [&str; 4] = [
        "5b0e7c1a-93d4-4f62-8a1e-2c7d9f04b6a3",
        "c7f2a915-0d6b-4e38-b4c1-7a5e3d92f0b8",
        "1e9d4b70-6a2c-4c85-9f13-b8e0a6c4d572",
        "9a4c2e81-7b35-4d06-a2f9-e61b8d3c0f47",
    ];
    const STRANGER: &str = "e83b5f0d-2c94-4a71-86de-0f7a9b1c3e25";
    // a random-looking scramble key.
    const KEY: u128 = 0x84cc_8700_e153_8ee5_3421_f8cd_18fb_2784;

    fn ranked() -> Session {
        serde_json::from_str(RANKED).expect("the fixture should parse")
    }

    fn lobby(session: Session) -> LobbyState {
        LobbyState {
            game_id: session.game_id,
            session: Some(session),
            ..LobbyState::default()
        }
    }

    #[test]
    fn reads_fast_at_first_then_slower() {
        assert_eq!(chat_interval(Duration::ZERO), Duration::from_millis(500));
        assert_eq!(
            chat_interval(Duration::from_secs(1)),
            Duration::from_millis(500)
        );
        assert_eq!(
            chat_interval(Duration::from_secs(5)),
            Duration::from_secs(2)
        );
        assert_eq!(
            chat_interval(Duration::from_secs(6)),
            Duration::from_secs(2)
        );
    }

    #[test]
    fn ranks_show_before_history() {
        let session = ranked();
        let mut state = lobby(ranked());
        state.identities.insert(
            TOP.to_string(),
            identity(REAL[0], "Name One", MatchKind::Key),
        );
        state.pending.insert(REAL[0].to_string());
        let scout = ScoutContext {
            provider: "opgg".to_string(),
            region: None,
        };
        let view_of = |state: &mut LobbyState| {
            let dodge = view::DodgeView {
                armed: false,
                lead_ms: 1500,
            };
            let view = build_view(&session, state, &|_| None, &scout, dodge, 0);
            view.players
                .into_iter()
                .find(|player| player.key == TOP)
                .expect("the top laner is in the lobby")
        };

        // the summoner and rank are in, the history is still on the way.
        let quick = PlayerStats {
            level: Some(120),
            rank: Some(stats::Rank {
                tier: "GOLD".to_string(),
                division: "II".to_string(),
                lp: 54,
                wins: 10,
                losses: 8,
            }),
            ..PlayerStats::default()
        };
        state.record_quick(REAL[0], quick);
        let top = view_of(&mut state);
        assert!(top.stats_pending);
        let stats = top.stats.expect("the rank shows already");
        assert_eq!(stats.rank.expect("a rank").tier, "GOLD");
        assert!(!stats.history_loaded);
        // flags wait for the history, and the slot guesses never see half
        // the stats.
        assert!(top.flags.is_empty());
        assert!(state.stats.is_empty());

        state.record_full(REAL[0], played(Position::Top, 266, (4, 12)));
        let top = view_of(&mut state);
        assert!(!top.stats_pending);
        assert!(top.stats.expect("stats").history_loaded);
        assert!(state.quick.is_empty());
    }

    #[test]
    fn identical_views_are_not_sent_again() {
        let session = ranked();
        let mut state = lobby(ranked());
        let scout = ScoutContext {
            provider: "opgg".to_string(),
            region: None,
        };
        let dodge = view::DodgeView {
            armed: false,
            lead_ms: 1500,
        };
        let view = build_view(&session, &mut state, &|_| None, &scout, dodge, 0);

        assert!(should_publish(None, &view));
        assert!(!should_publish(Some(&view), &view.clone()));
        let mut later = view.clone();
        later.time_left_ms -= 1000;
        assert!(should_publish(Some(&view), &later));
    }

    fn candidate(puuid: &str, game_name: &str) -> Candidate {
        Candidate {
            puuid: puuid.to_string(),
            game_name: game_name.to_string(),
            tag_line: "NA1".to_string(),
        }
    }

    fn identity(puuid: &str, game_name: &str, kind: MatchKind) -> Identity {
        Identity {
            puuid: puuid.to_string(),
            game_name: game_name.to_string(),
            tag_line: "NA1".to_string(),
            kind,
            confidence: Confidence::High,
        }
    }

    fn participant(cid: &str, puuid: &str, game_name: &str) -> ChatParticipant {
        ChatParticipant {
            cid: cid.to_string(),
            game_name: game_name.to_string(),
            game_tag: "NA1".to_string(),
            puuid: puuid.to_string(),
        }
    }

    // ten won games in one role, on one champion, with one pair of spells.
    fn played(position: Position, champion_id: i64, spells: (i64, i64)) -> PlayerStats {
        let game = GameSummary {
            champion_id,
            win: true,
            position: Some(position),
            spells,
            ..GameSummary::default()
        };
        PlayerStats {
            history: Some(PlayerHistory {
                games: vec![game; 10],
            }),
            ..PlayerStats::default()
        }
    }

    // how each slot was matched, in slot order.
    fn kinds(state: &LobbyState) -> Vec<Option<MatchKind>> {
        let session = state.session.as_ref().expect("a session");
        session
            .team_in_slot_order()
            .iter()
            .map(|player| {
                state
                    .identities
                    .get(&player.key())
                    .map(|identity| identity.kind)
            })
            .collect()
    }

    // who each slot was matched to, in slot order.
    fn names(state: &LobbyState) -> Vec<Option<&str>> {
        let session = state.session.as_ref().expect("a session");
        session
            .team_in_slot_order()
            .iter()
            .map(|player| {
                state
                    .identities
                    .get(&player.key())
                    .map(|identity| identity.game_name.as_str())
            })
            .collect()
    }

    #[test]
    fn reads_the_lobby_chat_room() {
        let chat_id = "c1~54a43c1848168346a1f1d305e999e202";
        let room = format!("{chat_id}@champ-select.na1.pvp.net");
        let participants = [
            participant(&room, &REAL[0].to_uppercase(), "Name One"),
            // listed again, as the riot client does for every conversation
            // a player is in.
            participant(&room, REAL[0], "Name One"),
            participant(&room, REAL[1], "Name Two"),
            // without a name, or without a real puuid, nobody is shown.
            participant(&room, REAL[2], ""),
            participant(&room, "", "Name Four"),
            participant(&room, "00000000-0000-0000-0000-000000000000", "Name Five"),
            participant(&room, "not-a-puuid", "Name Six"),
            // a friend in another conversation, and an older champ select.
            participant("friend@pvp.net", REAL[3], "Friend"),
            participant("c1~0ld@champ-select.na1.pvp.net", STRANGER, "Stranger"),
        ];
        // one entry per player, puuids written the one way, from the room.
        assert_eq!(
            lobby_candidates(&participants, chat_id),
            (
                vec![
                    candidate(REAL[0], "Name One"),
                    candidate(REAL[1], "Name Two")
                ],
                true
            )
        );

        // when no room carries the chat id, any champ select room does.
        let names = |(candidates, from_room): (Vec<Candidate>, bool)| -> Vec<String> {
            assert!(!from_room, "the lobby's room is not in this read");
            candidates
                .into_iter()
                .map(|candidate| candidate.game_name)
                .collect()
        };
        assert_eq!(
            names(lobby_candidates(&participants, "c1~somewhere-else")),
            ["Name One", "Name Two", "Stranger"]
        );
        assert_eq!(
            names(lobby_candidates(&participants, "")),
            ["Name One", "Name Two", "Stranger"]
        );
        assert!(lobby_candidates(&[], chat_id).0.is_empty());
    }

    #[test]
    fn guesses_wait_for_every_name_to_have_stats() {
        // the fixture's obfuscated puuids share no key with these names, so
        // only a guess can place them.
        let mut state = lobby(ranked());
        state.candidates = vec![
            candidate(YOU, "Example Player"),
            candidate(REAL[0], "Jungle Main"),
            candidate(REAL[1], "Top Main"),
        ];
        resolve(&mut state, true);
        assert_eq!(
            kinds(&state),
            [None, None, Some(MatchKind::Own), None, None]
        );

        // one name has stats, the other is still loading: no guess yet.
        state
            .stats
            .insert(REAL[0].to_string(), played(Position::Jungle, 64, (11, 4)));
        resolve(&mut state, true);
        assert_eq!(
            kinds(&state),
            [None, None, Some(MatchKind::Own), None, None]
        );

        state
            .stats
            .insert(REAL[1].to_string(), played(Position::Top, 266, (12, 4)));
        resolve(&mut state, true);
        assert_eq!(
            kinds(&state),
            [
                Some(MatchKind::Guess),
                Some(MatchKind::Guess),
                Some(MatchKind::Own),
                None,
                None,
            ]
        );
        assert_eq!(
            names(&state),
            [
                Some("Top Main"),
                Some("Jungle Main"),
                Some("Example Player"),
                None,
                None,
            ]
        );

        // with reveal off, hidden players are left alone and the chat names
        // are forgotten.
        resolve(&mut state, false);
        assert_eq!(
            kinds(&state),
            [None, None, Some(MatchKind::Own), None, None]
        );
        assert!(state.candidates.is_empty());
    }

    #[test]
    fn never_guesses_a_slot_the_key_rules_out() {
        // riot's scramble: each hidden teammate's obfuscated puuid is their
        // real one xor the lobby's key. the support hides their name, so the
        // chat lists the other three, and a stranger who is not in the lobby.
        let mut session = ranked();
        let hidden = session
            .my_team
            .iter_mut()
            .filter(|player| player.is_hidden());
        for (player, real) in hidden.zip(REAL) {
            let real = parse_uuid(real).expect("a puuid");
            player.obfuscated_puuid = format_uuid(real ^ KEY);
        }
        let mut state = lobby(session);
        state.candidates = vec![
            candidate(YOU, "Example Player"),
            candidate(REAL[0], "Name One"),
            candidate(REAL[1], "Name Two"),
            candidate(REAL[2], "Name Three"),
            candidate(STRANGER, "Stranger"),
        ];
        // everyone's stats are in, the stranger's a perfect fit for support.
        for candidate in &state.candidates {
            let stats = played(Position::Utility, 412, (4, 3));
            state.stats.insert(candidate.puuid.clone(), stats);
        }
        resolve(&mut state, true);
        assert_eq!(
            kinds(&state),
            [
                Some(MatchKind::Key),
                Some(MatchKind::Key),
                Some(MatchKind::Own),
                Some(MatchKind::Key),
                // the key says the support is nobody the chat lists.
                None,
            ]
        );
        assert_eq!(names(&state)[3], Some("Name Three"));
        // so the support's row says the name stays hidden.
        let support = state
            .session
            .as_ref()
            .expect("a session")
            .team_in_slot_order()[4]
            .key();
        assert_eq!(state.absent, HashSet::from([support]));
        assert!(state.matching.is_empty());
    }

    #[test]
    fn keeps_the_last_guess_while_a_new_name_loads() {
        let mut state = lobby(ranked());
        state.candidates = vec![
            candidate(REAL[0], "Jungle Main"),
            candidate(REAL[1], "Top Main"),
        ];
        state
            .stats
            .insert(REAL[0].to_string(), played(Position::Jungle, 64, (11, 4)));
        state
            .stats
            .insert(REAL[1].to_string(), played(Position::Top, 266, (12, 4)));
        resolve(&mut state, true);
        let owned = |state: &LobbyState| -> Vec<Option<String>> {
            names(state)
                .into_iter()
                .map(|name| name.map(str::to_string))
                .collect()
        };
        let guessed = owned(&state);
        assert_eq!(guessed[0].as_deref(), Some("Top Main"));
        assert_eq!(guessed[1].as_deref(), Some("Jungle Main"));

        // a third name shows up in the chat; its stats are on the way. the
        // guess stands meanwhile, and only the open slots show as matching.
        state.candidates.push(candidate(REAL[2], "Bot Main"));
        resolve(&mut state, true);
        assert_eq!(owned(&state), guessed);
        let open: HashSet<String> = [BOTTOM, SUPPORT].map(String::from).into();
        assert_eq!(state.matching, open);

        // the stats arrive: everyone is guessed again, and nothing is pending.
        state
            .stats
            .insert(REAL[2].to_string(), played(Position::Bottom, 222, (7, 4)));
        resolve(&mut state, true);
        assert_eq!(names(&state)[3], Some("Bot Main"));
        assert!(state.matching.is_empty());
    }

    #[test]
    fn shows_matching_until_there_is_nothing_left_to_try() {
        let hidden: HashSet<String> = [TOP, JUNGLE, BOTTOM, SUPPORT].map(String::from).into();
        let mut state = lobby(ranked());
        // champ select just started and the chat has not been read.
        resolve(&mut state, true);
        assert_eq!(state.matching, hidden);
        // with ranked reveal off nothing is being matched.
        resolve(&mut state, false);
        assert!(state.matching.is_empty());

        // empty reads while the riot client joins the room: still matching,
        // until it has had time to.
        let now = Instant::now();
        state.chat_started = now.checked_sub(Duration::from_secs(9));
        resolve(&mut state, true);
        assert_eq!(state.matching, hidden);
        state.chat_started = now.checked_sub(Duration::from_secs(11));
        resolve(&mut state, true);
        assert!(state.matching.is_empty());

        // a failed read gives up at once (the player gets a toast).
        state.chat_started = None;
        state.chat_failed = true;
        resolve(&mut state, true);
        assert!(state.matching.is_empty());
    }

    #[test]
    fn chat_names_stay_once_seen() {
        let mut state = lobby(ranked());
        // the champ select fallback found a room, maybe an older lobby's.
        let stray = vec![candidate(STRANGER, "Stranger")];
        assert!(merge_candidates(&mut state, stray, false));
        // the lobby's own room replaces those names.
        let room = vec![
            candidate(REAL[0], "Name One"),
            candidate(REAL[1], "Name Two"),
        ];
        assert!(merge_candidates(&mut state, room.clone(), true));
        assert_eq!(state.candidates, room);
        // a read missing a player (a presence flap) keeps them.
        assert!(!merge_candidates(
            &mut state,
            vec![candidate(REAL[0], "Name One")],
            true
        ));
        assert_eq!(state.candidates, room);
        // a newcomer is added, an empty read and the fallback change nothing.
        assert!(merge_candidates(
            &mut state,
            vec![candidate(REAL[2], "Name Three")],
            true
        ));
        assert!(!merge_candidates(&mut state, Vec::new(), true));
        assert!(!merge_candidates(
            &mut state,
            vec![candidate(STRANGER, "Stranger")],
            false
        ));
        let listed: Vec<&str> = state
            .candidates
            .iter()
            .map(|candidate| candidate.game_name.as_str())
            .collect();
        assert_eq!(listed, ["Name One", "Name Two", "Name Three"]);
    }

    #[test]
    fn reads_the_chat_until_every_hidden_teammate_is_matched() {
        let start = Instant::now();
        let mut state = lobby(ranked());
        // nobody hidden is matched yet and the chat was never read.
        assert!(chat_due(&state, start));
        // early in the lobby the chat is read every half second.
        state.chat_started = Some(start);
        state.participants_fetched_at = Some(start);
        assert!(!chat_due(&state, start + Duration::from_millis(300)));
        assert!(chat_due(&state, start + Duration::from_millis(500)));
        // once the room has settled, every couple of seconds.
        let settled = start + Duration::from_secs(10);
        state.participants_fetched_at = Some(settled);
        assert!(!chat_due(&state, settled + Duration::from_secs(1)));
        assert!(chat_due(&state, settled + Duration::from_secs(2)));

        // a guess is not a match, so the chat is still worth reading.
        let later = settled + Duration::from_secs(5);
        for (key, real) in [TOP, JUNGLE, BOTTOM, SUPPORT].into_iter().zip(REAL) {
            let guessed = identity(real, "Name", MatchKind::Guess);
            state.identities.insert(key.to_string(), guessed);
        }
        assert!(chat_due(&state, later));
        // once every hidden teammate is matched exactly, it is not.
        for (key, real) in [TOP, JUNGLE, BOTTOM, SUPPORT].into_iter().zip(REAL) {
            let decoded = identity(real, "Name", MatchKind::Key);
            state.identities.insert(key.to_string(), decoded);
        }
        assert!(!chat_due(&state, later));

        // a lobby that hides nobody never needs the chat.
        let mut open = ranked();
        for player in &mut open.my_team {
            player.name_visibility_type = "VISIBLE".to_string();
        }
        assert!(!chat_due(&lobby(open), later));
        assert!(!chat_due(&LobbyState::default(), later));
    }

    #[test]
    fn stats_come_from_the_cache_or_are_fetched_once() {
        let now = Instant::now();
        let mut cache = StatsCache::default();
        let cached = played(Position::Top, 266, (12, 4));
        cache.insert(REAL[0].to_string(), cached.clone(), now);

        let mut state = lobby(ranked());
        // you (the session spelled your puuid in capitals), and the chat:
        // you again and two names nobody has yet, one of them cached.
        state.identities.insert(
            YOU.to_string(),
            identity(&YOU.to_uppercase(), "Example Player", MatchKind::Own),
        );
        state.candidates = vec![
            candidate(YOU, "Example Player"),
            candidate(REAL[0], "Name One"),
            candidate(REAL[1], "Name Two"),
        ];
        let mut fetch = take_from_cache(&mut state, &cache, now);
        fetch.sort_unstable();
        assert_eq!(fetch, [REAL[1], YOU]);
        assert_eq!(state.stats.get(REAL[0]), Some(&cached));
        assert_eq!(
            state.pending,
            HashSet::from([YOU.to_string(), REAL[1].to_string()])
        );
        // asked again while those are on the way: nothing more to fetch.
        assert!(take_from_cache(&mut state, &cache, now).is_empty());
    }

    #[test]
    fn counts_matches_without_names() {
        let identities = HashMap::from([
            (
                YOU.to_string(),
                identity(YOU, "Example Player", MatchKind::Own),
            ),
            (
                TOP.to_string(),
                identity(REAL[0], "Name One", MatchKind::Key),
            ),
            (
                JUNGLE.to_string(),
                identity(REAL[1], "Name Two", MatchKind::Key),
            ),
            (
                BOTTOM.to_string(),
                identity(REAL[2], "Name Three", MatchKind::Guess),
            ),
        ]);
        assert_eq!(
            match_counts(&identities, 1),
            "lobby: own=1 visible=0 key=2 elimination=0 guess=1 unmatched=1"
        );
    }
}
