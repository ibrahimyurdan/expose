// hide the extra console window on windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod ddragon;
mod error;
mod features;
mod lcu;
mod models;
mod scout;
mod state;
#[cfg(target_os = "windows")]
mod tray;
mod update;

use std::sync::atomic::Ordering;

use tauri::{AppHandle, Manager, WindowEvent};
use tauri_plugin_store::StoreExt;

use commands::{AUTO_ACCEPT_KEY, AUTO_OPEN_KEY, REVEAL_RANKED_KEY, SCOUT_PROVIDER_KEY, STORE_FILE};
use state::AppState;

// passed by the os when it starts expose at login, so a login start can stay
// quietly in the tray while a normal launch opens the window.
const AUTOSTART_FLAG: &str = "--autostart";

fn main() {
    let app = tauri::Builder::default()
        // single instance must be registered first: a second launch is
        // intercepted here and simply focuses the already running window.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![AUTOSTART_FLAG]),
        ))
        .manage(AppState::default())
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
        ])
        .on_window_event(|window, event| {
            // closing the window keeps expose running and just hides it; it is
            // reopened from the dock on macos or the tray on windows.
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
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
            }

            // windows lives in the notification area, kept off the taskbar (see
            // reveal_main_window).
            #[cfg(target_os = "windows")]
            {
                if let Some(window) = handle.get_webview_window("main") {
                    // a dark title bar to match the panel even in light mode.
                    let _ = window.set_theme(Some(tauri::Theme::Dark));
                    // with no taskbar button a minimized window has nowhere to
                    // go, so the title bar only offers close (which hides it).
                    let _ = window.set_minimizable(false);
                    let _ = window.set_maximizable(false);
                }
                tray::build(&handle)?;
                // new tray icons often land in the hidden overflow area, so a
                // normal launch opens the window to show expose is running.
                if !std::env::args().any(|arg| arg == AUTOSTART_FLAG) {
                    show_main_window(&handle);
                }
            }

            // macos: a frosted-glass window, a dock icon, and open on launch.
            #[cfg(target_os = "macos")]
            {
                if let Some(window) = handle.get_webview_window("main") {
                    let _ = window_vibrancy::apply_vibrancy(
                        &window,
                        window_vibrancy::NSVisualEffectMaterial::HudWindow,
                        None,
                        None,
                    );
                }
                show_main_window(&handle);
            }

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

fn show_main_window(app: &AppHandle) {
    reveal_main_window(app, true);
}

// shows the panel, bringing it back if it was minimized. focus is optional so
// the champ-select pop can leave the keyboard with the league client.
fn reveal_main_window(app: &AppHandle, focus: bool) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_minimized().unwrap_or(false) {
            let _ = window.unminimize();
        }
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
