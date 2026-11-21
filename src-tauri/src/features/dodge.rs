// feature 3: dodge.
//
// leaving champ select on purpose, with riot's usual dodge penalty. the manual
// dodge is the two-click button in the frontend. the last-second dodge is
// armed for one lobby and fires when the finalization timer reaches the lead
// time, so the player keeps the whole lobby to change their mind. it disarms
// itself when the lobby changes or ends, so it can never fire in another one.

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use parking_lot::Mutex;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Manager};

use crate::features::notice;
use crate::lobby;
use crate::lobby::session::Session;
use crate::lobby::view::DodgeView;
use crate::models::NoticeLevel;
use crate::state::AppState;

// the last phase of champ select, counting down to the game.
const FINALIZATION: &str = "FINALIZATION";
// how long before the lobby locks in the last-second dodge fires, and the
// range the settings allow.
const DEFAULT_LEAD_MS: i64 = 1_500;
const MIN_LEAD_MS: i64 = 1_000;
const MAX_LEAD_MS: i64 = 3_000;
// what the toast says when the last-second dodge does not go through.
const FIRE_FAILED: &str = "Couldn't dodge at the last second.";

// true once the lobby is locking in with no more than the lead time left.
// picks and bans can end early, so only the finalization timer counts.
pub fn should_fire(phase: &str, time_left_ms: i64, lead_ms: i64) -> bool {
    phase == FINALIZATION && time_left_ms <= lead_ms
}

// how long to wait from now for the timer to reach the lead time; nothing when
// it already has.
pub fn fire_delay(time_left_ms: i64, lead_ms: i64) -> Duration {
    let wait = time_left_ms.saturating_sub(lead_ms);
    Duration::from_millis(u64::try_from(wait).unwrap_or(0))
}

// the lead time the settings allow, 1 to 3 seconds.
pub fn clamp_dodge_lead(ms: i64) -> i64 {
    ms.clamp(MIN_LEAD_MS, MAX_LEAD_MS)
}

pub struct DodgeState {
    // the game id of the lobby a last-second dodge is armed for.
    armed_game: Mutex<Option<i64>>,
    lead_ms: AtomicI64,
    timer: Mutex<Timer>,
}

// the one scheduled dodge. every schedule and every cancel starts a new
// generation, and only a timer of the current generation may fire, so a
// timer replaced just as it went off gives up on its own. a timer that does
// fire leaves the slot, so nothing can abort its dodge halfway.
#[derive(Default)]
struct Timer {
    generation: u64,
    handle: Option<JoinHandle<()>>,
}

impl Default for DodgeState {
    fn default() -> Self {
        Self {
            armed_game: Mutex::new(None),
            lead_ms: AtomicI64::new(DEFAULT_LEAD_MS),
            timer: Mutex::new(Timer::default()),
        }
    }
}

impl DodgeState {
    pub fn lead_ms(&self) -> i64 {
        self.lead_ms.load(Ordering::Relaxed)
    }

    // stores the lead time, clamped to what the settings allow, and returns
    // what was stored.
    pub fn set_lead_ms(&self, ms: i64) -> i64 {
        let lead = clamp_dodge_lead(ms);
        self.lead_ms.store(lead, Ordering::Relaxed);
        lead
    }

    // what the lobby with this game id shows about the last-second dodge.
    pub fn view(&self, game_id: i64) -> DodgeView {
        DodgeView {
            armed: *self.armed_game.lock() == Some(game_id),
            lead_ms: self.lead_ms(),
        }
    }

    fn arm(&self, game_id: i64) {
        *self.armed_game.lock() = Some(game_id);
    }

    fn disarm(&self) {
        *self.armed_game.lock() = None;
        self.cancel_timer();
    }

    fn cancel_timer(&self) {
        let mut timer = self.timer.lock();
        timer.generation += 1;
        if let Some(handle) = timer.handle.take() {
            handle.abort();
        }
    }

    // replaces the scheduled dodge with the one spawn starts, handing it its
    // generation. the slot stays locked while it starts, so even a dodge due
    // at once finds itself in place when it goes off.
    fn schedule(&self, spawn: impl FnOnce(u64) -> JoinHandle<()>) {
        let mut timer = self.timer.lock();
        if let Some(previous) = timer.handle.take() {
            previous.abort();
        }
        timer.generation += 1;
        timer.handle = Some(spawn(timer.generation));
    }

    // what a session means for the armed dodge: a session from another lobby
    // disarms it, and a lobby locking in says how long to wait before firing.
    // none when there is nothing to schedule.
    fn plan(&self, session: &Session) -> Option<Duration> {
        let armed = *self.armed_game.lock();
        match armed {
            None => return None,
            Some(game_id) if game_id != session.game_id => {
                self.disarm();
                return None;
            }
            Some(_) => {}
        }
        let phase = session.timer.phase.as_str();
        let left = session.timer.adjusted_time_left_in_phase;
        let lead = self.lead_ms();
        if should_fire(phase, left, lead) {
            // armed with less than the lead left: go now.
            Some(Duration::ZERO)
        } else if phase == FINALIZATION {
            Some(fire_delay(left, lead))
        } else {
            // not locking in, or past it: nothing should be waiting to fire.
            self.cancel_timer();
            None
        }
    }

