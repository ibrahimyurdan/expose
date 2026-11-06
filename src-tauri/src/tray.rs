// the notification area (system tray) icon on windows; macos uses the dock
// instead. left click toggles the window, right click opens a small menu.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::menu::MenuBuilder;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::show_main_window;

// windows reports a double click as two clicks plus a double click event, which
// would open the window and hide it again straight away. a click that comes
// within windows' default double click time of the last toggle, or right after
// the double click event, is treated as part of the double click.
const DOUBLE_CLICK: Duration = Duration::from_millis(500);
static LAST_CLICK: Mutex<Option<Instant>> = Mutex::new(None);

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .text("settings", "Settings")
        .text("about", "About Expose")
        .separator()
        .text("quit", "Quit Expose")
        .build()?;

    let mut builder = TrayIconBuilder::with_id("expose-tray")
        .tooltip("Expose")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "settings" | "about" => show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                if !is_second_click() {
                    toggle_window(tray.app_handle());
                }
            }
            // double click is the classic windows "open" gesture, so it always
            // ends with the window shown, however slow the clicks were.
            TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => {
                mark_click();
                show_main_window(tray.app_handle());
            }
            _ => {}
        });

    // reuse the app icon for the tray. it should always be present from the
    // bundle, but if it is missing we simply build a tray without an icon.
    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder.build(app)?;
    Ok(())
}

// true when this click belongs to a double click already being handled;
// otherwise it is remembered as the start of a possible one.
fn is_second_click() -> bool {
    let mut last = LAST_CLICK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let now = Instant::now();
    if last.is_some_and(|at| now.duration_since(at) < DOUBLE_CLICK) {
        return true;
    }
    *last = Some(now);
    false
}

// the click windows sends right after a double click event must not toggle.
fn mark_click() {
    *LAST_CLICK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Instant::now());
}

// a minimized window still counts as visible, so treat it as hidden and bring
// it back rather than hiding it for good.
fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let visible = window.is_visible().unwrap_or(false);
        let minimized = window.is_minimized().unwrap_or(false);
        if visible && !minimized {
            let _ = window.hide();
        } else {
            show_main_window(app);
        }
    }
}
