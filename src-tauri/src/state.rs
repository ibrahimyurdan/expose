// state shared between the tauri commands (called from the frontend) and the
// background lcu supervisor task. all locks here are held only briefly and
// never across an await, so the std sync primitives are enough.

use std::sync::atomic::AtomicBool;

use parking_lot::{Mutex, RwLock};

use crate::lcu::client::LcuClient;
use crate::lobby::stats::StatsCache;
use crate::lobby::view::LobbyView;
use crate::lobby::LobbyState;
use crate::models::{ChampionData, ConnectionStatus};
use crate::update::UpdateInfo;

#[derive(Default)]
pub struct AppState {
    // auto-accept toggle. this atomic is the runtime source of truth; it is
    // mirrored into the persistent store whenever the frontend changes it.
    pub auto_accept: AtomicBool,
    // data dragon champion lookup, loaded once on startup.
    pub champions: RwLock<Option<ChampionData>>,
    // op.gg region slug resolved from the client on connect (for example "na").
    pub region: RwLock<Option<String>>,
    // the last status emitted to the frontend, used to suppress duplicate emits.
    pub status: RwLock<ConnectionStatus>,
    // the connected client, so user-triggered commands (dodge) can reach the
    // lcu. some while connected, none otherwise.
    pub client: RwLock<Option<LcuClient>>,
    // the riot client, used to surface real names during anonymous champ select.
    pub riot_client: RwLock<Option<LcuClient>>,
    // the champ select lobby being followed. a new lobby resets it.
    pub lobby: Mutex<LobbyState>,
    // the view last emitted for it, so a frontend that mounts mid champ select
    // can read it; none outside champ select.
    pub lobby_view: RwLock<Option<LobbyView>>,
    // player stats from recent lobbies, reused for ten minutes.
    pub stats_cache: Mutex<StatsCache>,
    // chosen scout site ("opgg" or "ugg"); empty or unknown is treated as opgg.
    pub scout_provider: RwLock<String>,
    // open the scout link automatically when champ select begins.
    pub auto_open: AtomicBool,
    // reveal teammate names in anonymous ranked champ select. the baseline is
    // set to true at startup (the default is restored/overridden from the store)
    // so the headline feature works out of the box but can be turned off.
    pub reveal_ranked: AtomicBool,
    // the startup update check result. kept so the settings panel can read it
    // whenever it opens; the one-off event fires before the panel exists.
    pub update: RwLock<Option<UpdateInfo>>,
}
