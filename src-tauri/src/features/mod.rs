// the two product features, plus the shared status helper they and the
// supervisor use to keep the frontend in sync.

pub mod auto_accept;
pub mod teammates;

use tauri::{AppHandle, Emitter, Manager};

use crate::models::ConnectionStatus;
use crate::state::AppState;

// event names emitted to the frontend. the expose:// prefix mirrors tauri's own
// tauri:// convention and keeps our channels clearly namespaced.
pub const EVENT_STATUS: &str = "expose://status";
pub const EVENT_TEAMMATES: &str = "expose://teammates";
pub const EVENT_PHASE: &str = "expose://phase";

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
