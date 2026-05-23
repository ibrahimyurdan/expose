// the lcu event websocket.
//
// the client exposes a wamp-style socket at wss://127.0.0.1:<port>/. after the
// upgrade we send subscribe frames shaped like [5, "<event-name>"], and events
// arrive as [8, "<event-name>", { eventType, uri, data }].
//
// because the certificate is self-signed we provide a custom verifier that
// accepts any certificate. this is safe: the only host we ever dial is
// 127.0.0.1 and we still authenticate with the per-session token.

use std::sync::Arc;

use base64::Engine;
use futures_util::SinkExt;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::{header::AUTHORIZATION, HeaderValue};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use super::Credentials;
use crate::error::{Error, Result};

// the two subscriptions the app needs.
pub const CHAMP_SELECT_EVENT: &str = "OnJsonApiEvent_lol-champ-select_v1_session";
pub const READY_CHECK_EVENT: &str = "OnJsonApiEvent_lol-matchmaking_v1_ready-check";

// the established socket type after the tls and websocket handshakes.
pub type WsStream = WebSocketStream<TlsStream<TcpStream>>;

// a decoded lcu event message.
#[derive(Debug, Clone)]
pub struct LcuEvent {
    // the subscription name, for example OnJsonApiEvent_lol-champ-select_v1_session
    pub subscription: String,
    // Create, Update, or Delete
    pub event_type: String,
    pub data: serde_json::Value,
}

// parses one incoming text frame. returns none for anything that is not an
// event message (subscribe acknowledgements, malformed text, and so on).
pub fn parse_event(text: &str) -> Option<LcuEvent> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let array = value.as_array()?;
    if array.len() != 3 || array[0].as_i64()? != 8 {
        return None;
    }
    let subscription = array[1].as_str()?.to_string();
    let payload = array[2].as_object()?;
    Some(LcuEvent {
        subscription,
        event_type: payload
            .get("eventType")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        data: payload
            .get("data")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    })
}

// connects, performs the tls + websocket handshake, and subscribes to the two
// events. the caller drives the read loop.
pub async fn connect(credentials: &Credentials) -> Result<WsStream> {
    let tcp = TcpStream::connect(("127.0.0.1", credentials.port)).await?;

    let connector = TlsConnector::from(tls_config()?);
    let domain = ServerName::try_from("127.0.0.1")
        .map_err(|error| Error::Tls(format!("invalid server name: {error}")))?;
    let tls = connector.connect(domain, tcp).await?;

    let mut request = format!("wss://127.0.0.1:{}/", credentials.port).into_client_request()?;
    let encoded =
        base64::engine::general_purpose::STANDARD.encode(format!("riot:{}", credentials.token));
    let mut auth = HeaderValue::from_str(&format!("Basic {encoded}"))
        .map_err(|error| Error::Tls(format!("invalid authorization header: {error}")))?;
    auth.set_sensitive(true);
    request.headers_mut().insert(AUTHORIZATION, auth);

    let (mut stream, _response) = tokio_tungstenite::client_async(request, tls).await?;
    subscribe(&mut stream).await?;
    Ok(stream)
}

async fn subscribe(stream: &mut WsStream) -> Result<()> {
    for event in [CHAMP_SELECT_EVENT, READY_CHECK_EVENT] {
        let frame = serde_json::json!([5, event]).to_string();
        stream.send(Message::Text(frame.into())).await?;
    }
    Ok(())
}

fn tls_config() -> Result<Arc<rustls::ClientConfig>> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| Error::Tls(error.to_string()))?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AcceptAnyServerCert))
        .with_no_client_auth();
    Ok(Arc::new(config))
}

// accepts any server certificate. only ever used against 127.0.0.1.
#[derive(Debug)]
struct AcceptAnyServerCert;

impl ServerCertVerifier for AcceptAnyServerCert {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_champ_select_event() {
        let text = r#"[8,"OnJsonApiEvent_lol-champ-select_v1_session",{"eventType":"Update","uri":"/lol-champ-select/v1/session","data":{"myTeam":[]}}]"#;
        let event = parse_event(text).expect("should parse");
        assert_eq!(event.subscription, CHAMP_SELECT_EVENT);
        assert_eq!(event.event_type, "Update");
        assert!(event.data.is_object());
    }

    #[test]
    fn parses_a_ready_check_event() {
        let text = r#"[8,"OnJsonApiEvent_lol-matchmaking_v1_ready-check",{"eventType":"Update","uri":"/lol-matchmaking/v1/ready-check","data":{"state":"InProgress","playerResponse":"None"}}]"#;
        let event = parse_event(text).expect("should parse");
        assert_eq!(event.subscription, READY_CHECK_EVENT);
        assert_eq!(event.data["state"], "InProgress");
        assert_eq!(event.data["playerResponse"], "None");
    }

    #[test]
    fn ignores_subscribe_acknowledgements_and_junk() {
        // subscribe acks are length two, not an opcode-8 event triple
        assert!(parse_event(r#"[5,"OnJsonApiEvent_lol-champ-select_v1_session"]"#).is_none());
        assert!(parse_event("definitely not json").is_none());
        assert!(parse_event(r#"{"not":"an array"}"#).is_none());
    }
}
