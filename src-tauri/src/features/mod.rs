// the features that act on the client (auto-accept and the dodge), plus the
// shared status and notice helpers that the features, the lobby and the
// supervisor use to keep the frontend in sync.

pub mod auto_accept;
pub mod dodge;

use tauri::{AppHandle, Emitter, Manager};

use crate::models::{ConnectionStatus, Notice, NoticeLevel};
use crate::state::AppState;

// event names emitted to the frontend. the expose:// prefix mirrors tauri's own
// tauri:// convention and keeps our channels clearly namespaced.
pub const EVENT_STATUS: &str = "expose://status";
// the whole champ select lobby view, or null once champ select ends.
pub const EVENT_LOBBY: &str = "expose://lobby";
pub const EVENT_NOTICE: &str = "expose://notice";

// surfaces a transient message to the user (rendered as a toast). this is the
// one channel for "something the person should know" — failures that used to be
// logged to stderr and never seen.
pub fn notice(app: &AppHandle, level: NoticeLevel, message: impl Into<String>) {
    let _ = app.emit(
        EVENT_NOTICE,
        Notice {
            level,
            message: message.into(),
        },
    );
}

// updates and broadcasts the connection status, skipping redundant emits so the
// frontend is not spammed while champ select sends rapid session updates.
pub fn set_status(app: &AppHandle, status: ConnectionStatus) {
    let state = app.state::<AppState>();
    {
        let mut current = state.status.write();
        if *current == status {
            return;
        }
        *current = status;
    }
    let _ = app.emit(EVENT_STATUS, status);
}
