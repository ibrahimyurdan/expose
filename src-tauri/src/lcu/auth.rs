// credential discovery for the two local apis we talk to:
//   - the league client (lcu): champ select, matchmaking, summoner lookups
//   - the riot client (rc): the chat service, which still knows the real
//     teammate names during anonymous ranked champ select (the league client
//     hides them, the riot client does not)
//
// each api advertises a port and an auth token. for each we read the lockfile
// riot writes for it, falling back to the league process command line (which
// carries both the --app-* and --riotclient-* flags).

use std::cell::OnceCell;
use std::path::{Path, PathBuf};

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

use super::Credentials;
use crate::error::{Error, Result};

pub fn find_credentials() -> Option<Credentials> {
    // scanning processes is the expensive step, so it only runs when a lockfile
    // cannot answer, and at most once per attempt.
    let process_args = OnceCell::new();
    let args = || process_args.get_or_init(league_process_args).as_deref();

    let (port, token) = lcu_from_lockfile().or_else(|| lcu_from_args(args()?))?;
    let (riot_port, riot_token) = match riot_from_lockfile().or_else(|| riot_from_args(args()?)) {
        Some((port, token)) => (Some(port), Some(token)),
        None => (None, None),
    };
    Some(Credentials {
        port,
        token,
        riot_port,
        riot_token,
    })
}

// the directory a file watcher should observe so we react quickly to the client
// starting or stopping instead of waiting for the next poll.
pub fn lockfile_watch_dir() -> Option<PathBuf> {
    lcu_lockfiles()
        .into_iter()
        .filter_map(|path| path.parent().map(PathBuf::from))
        .find(|dir| dir.is_dir())
}

// ---- league client (lcu) ----

// where the league lockfile can live, most likely first.
fn lcu_lockfiles() -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        vec![PathBuf::from(
            "/Applications/League of Legends.app/Contents/LoL/lockfile",
        )]
    }
    #[cfg(target_os = "windows")]
    {
        // league can be installed on any drive. riot records the real folder in
        // its product metadata, which (unlike the process command line) can
        // still be read when the client runs as administrator.
        let mut paths = vec![PathBuf::from(r"C:\Riot Games\League of Legends\lockfile")];
        if let Some(dir) = windows_install_dir() {
            paths.push(dir.join("lockfile"));
        }
        paths
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Vec::new()
    }
}

// the league install folder from riot's metadata, for example
// "D:/Games/Riot Games/League of Legends".
#[cfg(target_os = "windows")]
fn windows_install_dir() -> Option<PathBuf> {
    let program_data = std::env::var_os("PROGRAMDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
    let settings = program_data
        .join("Riot Games")
        .join("Metadata")
        .join("league_of_legends.live")
        .join("league_of_legends.live.product_settings.yaml");
    let contents = std::fs::read_to_string(settings).ok()?;
    install_path_from_settings(&contents).map(PathBuf::from)
}

// reads product_install_full_path from a product_settings.yaml. a line scan is
// enough for this one top level key and avoids pulling in a yaml parser.
#[cfg(any(target_os = "windows", test))]
fn install_path_from_settings(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        // a byte order mark would hide the key if it is on the first line.
        let line = line.trim_start_matches('\u{feff}').trim();
        let value = line.strip_prefix("product_install_full_path:")?;
        let value = value.trim().trim_matches(|c| c == '"' || c == '\'').trim();
        (!value.is_empty()).then(|| value.to_string())
    })
}

fn lcu_from_lockfile() -> Option<(u16, String)> {
    lcu_lockfiles().iter().find_map(|path| read_lockfile(path))
}

fn lcu_from_args(args: &[String]) -> Option<(u16, String)> {
    let port = arg_value(args, "--app-port=")?.parse().ok()?;
    let token = arg_value(args, "--remoting-auth-token=")?;
    Some((port, token))
}

// ---- riot client (rc) ----

fn riot_lockfile() -> Option<PathBuf> {
    let base = directories::BaseDirs::new()?;
    Some(
        base.data_local_dir()
            .join("Riot Games")
            .join("Riot Client")
            .join("Config")
            .join("lockfile"),
    )
}

fn riot_from_lockfile() -> Option<(u16, String)> {
    read_lockfile(&riot_lockfile()?)
}

fn riot_from_args(args: &[String]) -> Option<(u16, String)> {
    let port = arg_value(args, "--riotclient-app-port=")?.parse().ok()?;
    let token = arg_value(args, "--riotclient-auth-token=")?;
    Some((port, token))
}

// ---- shared helpers ----

fn read_lockfile(path: &Path) -> Option<(u16, String)> {
    let contents = std::fs::read_to_string(path).ok()?;
    parse_lockfile(&contents).ok()
}

// lockfile format is five colon separated fields: name:pid:port:password:protocol
pub fn parse_lockfile(contents: &str) -> Result<(u16, String)> {
    let parts: Vec<&str> = contents.trim().split(':').collect();
    if parts.len() < 5 {
        return Err(Error::MalformedLockfile {
            path: "lockfile".to_string(),
            reason: format!("expected 5 colon separated fields, found {}", parts.len()),
        });
    }
    let port = parts[2]
        .parse::<u16>()
        .map_err(|_| Error::MalformedLockfile {
            path: "lockfile".to_string(),
            reason: format!("port field '{}' is not a valid number", parts[2]),
        })?;
    Ok((port, parts[3].to_string()))
}