    // claims the dodge for the timer going off: true, and disarmed, while it
    // is armed for this lobby and the timer is the current one. the timer is
    // let go rather than aborted, since it is the caller.
    fn take_armed(&self, game_id: i64, generation: u64) -> bool {
        let mut armed = self.armed_game.lock();
        let mut timer = self.timer.lock();
        if *armed != Some(game_id) || timer.generation != generation {
            return false;
        }
        *armed = None;
        timer.handle = None;
        true
    }

    #[cfg(test)]
    fn has_timer(&self) -> bool {
        self.timer.lock().handle.is_some()
    }
}

// leaves champ select now. only works while connected; the server-side dodge
// penalty still applies.
pub async fn dodge_now(app: &AppHandle) -> Result<(), String> {
    let client = app.state::<AppState>().client.read().clone();
    match client {
        Some(client) => client.dodge().await.map_err(|error| error.to_string()),
        None => Err("not connected to the league client".to_string()),
    }
}

// arms the last-second dodge for the lobby with this game id. a lobby that is
// already locking in gets its timer straight away.
pub fn arm(app: &AppHandle, game_id: i64) {
    let state = app.state::<AppState>();
    state.dodge.arm(game_id);
    let session = state.lobby.lock().session.clone();
    if let Some(session) = session {
        on_session(app, &session);
    }
}

pub fn disarm(app: &AppHandle) {
    app.state::<AppState>().dodge.disarm();
}

// called with every champ select session. while armed for this lobby and
// locking in, it (re)schedules the one dodge timer from the session's timer.
pub fn on_session(app: &AppHandle, session: &Session) {
    let state = app.state::<AppState>();
    let Some(delay) = state.dodge.plan(session) else {
        return;
    };
    let game_id = session.game_id;
    state.dodge.schedule(|generation| {
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(delay).await;
            fire(&app, game_id, generation).await;
        })
    });
}

