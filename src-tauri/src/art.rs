// the league client's own rank crests, rank emblems and role icons, served to
// the page through a custom protocol, so expose ships none of riot's art. the
// page asks league-art://localhost/<file> on macos and
// http://league-art.localhost/<file> on windows; rust reads the file from
// league's own archive on disk. the client's local server does not serve
// these files, and the archive works with league closed too.

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use parking_lot::Mutex;

use tauri::http::{header, Method, Request, Response, StatusCode};
use tauri::{AppHandle, Manager, UriSchemeContext, UriSchemeResponder, Wry};

use crate::state::AppState;

pub const SCHEME: &str = "league-art";

// where the client's interface plugin keeps these files inside its archive.
const PREFIX: &str = "plugins/rcp-fe-lol-static-assets/global/default";

const TIERS: [&str; 10] = [
    "iron",
    "bronze",
    "silver",
    "gold",
    "platinum",
    "emerald",
    "diamond",
    "master",
    "grandmaster",
    "challenger",
];

const POSITIONS: [&str; 5] = ["top", "jungle", "middle", "bottom", "utility"];

// one of the client's images the page may ask for, and its path inside the
// archive.
#[derive(Debug, PartialEq, Eq)]
pub struct Art {
    pub archive_path: String,
    pub content_type: &'static str,
}

// the request path must name one allowed file exactly: a small crest for a
// tier or unranked, a full emblem for a tier, or a role icon. anything else, or a method other than get, is refused. nothing is
// decoded, so an encoded or relative path never matches.
pub fn route(get: bool, path: &str) -> Option<Art> {
    if !get {
        return None;
    }
    let file = path.strip_prefix('/')?;
    let crest = file
        .strip_prefix("images/ranked-mini-crests/")
        .and_then(|rest| rest.strip_suffix(".svg"));
    let emblem = file
        .strip_prefix("images/ranked-emblem/emblem-")
        .and_then(|rest| rest.strip_suffix(".png"));
    let role = file
        .strip_prefix("svg/position-")
        .and_then(|rest| rest.strip_suffix(".svg"));
    let allowed = match (crest, emblem, role) {
        (Some(name), _, _) => name == "unranked" || TIERS.contains(&name),
        (_, Some(tier), _) => TIERS.contains(&tier),
        (_, _, Some(name)) => POSITIONS.contains(&name),
        _ => false,
    };
    if !allowed {
        return None;
    }
    let content_type = if emblem.is_some() {
        "image/png"
    } else {
        "image/svg+xml"
    };
    Some(Art {
        archive_path: format!("{PREFIX}/{file}"),
        content_type,
    })
}

// answers one image request off the main thread: from the cache, or read from
// league's archive once and kept for the app's run, misses included. a refused
// path, no league install, or a file the archive lacks is a 404, and the page
// keeps its drawing.
pub fn serve(
    ctx: UriSchemeContext<'_, Wry>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let app = ctx.app_handle().clone();
    let art = route(request.method() == Method::GET, request.uri().path());
    tauri::async_runtime::spawn(async move {
        responder.respond(answer(&app, art).await);
    });
}

async fn answer(app: &AppHandle, art: Option<Art>) -> Response<Vec<u8>> {
    let Some(art) = art else {
        return not_found();
    };
    let handle = app.clone();
    let path = art.archive_path.clone();
    let bytes = tauri::async_runtime::spawn_blocking(move || {
        let state = handle.state::<AppState>();
        cached(&state.art, &path, || read_art(&path))
    })
    .await
    .ok()
    .flatten();
    match bytes {
        Some(bytes) => found(&art, bytes),
        None => not_found(),
    }
}

// what one read of the archive found.
pub enum Read {
    Found(Vec<u8>),
    // the archive was read and has no such file.
    Missing,
    // no league install, or the archive couldn't be read.
    Failed,
}

// the cached answer for a path, or a fresh read. a file the archive has, or
// lacks, is kept for the app's run; a failed read is not, so the next request
// tries again (a locked or half-written archive, or league installed later).
pub fn cached(
    cache: &Mutex<HashMap<String, Option<Vec<u8>>>>,
    path: &str,
    read: impl FnOnce() -> Read,
) -> Option<Vec<u8>> {
    if let Some(known) = cache.lock().get(path).cloned() {
        return known;
    }
    let bytes = match read() {
        Read::Found(bytes) => Some(bytes),
        Read::Missing => None,
        Read::Failed => return None,
    };
    cache.lock().insert(path.to_string(), bytes.clone());
    bytes
}

// the interface plugin's archive in league's folder.
fn archive_in(folder: PathBuf) -> PathBuf {
    folder
        .join("Plugins")
        .join("rcp-fe-lol-static-assets")
        .join("assets.wad")
}

// paths whose failure is already in the log, so a retried read doesn't
// repeat it.
static WARNED: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn warn_once(path: &str, message: std::fmt::Arguments) {
    let mut warned = WARNED.lock();
    if !warned.iter().any(|known| known == path) {
        warned.push(path.to_string());
        log::warn!("{message}");
    }
}

