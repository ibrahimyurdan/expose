// the main window. it is built here rather than from the config alone, so
// each platform gets its own chrome, and the ui learns which platform and
// window frame it runs in before any page script runs. a launch shows it
// once the ui has painted its first screen, never empty.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Manager, WebviewWindow, WebviewWindowBuilder};

use crate::{remember_window, show_main_window, AUTOSTART_FLAG};

// how long a launch waits for the ui to say its first screen is painted
// before showing the window anyway, so a font or script that never loads
// cannot keep it hidden.
pub const FIRST_SHOW_FALLBACK: Duration = Duration::from_millis(1500);

// windows 11 starts at this build.
const WINDOWS_11: u32 = 22_000;

static FIRST_SHOWN: AtomicBool = AtomicBool::new(false);

// who draws the window's edge: macos itself (native), windows 11's own
// shadow and border (os), or a 1px line the ui draws on windows 10 (css).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frame {
    Native,
    Os,
    Css,
}

impl Frame {
    pub fn as_str(self) -> &'static str {
        match self {
            Frame::Native => "native",
            Frame::Os => "os",
            Frame::Css => "css",
        }
    }
}

pub fn frame_for(os: &str, windows_build: Option<u32>) -> Frame {
    match os {
        "windows" if windows_build.is_some_and(|build| build >= WINDOWS_11) => Frame::Os,
        "windows" => Frame::Css,
        _ => Frame::Native,
    }
}

// runs in the page before anything else, so the first render already knows.
pub fn init_script(platform: &str, frame: Frame) -> String {
    format!(
        r#"window.__EXPOSE_PLATFORM__="{platform}";window.__EXPOSE_FRAME__="{}";"#,
        frame.as_str()
    )
}

// a login start on windows stays quietly in the tray; every other launch
// opens the window.
pub fn launch_shows(os: &str, args: &[String]) -> bool {
    !(os == "windows" && args.iter().any(|arg| arg == AUTOSTART_FLAG))
}

fn should_first_show(already_shown: bool, launch_shows: bool) -> bool {
    !already_shown && launch_shows
}

// shows the window for this launch, once: when the ui says its first screen
// is painted, or after the fallback wait, whichever comes first. the log says
// which, so a slow first paint shows up in bug reports.
pub fn first_show(app: &AppHandle, reason: &str) {
    let args: Vec<String> = std::env::args().collect();
    let shows = launch_shows(std::env::consts::OS, &args);
    let already = FIRST_SHOWN.swap(true, Ordering::SeqCst);
    if should_first_show(already, shows) {
        log::info!("window: first shown ({reason})");
        show_main_window(app);
    }
}

pub fn build_main_window(app: &tauri::App) -> tauri::Result<WebviewWindow> {
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .cloned()
        .ok_or(tauri::Error::WindowNotFound)?;
    let platform = std::env::consts::OS;
    let frame = frame_for(platform, windows_build());
    let builder = WebviewWindowBuilder::from_config(app.handle(), &config)?
        .initialization_script(init_script(platform, frame));
    // macos: a title bar drawn over the page, with no title. the first click
    // on the inactive window goes through, so dodge works while league has
    // focus.
    #[cfg(target_os = "macos")]
    let builder = builder
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true)
        .accept_first_mouse(true);
    // windows: the window has no taskbar button, so a minimized window would
    // have nowhere to go.
    #[cfg(target_os = "windows")]
    let builder = builder.minimizable(false);
    builder.build()
}

// hides the window and saves its size and position.
pub fn hide_main(app: &AppHandle) {
    remember_window(app);
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[cfg(target_os = "windows")]
fn windows_build() -> Option<u32> {
    Some(windows_version::OsVersion::current().build)
}

#[cfg(not(target_os = "windows"))]
fn windows_build() -> Option<u32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_follows_the_platform_and_windows_build() {
        assert_eq!(frame_for("macos", None), Frame::Native);
        // windows 11 starts at build 22000 and draws the frame itself.
        assert_eq!(frame_for("windows", Some(22_000)), Frame::Os);
        assert_eq!(frame_for("windows", Some(22_631)), Frame::Os);
        // windows 10, or a build that could not be read, gets the css edge.
        assert_eq!(frame_for("windows", Some(19_045)), Frame::Css);
        assert_eq!(frame_for("windows", None), Frame::Css);
    }

    #[test]
    fn init_script_names_platform_and_frame() {
        assert_eq!(
            init_script("windows", Frame::Css),
            r#"window.__EXPOSE_PLATFORM__="windows";window.__EXPOSE_FRAME__="css";"#
        );
        assert_eq!(
            init_script("macos", Frame::Native),
            r#"window.__EXPOSE_PLATFORM__="macos";window.__EXPOSE_FRAME__="native";"#
        );
    }

    #[test]
    fn login_launch_stays_hidden_only_on_windows() {
        let login = ["expose".to_string(), "--autostart".to_string()];
        let normal = ["expose".to_string()];
        assert!(!launch_shows("windows", &login));
        assert!(launch_shows("windows", &normal));
        assert!(launch_shows("macos", &login));
        assert!(launch_shows("macos", &normal));
    }

    #[test]
    fn first_show_happens_once() {
        assert!(should_first_show(false, true));
        assert!(!should_first_show(true, true));
        assert!(!should_first_show(false, false));
    }
}
