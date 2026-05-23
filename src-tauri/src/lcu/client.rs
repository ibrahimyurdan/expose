// a thin https client bound to one league client session.
//
// the lcu serves a self-signed certificate on a random localhost port, so the
// client is built with danger_accept_invalid_certs. that is safe here because
// every request targets 127.0.0.1 and authenticates with the per-session token.

use std::time::Duration;

use base64::Engine;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::de::DeserializeOwned;

use crate::error::{Error, Result};
use crate::models::{ChatParticipant, ChatParticipants, RegionLocale, Summoner};

#[derive(Clone)]
pub struct LcuClient {
    http: reqwest::Client,
    base: String,
}

impl LcuClient {
    // builds a client for a local riot api. works for both the league client and
    // the riot client, since both use a self-signed cert and basic auth on
    // 127.0.0.1; only the port and token differ.
    pub fn from_parts(port: u16, token: &str) -> Result<Self> {
        let encoded = base64::engine::general_purpose::STANDARD.encode(format!("riot:{token}"));
        let mut header = HeaderValue::from_str(&format!("Basic {encoded}")).map_err(|error| {
            Error::Message(format!("could not build the authorization header: {error}"))
        })?;
        header.set_sensitive(true);

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, header);

        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .default_headers(headers)
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self {
            http,
            base: format!("https://127.0.0.1:{port}"),
        })
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let response = self.http.get(format!("{}{path}", self.base)).send().await?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::UnexpectedStatus {
                status: status.as_u16(),
                path: path.to_string(),
            });
        }
        Ok(response.json::<T>().await?)
    }

    // resolves a teammate's account from their puuid. this is the call that
    // surfaces the names riot hides in the in-game champ select ui.
    pub async fn summoner_by_puuid(&self, puuid: &str) -> Result<Summoner> {
        self.get_json(&format!("/lol-summoner/v2/summoners/puuid/{puuid}"))
            .await
    }

    // the client's region, used to build region-correct op.gg links.
    pub async fn region(&self) -> Result<RegionLocale> {
        self.get_json("/riotclient/region-locale").await
    }

    // chat participants across all conversations. on the riot client this still
    // carries real game names during anonymous champ select, which is how we
    // surface teammates the league client hides. the caller filters by cid.
    pub async fn chat_participants(&self) -> Result<Vec<ChatParticipant>> {
        let body: ChatParticipants = self.get_json("/chat/v5/participants").await?;
        Ok(body.participants)
    }

    // accepts the current ready check. an empty body is expected.
    pub async fn accept_ready_check(&self) -> Result<()> {
        let path = "/lol-matchmaking/v1/ready-check/accept";
        let response = self
            .http
            .post(format!("{}{path}", self.base))
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::UnexpectedStatus {
                status: status.as_u16(),
                path: path.to_string(),
            });
        }
        Ok(())
    }

    // dodges the current champ select via the legacy lcds quit call. this is the
    // same action as leaving champ select and does not close the client; the
    // normal dodge penalty (lp loss, queue lockout) is still applied server side.
    pub async fn dodge(&self) -> Result<()> {
        let path = "/lol-login/v1/session/invoke";
        let response = self
            .http
            .post(format!("{}{path}", self.base))
            .query(&[
                ("destination", "lcdsServiceProxy"),
                ("method", "call"),
                ("args", r#"["","teambuilder-draft","quitV2",""]"#),
            ])
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::UnexpectedStatus {
                status: status.as_u16(),
                path: path.to_string(),
            });
        }
        Ok(())
    }
}
