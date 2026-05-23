// feature 2: auto-accept.
//
// when a ready check pops, the lcu sends a matchmaking event. if the toggle is
// on and we have not already responded, we post an accept.

use std::sync::atomic::Ordering;

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::lcu::client::LcuClient;
use crate::models::ReadyCheck;
use crate::state::AppState;

// the accept decision, factored out so it can be tested without a client.
// player_response is "None" until we (or the user) answer, which guards against
// posting accept more than once for the same ready check.
pub fn should_accept(check: &ReadyCheck, enabled: bool) -> bool {
    enabled && check.state == "InProgress" && check.player_response == "None"
}

pub async fn handle_ready_check(app: &AppHandle, client: &LcuClient, data: Value) {
    let Ok(check) = serde_json::from_value::<ReadyCheck>(data) else {
        return;
    };
    let enabled = app.state::<AppState>().auto_accept.load(Ordering::Relaxed);
    if should_accept(&check, enabled) {
        if let Err(error) = client.accept_ready_check().await {
            eprintln!("expose: failed to accept the ready check: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(state: &str, response: &str) -> ReadyCheck {
        ReadyCheck {
            state: state.to_string(),
            player_response: response.to_string(),
        }
    }

    #[test]
    fn accepts_when_enabled_and_pending() {
        assert!(should_accept(&check("InProgress", "None"), true));
    }

    #[test]
    fn does_not_accept_when_the_toggle_is_off() {
        assert!(!should_accept(&check("InProgress", "None"), false));
    }

    #[test]
    fn does_not_accept_when_already_responded() {
        assert!(!should_accept(&check("InProgress", "Accepted"), true));
    }

    #[test]
    fn does_not_accept_outside_an_active_ready_check() {
        assert!(!should_accept(&check("Invalid", "None"), true));
    }
}
