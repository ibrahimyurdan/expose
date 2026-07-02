// state shared between the tauri commands (called from the frontend) and the
// background lcu supervisor task. all locks here are held only briefly and
// never across an await, so the std sync primitives are enough.

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;

use parking_lot::RwLock;

use crate::lcu::client::LcuClient;
use crate::models::{ChampionData, ConnectionStatus, Summoner, Teammate};

#[derive(Default)]
pub struct AppState {
    // auto-accept toggle. this atomic is the runtime source of truth; it is
    // mirrored into the persistent store whenever the frontend changes it.
    pub auto_accept: AtomicBool,
    // data dragon champion lookup, loaded once on startup.
    pub champions: RwLock<Option<ChampionData>>,
    // op.gg region slug resolved from the client on connect (for example "na").
    pub region: RwLock<Option<String>>,
    // summoners resolved during the current champ select, keyed by puuid.
    // cleared when the session ends so a new lobby starts fresh.
    pub summoner_cache: RwLock<HashMap<String, Summoner>>,
    // the last status emitted to the frontend, used to suppress duplicate emits.
    pub status: RwLock<ConnectionStatus>,
    // the connected client, so user-triggered commands (dodge) can reach the
    // lcu. some while connected, none otherwise.
    pub client: RwLock<Option<LcuClient>>,
    // the riot client, used to surface real names during anonymous champ select.
    pub riot_client: RwLock<Option<LcuClient>>,
    // the most recently resolved team, so the scout command can build a
    // multi-search link from the backend.
    pub teammates: RwLock<Vec<Teammate>>,
    // chosen scout site ("opgg" or "ugg"); empty or unknown is treated as opgg.
    pub scout_provider: RwLock<String>,
    // open the scout link automatically when champ select begins.
    pub auto_open: AtomicBool,
    // guards auto-open so it fires once per champ select, not on every update.
    pub auto_opened: AtomicBool,
    // reveal teammate names in anonymous ranked champ select. the baseline is
    // set to true at startup (the default is restored/overridden from the store)
    // so the headline feature works out of the box but can be turned off.
    pub reveal_ranked: AtomicBool,
    // guards the once-per-champ-select resolution notice (no riot client, reveal
    // disabled, chat read failed) so it does not toast on every session update.
    pub notified: AtomicBool,
}
