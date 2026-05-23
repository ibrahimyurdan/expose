// the lcu layer: everything required to find, connect to, and talk to the
// local league client api, plus the supervisor that ties it all together.

pub mod auth;
pub mod client;
pub mod websocket;

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::tungstenite::Message;

use crate::error::Result;
use crate::features::{self, auto_accept, teammates};
use crate::models::{ConnectionStatus, Teammate};
use crate::state::AppState;
use client::LcuClient;
use websocket::{LcuEvent, WsStream, CHAMP_SELECT_EVENT, READY_CHECK_EVENT};

// how often we re-check for the client when not connected. a file watcher wakes
// us sooner than this when the lockfile appears, so this is just the ceiling.
const POLL_INTERVAL: Duration = Duration::from_secs(2);

// credentials for one running league client session. the port is random per
// launch and the token authenticates every http and websocket request.
#[derive(Debug, Clone)]
pub struct Credentials {
    pub port: u16,
    pub token: String,
    // the riot client api, used to surface real names in anonymous champ select.
    // optional because a custom install might only yield the league lockfile.
    pub riot_port: Option<u16>,
    pub riot_token: Option<String>,
}

// the connection supervisor. it owns the full lifecycle: detect the client,
// connect, pump events until the socket drops, then reset and poll again. it is
// built to run forever and never panic, logging recoverable errors instead.
pub async fn supervise(app: AppHandle) {
    // keep the watcher alive for the whole loop; dropping it stops watching.
    let (_watcher, mut wake) = match spawn_lockfile_watcher() {
        Some((watcher, rx)) => (Some(watcher), Some(rx)),
        None => (None, None),
    };

    loop {
        let Some(credentials) = auth::find_credentials() else {
            wait_for_next_attempt(&mut wake).await;
            continue;
        };

        if let Err(error) = run_session(&app, &credentials).await {
            eprintln!("expose: lcu session ended: {error}");
        }

        // the client went away or the socket dropped: reset to waiting.
        features::set_status(&app, ConnectionStatus::Waiting);
        *app.state::<AppState>().client.write() = None;
        *app.state::<AppState>().riot_client.write() = None;
        app.state::<AppState>().summoner_cache.write().clear();
        app.state::<AppState>().teammates.write().clear();
        let _ = app.emit(features::EVENT_TEAMMATES, Vec::<Teammate>::new());

        wait_for_next_attempt(&mut wake).await;
    }
}

async fn run_session(app: &AppHandle, credentials: &Credentials) -> Result<()> {
    let client = LcuClient::from_parts(credentials.port, &credentials.token)?;
    let state = app.state::<AppState>();
    // publish the client so user-triggered commands (dodge) can reach it.
    *state.client.write() = Some(client.clone());

    // build the riot client too; it is how we surface real names when the
    // league client hides them in anonymous ranked champ select.
    if let (Some(port), Some(token)) = (credentials.riot_port, credentials.riot_token.as_deref()) {
        match LcuClient::from_parts(port, token) {
            Ok(riot) => *state.riot_client.write() = Some(riot),
            Err(error) => eprintln!("expose: could not build riot client: {error}"),
        }
    }

    // best effort region for op.gg links; failing this is not fatal.
    if let Ok(region) = client.region().await {
        let slug = if region.web_region.is_empty() {
            region.region.to_lowercase()
        } else {
            region.web_region
        };
        *app.state::<AppState>().region.write() = Some(slug);
    }

    let stream = websocket::connect(credentials).await?;
    features::set_status(app, ConnectionStatus::Connected);
    read_loop(app, &client, stream).await;
    Ok(())
}

async fn read_loop(app: &AppHandle, client: &LcuClient, mut stream: WsStream) {
    while let Some(message) = stream.next().await {
        match message {
            Ok(Message::Text(text)) => {
                if let Some(event) = websocket::parse_event(text.as_str()) {
                    dispatch(app, client, event).await;
                }
            }
            // answer keepalive pings so the client does not drop us.
            Ok(Message::Ping(payload)) => {
                let _ = stream.send(Message::Pong(payload)).await;
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(error) => {
                eprintln!("expose: websocket read error: {error}");
                break;
            }
        }
    }
}

async fn dispatch(app: &AppHandle, client: &LcuClient, event: LcuEvent) {
    match event.subscription.as_str() {
        CHAMP_SELECT_EVENT => {
            teammates::handle_session(app, client, &event.event_type, event.data).await;
        }
        READY_CHECK_EVENT => {
            auto_accept::handle_ready_check(app, client, event.data).await;
        }
        _ => {}
    }
}

// sleeps until the poll interval elapses or the lockfile directory changes,
// whichever comes first.
async fn wait_for_next_attempt(wake: &mut Option<Receiver<()>>) {
    match wake {
        Some(rx) => {
            tokio::select! {
                _ = tokio::time::sleep(POLL_INTERVAL) => {}
                _ = rx.recv() => {}
            }
        }
        None => tokio::time::sleep(POLL_INTERVAL).await,
    }
}

// watches the lockfile's parent directory and signals on any change so the
// supervisor reacts to the client starting or stopping without waiting a full
// poll. returns none when the default directory does not exist, in which case
// the poll alone (which also does process inspection) is used.
fn spawn_lockfile_watcher() -> Option<(RecommendedWatcher, Receiver<()>)> {
    let dir = auth::lockfile_watch_dir()?;
    if !dir.exists() {
        return None;
    }
    let (tx, rx) = tokio::sync::mpsc::channel::<()>(8);
    let mut watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        if result.is_ok() {
            // a full queue already means a wake is pending, so dropping is fine.
            let _ = tx.try_send(());
        }
    })
    .ok()?;
    watcher.watch(&dir, RecursiveMode::NonRecursive).ok()?;
    Some((watcher, rx))
}