// the league client ui process command line carries both the lcu and riot
// client connection flags.
fn league_process_args() -> Option<Vec<String>> {
    // list processes by name only (cheap), then read command lines for just the
    // league candidates instead of every process on the machine.
    let mut system = System::new();
    system.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    let candidates: Vec<Pid> = system
        .processes()
        .iter()
        .filter(|(_, process)| is_league_process(&process.name().to_string_lossy()))
        .map(|(pid, _)| *pid)
        .collect();
    if candidates.is_empty() {
        return None;
    }
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&candidates),
        true,
        ProcessRefreshKind::nothing().with_cmd(UpdateKind::Always),
    );
    candidates
        .iter()
        .filter_map(|pid| system.process(*pid))
        .map(|process| {
            process
                .cmd()
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<String>>()
        })
        .find(|args| has_connection_flags(args))
}

// leagueclient, leagueclientux and its renderer helpers all match; only the ui
// process carries the connection flags, which has_connection_flags checks.
fn is_league_process(name: &str) -> bool {
    name.to_lowercase().contains("leagueclient")
}

// picks the ui process out of the league candidates. the helpers share the name
// prefix, so whichever the process table lists first is not enough.
fn has_connection_flags<S: AsRef<str>>(args: &[S]) -> bool {
    arg_value(args, "--app-port=").is_some() && arg_value(args, "--remoting-auth-token=").is_some()
}

// extracts the value of a --flag=value style argument.
pub fn arg_value<S: AsRef<str>>(args: &[S], prefix: &str) -> Option<String> {
    args.iter().find_map(|arg| {
        arg.as_ref()
            .strip_prefix(prefix)
            .map(|value| value.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_well_formed_lockfile() {
        let (port, token) = parse_lockfile("LeagueClient:12345:54321:abcdEFGtoken:https").unwrap();
        assert_eq!(port, 54321);
        assert_eq!(token, "abcdEFGtoken");
    }

    #[test]
    fn tolerates_trailing_whitespace_in_the_lockfile() {
        let (port, _token) = parse_lockfile("LeagueClient:1:2999:tok:https\n").unwrap();
        assert_eq!(port, 2999);
    }

    #[test]
    fn rejects_a_lockfile_with_too_few_fields() {
        assert!(parse_lockfile("LeagueClient:123:456").is_err());
    }

    #[test]
    fn rejects_a_lockfile_with_a_non_numeric_port() {
        assert!(parse_lockfile("LeagueClient:123:notaport:token:https").is_err());
    }

    #[test]
    fn extracts_lcu_and_riot_client_flags_from_a_command_line() {
        let args = [
            "LeagueClientUx.exe",
            "--app-port=54321",
            "--remoting-auth-token=secret-token",
            "--riotclient-app-port=61099",
            "--riotclient-auth-token=rc-token",
        ];
        assert_eq!(arg_value(&args, "--app-port=").as_deref(), Some("54321"));
        assert_eq!(
            arg_value(&args, "--remoting-auth-token=").as_deref(),
            Some("secret-token")
        );
        assert_eq!(
            arg_value(&args, "--riotclient-app-port=").as_deref(),
            Some("61099")
        );
        assert_eq!(
            arg_value(&args, "--riotclient-auth-token=").as_deref(),
            Some("rc-token")
        );
    }

    #[test]
    fn returns_none_for_a_missing_flag() {
        let args = ["LeagueClientUx.exe", "--app-port=54321"];
        assert!(arg_value(&args, "--remoting-auth-token=").is_none());
    }

    #[test]
    fn matches_league_process_names_on_both_platforms() {
        assert!(is_league_process("LeagueClientUx.exe"));
        assert!(is_league_process("LeagueClientUxRender.exe"));
        assert!(is_league_process("LeagueClientUx"));
        assert!(!is_league_process("RiotClientServices.exe"));
        assert!(!is_league_process("League of Legends.exe"));
    }

    #[test]
    fn only_the_ui_process_has_the_connection_flags() {
        let ui = [
            "LeagueClientUx.exe",
            "--riotclient-auth-token=rc-token",
            "--app-port=54321",
            "--remoting-auth-token=secret-token",
        ];
        let renderer = [
            "LeagueClientUxRender.exe",
            "--type=renderer",
            "--lang=en-US",
        ];
        assert!(has_connection_flags(&ui));
        assert!(!has_connection_flags(&renderer));
        assert!(!has_connection_flags(&[
            "LeagueClientUx.exe",
            "--app-port=54321"
        ]));
    }

    #[test]
    fn reads_the_install_folder_from_riot_product_settings() {
        let settings = "auto_patching_enabled_by_player: false\n\
                        product_install_full_path: \"D:/Games/Riot Games/League of Legends\"\n\
                        product_install_root: \"D:/Games/Riot Games\"\n";
        assert_eq!(
            install_path_from_settings(settings).as_deref(),
            Some("D:/Games/Riot Games/League of Legends")
        );
        // windows line endings and an unquoted value.
        assert_eq!(
            install_path_from_settings(
                "product_install_full_path: C:/Riot Games/League of Legends\r\n"
            )
            .as_deref(),
            Some("C:/Riot Games/League of Legends")
        );
        assert!(install_path_from_settings("product_install_root: \"C:/Riot Games\"\n").is_none());
        assert!(install_path_from_settings("product_install_full_path: \"\"\n").is_none());
    }
}