// reads one allowed file from league's archive, logging a miss or a failure.
fn read_art(archive_path: &str) -> Read {
    let Some(archive) = crate::lcu::auth::league_folders()
        .into_iter()
        .map(archive_in)
        .find(|path| path.is_file())
    else {
        warn_once(
            archive_path,
            format_args!("art: no league archive found for {archive_path}"),
        );
        return Read::Failed;
    };
    let read = File::open(&archive)
        .map_err(crate::error::Error::from)
        .and_then(|file| crate::wad::read_file(&mut BufReader::new(file), archive_path));
    match read {
        Ok(Some(bytes)) => Read::Found(bytes),
        Ok(None) => {
            log::warn!("art: {archive_path} is not in league's archive");
            Read::Missing
        }
        Err(error) => {
            warn_once(
                archive_path,
                format_args!("art: could not read {archive_path}: {error}"),
            );
            Read::Failed
        }
    }
}

fn found(art: &Art, bytes: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, art.content_type)
        .header(header::CACHE_CONTROL, "max-age=86400")
        .body(bytes)
        .unwrap_or_else(|_| not_found())
}

fn not_found() -> Response<Vec<u8>> {
    let mut response = Response::new(Vec::new());
    *response.status_mut() = StatusCode::NOT_FOUND;
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serves_the_known_crests_emblems_and_role_icons() {
        let crest = route(true, "/images/ranked-mini-crests/gold.svg").expect("a crest");
        assert_eq!(
            crest.archive_path,
            "plugins/rcp-fe-lol-static-assets/global/default/images/ranked-mini-crests/gold.svg"
        );
        assert_eq!(crest.content_type, "image/svg+xml");
        let emblem = route(true, "/images/ranked-emblem/emblem-diamond.png").expect("an emblem");
        assert_eq!(emblem.content_type, "image/png");
        assert!(route(true, "/images/ranked-mini-crests/unranked.svg").is_some());
        assert!(route(true, "/svg/position-utility.svg").is_some());
    }

    #[test]
    fn refuses_everything_else() {
        for path in [
            "",
            "/",
            "/lol-summoner/v1/current-summoner",
            "/images/ranked-mini-crests/../../lol-summoner/v1/current-summoner",
            "/images/ranked-mini-crests/%2e%2e/gold.svg",
            "/images/ranked-mini-crests/Gold.svg",
            "/images/ranked-mini-crests/gold.png",
            "/images/ranked-mini-crests/gold.svg/",
            "/images/ranked-emblem/emblem-unranked.png",
            "/images/ranked-emblem/emblem-wood.png",
            "/svg/position-fill.svg",
            "/svg/position-top-red.svg",
            // the silver role icons are not used, so not served.
            "/svg/position-utility-light.svg",
        ] {
            assert!(route(true, path).is_none(), "{path}");
        }
    }

    #[test]
    fn a_found_or_missing_file_is_read_once() {
        let cache = Mutex::new(HashMap::new());
        assert_eq!(
            cached(&cache, "a.svg", || Read::Found(b"x".to_vec())),
            Some(b"x".to_vec())
        );
        assert_eq!(
            cached(&cache, "a.svg", || panic!("read twice")),
            Some(b"x".to_vec())
        );
        assert_eq!(cached(&cache, "b.svg", || Read::Missing), None);
        assert_eq!(cached(&cache, "b.svg", || panic!("read twice")), None);
    }

    #[test]
    fn a_failed_read_is_tried_again() {
        // a locked or half-written archive, or league installed later.
        let cache = Mutex::new(HashMap::new());
        assert_eq!(cached(&cache, "a.svg", || Read::Failed), None);
        assert_eq!(
            cached(&cache, "a.svg", || Read::Found(b"<svg/>".to_vec())),
            Some(b"<svg/>".to_vec())
        );
    }

    #[test]
    fn only_get_is_served() {
        assert!(route(false, "/images/ranked-mini-crests/gold.svg").is_none());
    }

    // the live check: every allowed file, read from league's own archive.
    // run with --ignored on a machine with league installed.
    #[test]
    #[ignore = "needs league installed"]
    fn reads_every_allowed_file_from_the_installed_archive() {
        let mut files: Vec<String> = TIERS
            .iter()
            .chain(["unranked"].iter())
            .map(|tier| format!("/images/ranked-mini-crests/{tier}.svg"))
            .collect();
        files.extend(
            TIERS
                .iter()
                .map(|tier| format!("/images/ranked-emblem/emblem-{tier}.png")),
        );
        for position in POSITIONS {
            files.push(format!("/svg/position-{position}.svg"));
        }
        assert_eq!(files.len(), 26);
        for file in files {
            let art = route(true, &file).expect("an allowed file");
            let Read::Found(bytes) = read_art(&art.archive_path) else {
                panic!("{file} is not in league's archive");
            };
            if art.content_type == "image/png" {
                assert!(bytes.starts_with(b"\x89PNG"), "{file}");
            } else {
                assert!(String::from_utf8_lossy(&bytes).contains("<svg"), "{file}");
            }
        }
    }
}