// the timer went off. the latest session has the last word: the dodge only
// goes out while that same lobby is still locking in, still armed, and this
// timer was not replaced in the meantime.
async fn fire(app: &AppHandle, game_id: i64, generation: u64) {
    let state = app.state::<AppState>();
    let locking_in =
        state.lobby.lock().session.as_ref().is_some_and(|session| {
            session.game_id == game_id && session.timer.phase == FINALIZATION
        });
    if !locking_in || !state.dodge.take_armed(game_id, generation) {
        return;
    }
    log::info!("dodging at the last second");
    if let Err(error) = dodge_now(app).await {
        log::warn!("the last-second dodge failed: {error}");
        notice(app, NoticeLevel::Error, FIRE_FAILED);
    }
    // the lobby shows the dodge disarmed until champ select ends.
    lobby::refresh(app);
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::lobby::session::SessionTimer;

    fn session(game_id: i64, phase: &str, time_left_ms: i64) -> Session {
        Session {
            game_id,
            timer: SessionTimer {
                phase: phase.to_string(),
                adjusted_time_left_in_phase: time_left_ms,
            },
            ..Session::default()
        }
    }

    #[test]
    fn fires_only_in_finalization_within_lead() {
        assert!(should_fire("FINALIZATION", 1_500, 1_500));
        assert!(should_fire("FINALIZATION", 900, 1_500));
        assert!(should_fire("FINALIZATION", 0, 1_500));
        assert!(should_fire("FINALIZATION", -40, 1_500));
        assert!(!should_fire("FINALIZATION", 1_501, 1_500));
        assert!(!should_fire("FINALIZATION", 30_000, 3_000));
        // picks and bans can end early, so their timer never fires a dodge.
        assert!(!should_fire("BAN_PICK", 900, 1_500));
        assert!(!should_fire("PLANNING", 0, 1_500));
        assert!(!should_fire("GAME_STARTING", 0, 1_500));
        assert!(!should_fire("", 0, 1_500));
    }

    #[test]
    fn delay_saturates_at_zero() {
        assert_eq!(fire_delay(9_000, 1_500), Duration::from_millis(7_500));
        assert_eq!(fire_delay(1_501, 1_500), Duration::from_millis(1));
        assert_eq!(fire_delay(1_500, 1_500), Duration::ZERO);
        assert_eq!(fire_delay(800, 1_500), Duration::ZERO);
        assert_eq!(fire_delay(-5, 1_500), Duration::ZERO);
        assert_eq!(fire_delay(i64::MIN, 1_500), Duration::ZERO);
    }

    #[test]
    fn clamps_the_lead() {
        assert_eq!(clamp_dodge_lead(999), 1_000);
        assert_eq!(clamp_dodge_lead(3_001), 3_000);
        assert_eq!(clamp_dodge_lead(1_500), 1_500);
        assert_eq!(clamp_dodge_lead(-1), 1_000);

        let dodge = DodgeState::default();
        assert_eq!(dodge.lead_ms(), 1_500);
        assert_eq!(dodge.set_lead_ms(10_000), 3_000);
        assert_eq!(dodge.lead_ms(), 3_000);
        assert_eq!(
            dodge.view(7),
            DodgeView {
                armed: false,
                lead_ms: 3_000
            }
        );
    }

    #[test]
    fn schedules_once_the_lobby_locks_in() {
        let dodge = DodgeState::default();
        // not armed: nothing to schedule, whatever the phase.
        assert_eq!(dodge.plan(&session(1, "FINALIZATION", 9_000)), None);

        dodge.arm(1);
        assert_eq!(
            dodge.view(1),
            DodgeView {
                armed: true,
                lead_ms: 1_500
            }
        );
        // armed for this lobby only.
        assert!(!dodge.view(2).armed);
        // picks and bans: wait for finalization.
        assert_eq!(dodge.plan(&session(1, "BAN_PICK", 20_000)), None);
        assert!(dodge.view(1).armed);
        // locking in: fire when the timer reaches the lead.
        assert_eq!(
            dodge.plan(&session(1, "FINALIZATION", 9_000)),
            Some(Duration::from_millis(7_500))
        );
        // armed with less than the lead left: right away.
        assert_eq!(
            dodge.plan(&session(1, "FINALIZATION", 1_000)),
            Some(Duration::ZERO)
        );
        // the lead follows the setting.
        dodge.set_lead_ms(3_000);
        assert_eq!(
            dodge.plan(&session(1, "FINALIZATION", 9_000)),
            Some(Duration::from_millis(6_000))
        );
        dodge.disarm();
        assert!(!dodge.view(1).armed);
        assert_eq!(dodge.plan(&session(1, "FINALIZATION", 9_000)), None);
    }

    // a timer that never goes off by itself, and the generation it was given.
    // its task holds a sender, so the receiver hears about it once the task
    // is gone.
    fn watched_timer(dodge: &DodgeState) -> (u64, tokio::sync::oneshot::Receiver<()>) {
        let (sender, gone) = tokio::sync::oneshot::channel::<()>();
        let mut given = 0;
        dodge.schedule(|generation| {
            given = generation;
            tauri::async_runtime::spawn(async move {
                let _sender = sender;
                std::future::pending::<()>().await;
            })
        });
        (given, gone)
    }

    // true once the task behind the receiver has been aborted.
    fn aborted(gone: tokio::sync::oneshot::Receiver<()>) -> bool {
        let ended = tauri::async_runtime::block_on(async {
            tokio::time::timeout(Duration::from_secs(5), gone).await
        });
        matches!(ended, Ok(Err(_)))
    }

    #[test]
    fn disarms_on_new_game() {
        let dodge = DodgeState::default();
        dodge.arm(1);
        // a timer waiting to dodge lobby 1.
        let (_, gone) = watched_timer(&dodge);
        assert!(dodge.has_timer());

        // a session from another lobby: disarmed, timer aborted, nothing new.
        assert_eq!(dodge.plan(&session(2, "FINALIZATION", 9_000)), None);
        assert!(!dodge.view(1).armed);
        assert!(!dodge.view(2).armed);
        assert!(!dodge.has_timer());
        assert!(aborted(gone), "the timer task should be aborted");
        // and lobby 1 coming back later does not re-arm it.
        assert_eq!(dodge.plan(&session(1, "FINALIZATION", 9_000)), None);
    }

    #[test]
    fn fires_once_for_the_armed_lobby() {
        let dodge = DodgeState::default();
        dodge.arm(4);
        let (timer, _) = watched_timer(&dodge);
        // another lobby's timer cannot claim it.
        assert!(!dodge.take_armed(5, timer));
        assert!(dodge.view(4).armed);

        assert!(dodge.take_armed(4, timer));
        assert!(!dodge.view(4).armed);
        // the firing timer is let go, and it fires once.
        assert!(!dodge.has_timer());
        assert!(!dodge.take_armed(4, timer));
    }

    #[test]
    fn a_replaced_timer_gives_up() {
        let dodge = DodgeState::default();
        dodge.arm(4);
        // a session update reschedules just as the first timer goes off: the
        // first is aborted, and if it got as far as claiming the dodge, the
        // claim fails.
        let (first, first_gone) = watched_timer(&dodge);
        let (second, _) = watched_timer(&dodge);
        assert!(aborted(first_gone), "the replaced timer should be aborted");
        assert!(!dodge.take_armed(4, first));
        assert!(dodge.view(4).armed);
        assert!(dodge.take_armed(4, second));

        // a cancelled timer gives up too: the lobby moved past locking in.
        dodge.arm(4);
        let (third, third_gone) = watched_timer(&dodge);
        assert_eq!(dodge.plan(&session(4, "GAME_STARTING", 0)), None);
        assert!(!dodge.has_timer());
        assert!(aborted(third_gone), "the cancelled timer should be aborted");
        assert!(!dodge.take_armed(4, third));
        assert!(dodge.view(4).armed);
    }
}
