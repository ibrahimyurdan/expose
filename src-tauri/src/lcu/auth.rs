// credential discovery for the two local apis we talk to:
//   - the league client (lcu): champ select, matchmaking, summoner lookups
//   - the riot client (rc): the chat service, which still knows the real
//     teammate names during anonymous ranked champ select (the league client
//     hides them, the riot client does not)
//
// each api advertises a port and an auth token. for each we read the lockfile
// riot writes for it, falling back to the league process command line (which
// carries both the --app-* and --riotclient-* flags).

use std::path::PathBuf;

use sysinfo::System;

use super::Credentials;
use crate::error::{Error, Result};

pub fn find_credentials() -> Option<Credentials> {
    let (port, token) = lcu_from_lockfile().or_else(lcu_from_process)?;
    let (riot_port, riot_token) = match riot_from_lockfile().or_else(riot_from_process) {
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
    lcu_lockfile().and_then(|path| path.parent().map(PathBuf::from))
}

// ---- league client (lcu) ----

fn lcu_lockfile() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        Some(PathBuf::from(
            "/Applications/League of Legends.app/Contents/LoL/lockfile",
        ))
    }
    #[cfg(target_os = "windows")]
    {
        Some(PathBuf::from(r"C:\Riot Games\League of Legends\lockfile"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

fn lcu_from_lockfile() -> Option<(u16, String)> {
    let contents = std::fs::read_to_string(lcu_lockfile()?).ok()?;
    parse_lockfile(&contents).ok()
}

fn lcu_from_process() -> Option<(u16, String)> {
    let args = league_process_args()?;
    let port = arg_value(&args, "--app-port=")?.parse().ok()?;
    let token = arg_value(&args, "--remoting-auth-token=")?;
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
    let contents = std::fs::read_to_string(riot_lockfile()?).ok()?;
    parse_lockfile(&contents).ok()
}

fn riot_from_process() -> Option<(u16, String)> {
    let args = league_process_args()?;
    let port = arg_value(&args, "--riotclient-app-port=")?.parse().ok()?;
    let token = arg_value(&args, "--riotclient-auth-token=")?;
    Some((port, token))
}

// ---- shared helpers ----

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

// the league client process command line carries both the lcu and riot client
// connection flags.
fn league_process_args() -> Option<Vec<String>> {
    let system = System::new_all();
    for process in system.processes().values() {
        let name = process.name().to_string_lossy().to_lowercase();
        if name.contains("leagueclientux") || name.contains("leagueclient") {
            let args: Vec<String> = process
                .cmd()
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect();
            if !args.is_empty() {
                return Some(args);
            }
        }
    }
    None
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
}
