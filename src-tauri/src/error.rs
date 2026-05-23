// the single error type for the backend. every fallible path in the lcu and
// feature layers returns this so we never have to reach for unwrap in
// production code. the top level entry point and the supervisor log these and
// keep running rather than crashing the app.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the lockfile at {path} is malformed: {reason}")]
    MalformedLockfile { path: String, reason: String },

    #[error("http request to the league client failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("the league client returned status {status} for {path}")]
    UnexpectedStatus { status: u16, path: String },

    #[error("websocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("could not build the tls configuration: {0}")]
    Tls(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Message(String),
}

pub type Result<T> = std::result::Result<T, Error>;
