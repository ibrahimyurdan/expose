// the league client's own rank crests, rank emblems and role icons, served to
// the page through a custom protocol, so expose ships none of riot's art. the
// page asks league-art://localhost/<file> on macos and
// http://league-art.localhost/<file> on windows; rust reads the file from
// league's own archive on disk. the client's local server does not serve
// these files, and the archive works with league closed too.

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

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
// tier or unranked, a full emblem for a tier, or a role icon or its light
// variant. anything else, or a method other than get, is refused. nothing is
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
        (_, _, Some(name)) => POSITIONS.contains(&name.strip_suffix("-light").unwrap_or(name)),
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
    let state = app.state::<AppState>();
    let cached = state.art.lock().get(&art.archive_path).cloned();
    let bytes = match cached {
        Some(known) => known,
        None => {
            let path = art.archive_path.clone();
            let read = tauri::async_runtime::spawn_blocking(move || read_art(&path))
                .await
                .ok()
                .flatten();
            state
                .art
                .lock()
                .insert(art.archive_path.clone(), read.clone());
            read
        }
    };
    match bytes {
        Some(bytes) => found(&art, bytes),
        None => not_found(),
    }
}

// the interface plugin's archive in league's folder.
fn archive_in(folder: PathBuf) -> PathBuf {
    folder
        .join("Plugins")
        .join("rcp-fe-lol-static-assets")
        .join("assets.wad")
}

// reads one allowed file from league's archive. a miss is logged; the cache
// keeps it, so each file is tried once per run.
fn read_art(archive_path: &str) -> Option<Vec<u8>> {
    let Some(archive) = crate::lcu::auth::league_folders()
        .into_iter()
        .map(archive_in)
        .find(|path| path.is_file())
    else {
        log::warn!("art: no league archive found for {archive_path}");
        return None;
    };
    let read = File::open(&archive)
        .map_err(crate::error::Error::from)
        .and_then(|file| crate::wad::read_file(&mut BufReader::new(file), archive_path));
    match read {
        Ok(Some(bytes)) => Some(bytes),
        Ok(None) => {
            log::warn!("art: {archive_path} is not in league's archive");
            None
        }
        Err(error) => {
            log::warn!("art: could not read {archive_path}: {error}");
            None
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
        assert!(route(true, "/svg/position-utility-light.svg").is_some());
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
        ] {
            assert!(route(true, path).is_none(), "{path}");
        }
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
            files.push(format!("/svg/position-{position}-light.svg"));
        }
        assert_eq!(files.len(), 31);
        for file in files {
            let art = route(true, &file).expect("an allowed file");
            let bytes = read_art(&art.archive_path).expect("in league's archive");
            if art.content_type == "image/png" {
                assert!(bytes.starts_with(b"\x89PNG"), "{file}");
            } else {
                assert!(String::from_utf8_lossy(&bytes).contains("<svg"), "{file}");
            }
        }
    }
}
