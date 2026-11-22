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
    // windows: no system frame; the title bar is expose's own. windows 11
    // keeps its shadow (with square corners, set below); windows 10 has no
    // shadow, and the ui draws the edge. the window has no taskbar button, so
    // a minimized window would have nowhere to go.
    #[cfg(target_os = "windows")]
    let builder = builder
        .decorations(false)
        .shadow(frame == Frame::Os)
        .minimizable(false);
    let window = builder.build()?;
    #[cfg(target_os = "windows")]
    if frame == Frame::Os {
        square_corners(&window);
    }
    // macos: the title bar's own buttons give way to expose's.
    #[cfg(target_os = "macos")]
    hide_traffic_lights(window.as_ref().window());
    Ok(window)
}

// hides the red, yellow and green buttons; expose's title bar draws league
// style ones instead. appkit can bring them back when the window resizes or
// comes forward, so this runs again then (see main.rs). never set a traffic
// light position: tao moves the buttons on every redraw when one is set.
#[cfg(target_os = "macos")]
pub fn hide_traffic_lights(window: tauri::Window) {
    let target = window.clone();
    let _ = window.run_on_main_thread(move || {
        let Ok(handle) = target.ns_window() else {
            return;
        };
        // safety: tauri hands out this window's live NSWindow, and appkit is
        // used here on the main thread only.
        let ns_window = unsafe { &*handle.cast::<objc2_app_kit::NSWindow>() };
        let mut shown = 0;
        for kind in [
            objc2_app_kit::NSWindowButton::CloseButton,
            objc2_app_kit::NSWindowButton::MiniaturizeButton,
            objc2_app_kit::NSWindowButton::ZoomButton,
        ] {
            if let Some(button) = ns_window.standardWindowButton(kind) {
                button.setHidden(true);
                if !button.isHidden() {
                    shown += 1;
                }
            }
        }
        // said once, so a bug report shows whether the hiding held.
        if !LIGHTS_LOGGED.swap(true, Ordering::Relaxed) {
            if shown == 0 {
                log::info!("window: traffic lights hidden");
            } else {
                log::warn!("window: {shown} traffic light button(s) stayed visible");
            }
        }
    });
}

#[cfg(target_os = "macos")]
static LIGHTS_LOGGED: AtomicBool = AtomicBool::new(false);

// windows 11 rounds a frameless window's corners and draws a light border;
// the league look keeps them square, with an edge in the window's dark blue.
#[cfg(target_os = "windows")]
fn square_corners(window: &WebviewWindow) {
    use windows::Win32::Foundation::COLORREF;
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE,
        DWMWCP_DONOTROUND, DWM_WINDOW_CORNER_PREFERENCE,
    };

    let Ok(hwnd) = window.hwnd() else {
        return;
    };
    let corners = DWMWCP_DONOTROUND;
    // #1e282d, written as 0x00bbggrr.
    let border = COLORREF(0x002D_281E);
    // safety: both values live across the calls, and each size matches its
    // value's type.
    unsafe {
        if let Err(error) = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            std::ptr::from_ref(&corners).cast(),
            std::mem::size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32,
        ) {
            log::warn!("could not square the window corners: {error}");
        }
        if let Err(error) = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            std::ptr::from_ref(&border).cast(),
            std::mem::size_of::<COLORREF>() as u32,
        ) {
            log::warn!("could not color the window border: {error}");
        }
    }
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
