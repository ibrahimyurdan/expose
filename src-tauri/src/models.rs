// data shapes for the two directions of the app:
//   - inbound: the subset of league client (lcu) payloads we read
//   - outbound: the typed payloads we emit to the react frontend
//
// only the fields we actually use are modeled. every inbound field is marked
// with serde(default) because the lcu omits or zeroes fields depending on the
// lobby state (for example puuid is empty for hidden players).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---- inbound: lcu champ select ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectSession {
    #[serde(default)]
    pub my_team: Vec<ChampSelectPlayer>,
    #[serde(default)]
    pub timer: ChampSelectTimer,
    // theirTeam arrives with identities scrubbed by riot, so we ignore it.
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectTimer {
    // PLANNING, BAN_PICK, FINALIZATION, and so on.
    #[serde(default)]
    pub phase: String,
    // milliseconds remaining in the current phase.
    #[serde(default)]
    pub adjusted_time_left_in_phase: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectPlayer {
    #[serde(default)]
    pub cell_id: i64,
    #[serde(default)]
    pub puuid: String,
    #[serde(default)]
    pub champion_id: i64,
    #[serde(default)]
    pub champion_pick_intent: i64,
    #[serde(default)]
    pub assigned_position: String,
    // "HIDDEN" in anonymous ranked champ select, "UNHIDDEN" otherwise.
    #[serde(default)]
    pub name_visibility_type: String,
}

// ---- inbound: lcu summoner lookup ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summoner {
    #[serde(default)]
    pub game_name: String,
    #[serde(default)]
    pub tag_line: String,
    #[serde(default)]
    pub summoner_level: i64,
    #[serde(default)]
    pub profile_icon_id: i64,
}

// ---- inbound: lcu ready check ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadyCheck {
    // one of: Invalid, InProgress, Accepted, Declined
    #[serde(default)]
    pub state: String,
    // one of: None, Accepted, Declined
    #[serde(default)]
    pub player_response: String,
}

// ---- inbound: lcu region (used to build op.gg links) ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionLocale {
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub web_region: String,
}

// ---- inbound: riot client chat participants ----
// the riot client chat api (/chat/v5/participants) still reports real game
// names during anonymous champ select. its json fields are already snake_case.

#[derive(Debug, Clone, Deserialize)]
pub struct ChatParticipants {
    #[serde(default)]
    pub participants: Vec<ChatParticipant>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatParticipant {
    // conversation id; the champ-select chat's cid contains "champ-select".
    #[serde(default)]
    pub cid: String,
    #[serde(default)]
    pub game_name: String,
    #[serde(default)]
    pub game_tag: String,
    #[serde(default)]
    pub puuid: String,
}

// ---- inbound: data dragon static champion data ----

#[derive(Debug, Deserialize)]
pub struct DataDragonChampions {
    pub data: HashMap<String, DataDragonChampion>,
}

#[derive(Debug, Deserialize)]
pub struct DataDragonChampion {
    // numeric champion id as a string, for example "266"
    pub key: String,
    // image/lookup id, for example "Aatrox"
    pub id: String,
    pub name: String,
}

// ---- outbound: emitted to the frontend ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionStatus {
    #[default]
    Waiting,
    Connected,
    ChampSelect,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectPhase {
    // the raw lcu phase; the frontend maps it to a friendly label.
    pub phase: String,
    pub time_left_ms: i64,
}

// a transient message surfaced to the user as a toast. used so failures that
// were previously only logged to stderr (a failed dodge, an unreachable riot
// client) actually reach the person using the app.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NoticeLevel {
    Info,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Notice {
    pub level: NoticeLevel,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Teammate {
    pub cell_id: i64,
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
    pub summoner_level: i64,
    pub profile_icon_id: i64,
    pub assigned_position: String,
    pub champion_id: i64,
    pub champion_pick_intent: i64,
    // single-summoner profile link on the chosen scout site, plus the site's
    // label for the row button. rebuilt when the scout provider changes.
    pub scout_url: String,
    pub scout_label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampionData {
    pub version: String,
    // numeric champion id -> display entry. serializes to an object keyed by
    // the stringified id, which is how the frontend looks champions up.
    pub champions: HashMap<i64, ChampionEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampionEntry {
    // data dragon image id, used to build the square icon url
    pub id: String,
    pub name: String,
}
