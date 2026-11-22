// hide the extra console window on windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod art;
mod commands;
mod ddragon;
mod error;
mod features;
mod lcu;
mod lenient;
mod lobby;
mod models;
mod scout;
mod state;
mod tls;
#[cfg(target_os = "windows")]
mod tray;
mod update;
mod wad;
mod window;

use std::sync::atomic::Ordering;

use tauri::{AppHandle, Manager, WindowEvent};
use tauri_plugin_store::StoreExt;
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

use commands::{
    AUTO_ACCEPT_KEY, AUTO_OPEN_KEY, DODGE_LEAD_KEY, REVEAL_RANKED_KEY, SCOUT_PROVIDER_KEY,
    STORE_FILE,
};
use state::AppState;

// passed by the os when it starts expose at login, so a login start can stay
// quietly in the tray while a normal launch opens the window.
const AUTOSTART_FLAG: &str = "--autostart";

// the window comes back at the size and position it was left at. whether it
// shows stays with expose: it starts hidden and appears when expose decides.
const WINDOW_STATE: StateFlags = StateFlags::SIZE.union(StateFlags::POSITION);

fn main() {
    tls::install_crypto();
    log_panics();
    let app = tauri::Builder::default()
        // single instance must be registered first: a second launch is
        // intercepted here and simply focuses the already running window.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(WINDOW_STATE)
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![AUTOSTART_FLAG]),
        ))
        .manage(AppState::default())
        // the league client's own rank and role images, for the page.
        .register_asynchronous_uri_scheme_protocol(art::SCHEME, art::serve)
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::get_auto_accept,
            commands::set_auto_accept,
            commands::get_champions,
            commands::open_external,
            commands::dodge,
            commands::get_settings,
            commands::set_scout_provider,
            commands::set_auto_open,
            commands::set_reveal_ranked,
            commands::set_launch_at_login,
            commands::open_scout,
            commands::get_update,
            commands::get_lobby,
            commands::set_last_second_dodge,
            commands::set_dodge_lead,
            commands::set_welcomed,
            commands::open_logs_folder,
            commands::hide_main_window,
            commands::frontend_ready,
        ])
        .on_window_event(|window, event| {
            // closing the window keeps expose running and just hides it; it is
            // reopened from the dock on macos or the tray on windows.
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    window::hide_main(window.app_handle());
                }
            }
            // appkit may bring the traffic lights back; hide them again.
            #[cfg(target_os = "macos")]
            if matches!(event, WindowEvent::Resized(_) | WindowEvent::Focused(true))
                && window.label() == "main"
            {
                window::hide_traffic_lights(window.clone());
            }
        })
        .setup(|app| {
            let handle = app.handle().clone();
            start_logging(&handle);
            log::info!(
                "expose {} starting on {}",
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS
            );

            // reveal-in-ranked defaults on so the headline feature works out of
            // the box; the persisted store can override it to off below.
            handle
                .state::<AppState>()
                .reveal_ranked
                .store(true, Ordering::Relaxed);

            // restore persisted settings.
            if let Ok(store) = handle.store(STORE_FILE) {
                let state = handle.state::<AppState>();
                if let Some(enabled) = store.get(AUTO_ACCEPT_KEY).and_then(|value| value.as_bool())
                {
                    state.auto_accept.store(enabled, Ordering::Relaxed);
                }
                if let Some(enabled) = store.get(AUTO_OPEN_KEY).and_then(|value| value.as_bool()) {
                    state.auto_open.store(enabled, Ordering::Relaxed);
                }
                if let Some(enabled) = store
                    .get(REVEAL_RANKED_KEY)
                    .and_then(|value| value.as_bool())
                {
                    state.reveal_ranked.store(enabled, Ordering::Relaxed);
                }
                if let Some(provider) = store
                    .get(SCOUT_PROVIDER_KEY)
                    .and_then(|value| value.as_str().map(str::to_string))
                {
                    *state.scout_provider.write() = provider;
                }
                // clamped again, in case the file was edited by hand.
                if let Some(lead) = store.get(DODGE_LEAD_KEY).and_then(|value| value.as_i64()) {
                    state.dodge.set_lead_ms(lead);
                }
            }

            window::build_main_window(app)?;
            // windows lives in the notification area, kept off the taskbar (see
            // reveal_main_window); macos lives in the dock.
            #[cfg(target_os = "windows")]
            tray::build(&handle)?;
            // a launch shows the window once its first screen has painted (see
            // frontend_ready), or after a short wait if the ui never says so.
            // new tray icons often land in the hidden overflow area, so on
            // windows a normal launch opening the window shows expose runs.
            let fallback = handle.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(window::FIRST_SHOW_FALLBACK).await;
                window::first_show(&fallback, "after the fallback wait");
            });

            // background work: champion data, an update check, the supervisor.
            tauri::async_runtime::spawn(ddragon::load(handle.clone()));
            tauri::async_runtime::spawn(update::check(handle.clone()));
            tauri::async_runtime::spawn(lcu::supervise(handle));

            Ok(())
        })
        .build(tauri::generate_context!());

    match app {
        Ok(app) => app.run(|_app_handle, _event| {
            // clicking the dock icon on macos (with no visible window) reopens it.
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = _event {
                show_main_window(_app_handle);
            }
        }),
        Err(error) => {
            eprintln!("expose: fatal error while starting the application: {error}");
            std::process::exit(1);
        }
    }
}

