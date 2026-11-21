// data shapes for the two directions of the app:
//   - inbound: the subset of league client (lcu) payloads we read
//   - outbound: the typed payloads we emit to the react frontend
//
// only the fields we actually use are modeled. every inbound field is marked
// with serde(default) because the lcu omits or zeroes fields depending on the
// lobby state (for example puuid is empty for hidden players). the champ
// select session and the lobby view built from it live in the lobby module.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::lenient::lenient;

// ---- inbound: lcu summoner lookup ----

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summoner {
    #[serde(default, deserialize_with = "lenient")]
    pub game_name: String,
    #[serde(default, deserialize_with = "lenient")]
    pub tag_line: String,
    #[serde(default, deserialize_with = "lenient")]
    pub summoner_level: i64,
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
    #[serde(default, deserialize_with = "lenient")]
    pub participants: Vec<ChatParticipant>,
}

// every field is lenient: one participant with an odd field must not cost the
// whole list, and with it every name in a ranked lobby.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ChatParticipant {
    // conversation id; the champ-select chat's cid contains "champ-select".
    #[serde(deserialize_with = "lenient")]
    pub cid: String,
    #[serde(deserialize_with = "lenient")]
    pub game_name: String,
    #[serde(deserialize_with = "lenient")]
    pub game_tag: String,
    #[serde(deserialize_with = "lenient")]
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

// a transient message surfaced to the user as a toast. used so failures that
// were previously only logged to stderr (a failed dodge, an unreachable riot
// client) actually reach the person using the app. the backend only raises
// failures; the frontend has an info level of its own.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NoticeLevel {
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
