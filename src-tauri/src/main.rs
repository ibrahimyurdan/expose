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

use commands::{AUTO_ACCEPT_KEY, AUTO_OPEN_KEY, SCOUT_PROVIDER_KEY, STORE_FILE};
use state::AppState;

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
            None,
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
            commands::set_launch_at_login,
            commands::open_scout,
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
                if let Some(provider) = store
                    .get(SCOUT_PROVIDER_KEY)
                    .and_then(|value| value.as_str().map(str::to_string))
                {
                    *state.scout_provider.write() = provider;
                }
            }

            // windows lives in the notification area, kept off the taskbar.
            #[cfg(target_os = "windows")]
            {
                if let Some(window) = handle.get_webview_window("main") {
                    let _ = window.set_skip_taskbar(true);
                }
                tray::build(&handle)?;
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

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