// a windows release build has no console, so diagnostics go to a log file:
// %LOCALAPPDATA%\io.github.ibrahimyurdan.expose\logs\Expose.log on windows and
// ~/Library/Logs/io.github.ibrahimyurdan.expose/Expose.log on macos. the logger
// is attached by hand rather than as a plugin, so a log file that cannot be
// opened (held by another program, say) costs only the log, not the startup.
fn start_logging(app: &AppHandle) {
    let builder = tauri_plugin_log::Builder::new()
        .level(log::LevelFilter::Info)
        // keep the previous file when rotating, so the start of a session
        // survives a noisy one.
        .max_file_size(1_000_000)
        .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(2));
    match builder.split(app) {
        Ok((_plugin, level, logger)) => {
            if let Err(error) = tauri_plugin_log::attach_logger(level, logger) {
                eprintln!("expose: could not attach the logger: {error}");
            }
        }
        Err(error) => eprintln!("expose: logging is off, could not open the log file: {error}"),
    }
}

// a release build aborts on a panic, so the panic is written to the log
// first; otherwise a crash in a background task would leave no trace.
fn log_panics() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("panic: {info}");
        log::logger().flush();
        previous(info);
    }));
}

fn show_main_window(app: &AppHandle) {
    reveal_main_window(app, true);
}

// writes the window's size and position to disk now. the plugin also does on
// exit, but a hidden expose may never exit cleanly (a windows shutdown ends
// it), so every hide by the user saves it as well.
fn remember_window(app: &AppHandle) {
    if let Err(error) = app.save_window_state(WINDOW_STATE) {
        log::warn!("could not save the window size and position: {error}");
    }
}

// shows the panel, bringing it back if it was minimized. focus is optional so
// the champ-select pop can leave the keyboard with the league client.
fn reveal_main_window(app: &AppHandle, focus: bool) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_minimized().unwrap_or(false) {
            let _ = window.unminimize();
        }
        // back to the normal memory target before it paints again.
        #[cfg(target_os = "windows")]
        window::set_memory_target(&window, false);
        let _ = window.show();
        // windows gives a window its taskbar button back every time it is
        // shown, so the skip has to be reapplied after each show.
        #[cfg(target_os = "windows")]
        let _ = window.set_skip_taskbar(true);
        if focus {
            let _ = window.set_focus();
        }
    }
}
