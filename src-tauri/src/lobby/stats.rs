// what expose knows about a player: their recent games, their solo queue rank,
// their level and their champion mastery, all read from the league client.
// each player costs four requests, and each request stands alone: one that
// fails hides only its own part, so a private match history still leaves the
// rank and the level. stats are reused for ten minutes, so a requeue into a
// new lobby with the same players does not fetch them again.

use std::cmp::Reverse;
use std::collections::HashMap;
use std::future::Future;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

use super::identity::parse_uuid;
use super::session::is_blank;
use crate::error::{Error, Result};
use crate::lcu::client::LcuClient;
use crate::lenient::lenient;
use crate::models::Summoner;

// how many recent games are fetched per player.
pub const HISTORY_GAMES: usize = 20;

// only summoner's rift games say how someone plays ranked.
const SUMMONERS_RIFT: i64 = 11;
// and only its standard queues: ranked solo/duo and flex, normal draft and
// blind, quickplay, swiftplay and clash. bots, customs and the event modes
// played on the same map (urf, one for all) say little about a ranked game.
const STANDARD_QUEUES: [i64; 7] = [400, 420, 430, 440, 480, 490, 700];
// a game under five minutes was a remake and says nothing about the player.
const REMAKE_SECONDS: i64 = 300;
// a bottom laner under this many creeps a minute was the support.
const SUPPORT_CREEPS_PER_MINUTE: f64 = 2.0;
// how long fetched stats are reused across lobbies.
const CACHE_LIFETIME: Duration = Duration::from_secs(600);
// the solo/duo queue in the ranked stats.
const SOLO_QUEUE: &str = "RANKED_SOLO_5x5";

// the five roles on summoner's rift, serialized the way champ select names
// them ("top", "jungle", "middle", "bottom", "utility").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    Top,
    Jungle,
    Middle,
    Bottom,
    Utility,
}

impl Position {
    pub const ALL: [Position; 5] = [
        Position::Top,
        Position::Jungle,
        Position::Middle,
        Position::Bottom,
        Position::Utility,
    ];

    // champ select's assignedPosition, in any case. blind pick leaves it
    // empty, which is no position.
    pub fn from_assigned(text: &str) -> Option<Position> {
        match text.to_ascii_lowercase().as_str() {
            "top" => Some(Position::Top),
            "jungle" => Some(Position::Jungle),
            "middle" => Some(Position::Middle),
            "bottom" => Some(Position::Bottom),
            "utility" => Some(Position::Utility),
            _ => None,
        }
    }

    // the short name players use.
    pub fn label(self) -> &'static str {
        match self {
            Position::Top => "top",
            Position::Jungle => "jungle",
            Position::Middle => "mid",
            Position::Bottom => "bot",
            Position::Utility => "support",
        }
    }
}

// one summoner's rift game, from the player's side.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameSummary {
    pub champion_id: i64,
    pub win: bool,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    // when the game was created, in epoch milliseconds.
    pub started_at_ms: i64,
    pub duration_s: i64,
    // none when riot could not tell the lane.
    pub position: Option<Position>,
    pub spells: (i64, i64),
}

// a player's recent summoner's rift games, newest first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerHistory {
    pub games: Vec<GameSummary>,
}

impl PlayerHistory {
    // the player's games in a match history response: summoner's rift's
    // standard queues only, remakes left out, newest first whatever order the
    // client sent them in.
    pub fn from_matches(list: &MatchHistoryList, puuid: &str) -> PlayerHistory {
        let mut games: Vec<GameSummary> = list
            .games
            .games
            .iter()
            .filter(|game| {
                game.map_id == SUMMONERS_RIFT
                    && STANDARD_QUEUES.contains(&game.queue_id)
                    && game.game_duration >= REMAKE_SECONDS
            })
            .filter_map(|game| summarize(game, puuid))
            .collect();
        games.sort_by_key(|game| Reverse(game.started_at_ms));
        PlayerHistory { games }
    }

    // the share of the games with a known role that were played in this one.
    pub fn role_share(&self, position: Position) -> f64 {
        let played = self
            .games
            .iter()
            .filter(|game| game.position == Some(position))
            .count();
        share(played, self.positioned_games())
    }

    // the roles played most, at most n of them. of two roles played equally
    // often, the one played more recently comes first.
    pub fn top_roles(&self, n: usize) -> Vec<Position> {
        // each role and its games, in the order of the newest game in it.
        let mut roles: Vec<(Position, usize)> = Vec::new();
        for position in self.games.iter().filter_map(|game| game.position) {
            match roles.iter_mut().find(|(role, _)| *role == position) {
                Some((_, games)) => *games += 1,
                None => roles.push((position, 1)),
            }
        }
        // the sort is stable, so equal counts keep that order.
        roles.sort_by_key(|&(_, games)| Reverse(games));
        roles
            .into_iter()
            .take(n)
            .map(|(position, _)| position)
            .collect()
    }

    // the share of games played with exactly this pair of summoner spells, in
    // this order: players keep flash on the same key game after game.
    pub fn spell_share(&self, spells: (i64, i64)) -> f64 {
        let matching = self
            .games
            .iter()
            .filter(|game| game.spells == spells)
            .count();
        share(matching, self.games.len())
    }

    pub fn games_on(&self, champion_id: i64) -> usize {
        self.games
            .iter()
            .filter(|game| game.champion_id == champion_id)
            .count()
    }

    // losses in a row, counting back from the newest game.
    pub fn loss_streak(&self) -> usize {
        self.games.iter().take_while(|game| !game.win).count()
    }

    // the share of the newest `last` games that were won, and how many games
    // that was. none without games.
    pub fn win_rate(&self, last: usize) -> Option<(f64, usize)> {
        let games = self.newest(last);
        if games.is_empty() {
            return None;
        }
        let wins = games.iter().filter(|game| game.win).count();
        Some((share(wins, games.len()), games.len()))
    }

    // (kills + assists) / deaths over the newest `last` games, from the totals,
    // and how many games that was. a stretch without deaths divides by one.
    pub fn kda(&self, last: usize) -> Option<(f64, usize)> {
        let games = self.newest(last);
        if games.is_empty() {
            return None;
        }
        let (takedowns, deaths) = games.iter().fold((0, 0), |(takedowns, deaths), game| {
            (takedowns + game.kills + game.assists, deaths + game.deaths)
        });
        Some((takedowns as f64 / deaths.max(1) as f64, games.len()))
    }

    // the games started at or after since_ms, and how many of them were lost.
    pub fn games_since(&self, since_ms: i64) -> (usize, usize) {
        self.games
            .iter()
            .filter(|game| game.started_at_ms >= since_ms)
            .fold((0, 0), |(games, losses), game| {
                (games + 1, losses + usize::from(!game.win))
            })
    }

    // the games riot could tell the lane of.
    pub fn positioned_games(&self) -> usize {
        self.games
            .iter()
            .filter(|game| game.position.is_some())
            .count()
    }

    fn newest(&self, last: usize) -> &[GameSummary] {
        &self.games[..last.min(self.games.len())]
    }
}

// a solo/duo queue standing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rank {
    // IRON through CHALLENGER.
    pub tier: String,
    pub division: String,
    pub lp: i64,
    pub wins: i64,
    pub losses: i64,
}

// everything expose shows about one player. each part is none when its
// request failed or came back empty.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlayerStats {
    pub level: Option<i64>,
    pub game_name: Option<String>,
    pub tag_line: Option<String>,
    pub rank: Option<Rank>,
    // the ranked stats came back without a solo/duo rank. a rank that is none
    // with this false was never loaded, so it says nothing.
    pub unranked: bool,
    pub history: Option<PlayerHistory>,
    // champion id to mastery points.
    pub mastery: Option<HashMap<i64, i64>>,
}

impl PlayerStats {
    // puts the four responses together, none standing for a request that
    // failed. an empty answer counts as missing too: a level of 0, a blank
    // name, a history without a single game, a mastery list without a single
    // champion. every real account has some, and reading an empty mastery
    // list as "never played anything" would call every pick a first time.
    pub fn from_parts(
        summoner: Option<Summoner>,
        ranked: Option<RankedStats>,
        history: Option<MatchHistoryList>,
        mastery: Option<Vec<ChampionMastery>>,
        puuid: &str,
    ) -> PlayerStats {
        let (level, game_name, tag_line) = match summoner {
            Some(summoner) => (
                (summoner.summoner_level > 0).then_some(summoner.summoner_level),
                non_empty(summoner.game_name),
                non_empty(summoner.tag_line),
            ),
            None => (None, None, None),
        };
        let rank = ranked.as_ref().and_then(RankedStats::solo_rank);
        // an answer without a single queue is as empty as no answer.
        let answered = ranked
            .as_ref()
            .is_some_and(|ranked| !ranked.queue_map.is_empty() || !ranked.queues.is_empty());
        PlayerStats {
            level,
            game_name,
            tag_line,
            unranked: answered && rank.is_none(),
            rank,
            history: history
                .filter(|list| !list.games.games.is_empty())
                .map(|list| PlayerHistory::from_matches(&list, puuid)),
            mastery: mastery.filter(|list| !list.is_empty()).map(|list| {
                list.into_iter()
                    .map(|entry| (entry.champion_id, entry.champion_points))
                    .collect()
            }),
        }
    }
}

// one player's stats, and whether every request went through. only complete
// stats are worth keeping for later lobbies.
#[derive(Debug, Clone, PartialEq)]
pub struct Fetched {
    pub stats: PlayerStats,
    pub complete: bool,
}

// the quick half of one player's stats: the summoner and ranked answers,
// which come back long before the match history.
#[derive(Debug, Clone, Default)]
pub struct Quick {
    puuid: String,
    summoner: Option<Summoner>,
    ranked: Option<RankedStats>,
}

impl Quick {
    // what the quick half alone says: level, name and rank, no history.
    pub fn stats(&self) -> PlayerStats {
        PlayerStats::from_parts(
            self.summoner.clone(),
            self.ranked.clone(),
            None,
            None,
            &self.puuid,
        )
    }
}

// fetches a player's summoner and ranked stats, both at once, each once the
// limit, shared by every player being fetched, has room for it.
pub async fn fetch_quick(client: &LcuClient, puuid: &str, limit: &Semaphore) -> Quick {
    // nothing but a well formed puuid goes into a request path.
    if parse_uuid(puuid).is_none() {
        return Quick {
            puuid: puuid.to_string(),
            ..Quick::default()
        };
    }
    let (summoner, ranked) = tokio::join!(
        limited(limit, client.summoner_by_puuid(puuid)),
        limited(limit, client.ranked_stats(puuid)),
    );
    Quick {
        puuid: puuid.to_string(),
        summoner: loaded("summoner", summoner),
        ranked: loaded("ranked stats", ranked),
    }
}

// fetches the rest of a player's stats, the match history and the mastery,
// and puts them together with the quick half. complete only when all four
// requests went through.
pub async fn fetch_rest(
    client: &LcuClient,
    puuid: &str,
    limit: &Semaphore,
    quick: Quick,
) -> Fetched {
    if parse_uuid(puuid).is_none() {
        return Fetched {
            stats: PlayerStats::default(),
            complete: false,
        };
    }
    let (history, mastery) = tokio::join!(
        limited(limit, client.match_history(puuid, HISTORY_GAMES)),
        limited(limit, client.champion_mastery(puuid)),
    );
    let history = loaded("match history", history);
    let mastery = loaded("champion mastery", mastery);
    let complete = quick.summoner.is_some()
        && quick.ranked.is_some()
        && history.is_some()
        && mastery.is_some();
    Fetched {
        stats: PlayerStats::from_parts(quick.summoner, quick.ranked, history, mastery, puuid),
        complete,
    }
}

// runs a request once the limit has a permit free, holding it until the
// request is done.
async fn limited<T>(limit: &Semaphore, request: impl Future<Output = T>) -> T {
    // acquire only fails on a closed semaphore, and this one is never closed;
    // the request still goes out if it were.
    let _permit = limit.acquire().await;
    request.await
}

// stats fetched in earlier lobbies, keyed by puuid and reused for ten minutes.
#[derive(Debug, Default)]
pub struct StatsCache {
    entries: HashMap<String, (Instant, PlayerStats)>,
}

impl StatsCache {
    pub fn get(&self, puuid: &str, now: Instant) -> Option<PlayerStats> {
        let (stored_at, stats) = self.entries.get(&puuid.to_ascii_lowercase())?;
        fresh(*stored_at, now).then(|| stats.clone())
    }

    // stores the stats and drops whatever has expired, so the cache never
    // holds more than the last few lobbies.
    pub fn insert(&mut self, puuid: String, stats: PlayerStats, now: Instant) {
        self.entries
            .retain(|_, (stored_at, _)| fresh(*stored_at, now));
        self.entries
            .insert(puuid.to_ascii_lowercase(), (now, stats));
    }
}

// ---- inbound: the league client's stats endpoints, only the fields read ----

// /lol-match-history/v1/products/lol/{puuid}/matches
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MatchHistoryList {
    #[serde(deserialize_with = "lenient")]
    pub games: MatchHistoryGames,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MatchHistoryGames {
    #[serde(deserialize_with = "lenient")]
    pub games: Vec<MatchHistoryGame>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MatchHistoryGame {
    // epoch milliseconds.
    #[serde(deserialize_with = "lenient")]
    pub game_creation: i64,
    // seconds.
    #[serde(deserialize_with = "lenient")]
    pub game_duration: i64,
    #[serde(deserialize_with = "lenient")]
    pub map_id: i64,
    #[serde(deserialize_with = "lenient")]
    pub queue_id: i64,
    // who each participant is. a player's own history can list only them.
    #[serde(deserialize_with = "lenient")]
    pub participant_identities: Vec<ParticipantIdentity>,
    #[serde(deserialize_with = "lenient")]
    pub participants: Vec<MatchParticipant>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ParticipantIdentity {
    #[serde(deserialize_with = "lenient")]
    pub participant_id: i64,
    #[serde(deserialize_with = "lenient")]
    pub player: ParticipantPlayer,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ParticipantPlayer {
    #[serde(deserialize_with = "lenient")]
    pub puuid: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MatchParticipant {
    #[serde(deserialize_with = "lenient")]
    pub participant_id: i64,
    #[serde(deserialize_with = "lenient")]
    pub champion_id: i64,
    #[serde(deserialize_with = "lenient")]
    pub spell1_id: i64,
    #[serde(deserialize_with = "lenient")]
    pub spell2_id: i64,
    #[serde(deserialize_with = "lenient")]
    pub stats: ParticipantStats,
    #[serde(deserialize_with = "lenient")]
    pub timeline: ParticipantTimeline,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ParticipantStats {
    #[serde(deserialize_with = "lenient")]
    pub win: bool,
    #[serde(deserialize_with = "lenient")]
    pub kills: i64,
    #[serde(deserialize_with = "lenient")]
    pub deaths: i64,
    #[serde(deserialize_with = "lenient")]
    pub assists: i64,
    #[serde(deserialize_with = "lenient")]
    pub total_minions_killed: i64,
    #[serde(deserialize_with = "lenient")]
    pub neutral_minions_killed: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ParticipantTimeline {
    // what riot inferred: TOP, JUNGLE, MIDDLE, BOTTOM or NONE for the lane,
    // SOLO, DUO, DUO_CARRY, DUO_SUPPORT or NONE for the role.
    #[serde(deserialize_with = "lenient")]
    pub lane: String,
    #[serde(deserialize_with = "lenient")]
    pub role: String,
}

// /lol-ranked/v1/ranked-stats/{puuid}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RankedStats {
    // the same entries twice: keyed by queue type, and as a list.
    #[serde(deserialize_with = "lenient")]
    pub queue_map: HashMap<String, RankedQueue>,
    #[serde(deserialize_with = "lenient")]
    pub queues: Vec<RankedQueue>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RankedQueue {
    #[serde(deserialize_with = "lenient")]
    pub queue_type: String,
    // "NONE" or empty while unranked.
    #[serde(deserialize_with = "lenient")]
    pub tier: String,
    #[serde(deserialize_with = "lenient")]
    pub division: String,
    #[serde(deserialize_with = "lenient")]
    pub league_points: i64,
    #[serde(deserialize_with = "lenient")]
    pub wins: i64,
    #[serde(deserialize_with = "lenient")]
    pub losses: i64,
}

impl RankedStats {
    // the solo/duo standing, or none while the player is unranked there.
    pub fn solo_rank(&self) -> Option<Rank> {
        let queue = self.queue_map.get(SOLO_QUEUE).or_else(|| {
            self.queues
                .iter()
                .find(|queue| queue.queue_type == SOLO_QUEUE)
        })?;
        if queue.tier.is_empty() || queue.tier.eq_ignore_ascii_case("NONE") {
            return None;
        }
        Some(Rank {
            tier: queue.tier.clone(),
            division: queue.division.clone(),
            lp: queue.league_points,
            wins: queue.wins,
            losses: queue.losses,
        })
    }
}

// /lol-champion-mastery/v1/{puuid}/champion-mastery: one entry per champion
// the player has played.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ChampionMastery {
    #[serde(deserialize_with = "lenient")]
    pub champion_id: i64,
    #[serde(deserialize_with = "lenient")]
    pub champion_points: i64,
}

// the player's side of one game, or none when the game does not show which
// participant they were.
fn summarize(game: &MatchHistoryGame, puuid: &str) -> Option<GameSummary> {
    let player = own_participant(game, puuid)?;
    let stats = &player.stats;
    let creeps = stats.total_minions_killed + stats.neutral_minions_killed;
    Some(GameSummary {
        champion_id: player.champion_id,
        win: stats.win,
        kills: stats.kills,
        deaths: stats.deaths,
        assists: stats.assists,
        started_at_ms: game.game_creation,
        duration_s: game.game_duration,
        position: played_position(
            &player.timeline.lane,
            &player.timeline.role,
            creeps,
            game.game_duration,
        ),
        spells: (player.spell1_id, player.spell2_id),
    })
}

// finds the player through the game's identities. when no identity names
// anyone, a game with a single participant is the player's own; when the
// identities name only other players, the game is not theirs at all.
fn own_participant<'a>(game: &'a MatchHistoryGame, puuid: &str) -> Option<&'a MatchParticipant> {
    let mut named = game
        .participant_identities
        .iter()
        .filter(|identity| !is_blank(&identity.player.puuid))
        .peekable();
    if named.peek().is_none() {
        return match game.participants.as_slice() {
            [only] => Some(only),
            _ => None,
        };
    }
    let identity = named.find(|identity| identity.player.puuid.eq_ignore_ascii_case(puuid))?;
    game.participants
        .iter()
        .find(|participant| participant.participant_id == identity.participant_id)
}

// the role a game was played in, from the lane and the role riot inferred.
// riot marks supports unreliably, so a bottom laner under two creeps a minute
// (minions and monsters) counts as the support as well.
fn played_position(lane: &str, role: &str, creeps: i64, duration_s: i64) -> Option<Position> {
    match lane.to_ascii_uppercase().as_str() {
        "TOP" => Some(Position::Top),
        "JUNGLE" => Some(Position::Jungle),
        "MIDDLE" | "MID" => Some(Position::Middle),
        "BOTTOM" | "BOT" => {
            let support = matches!(
                role.to_ascii_uppercase().as_str(),
                "DUO_SUPPORT" | "SUPPORT"
            );
            let minutes = duration_s.max(1) as f64 / 60.0;
            if support || creeps as f64 / minutes < SUPPORT_CREEPS_PER_MINUTE {
                Some(Position::Utility)
            } else {
                Some(Position::Bottom)
            }
        }
        // "NONE": riot could not tell.
        _ => None,
    }
}

fn share(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        0.0
    } else {
        part as f64 / whole as f64
    }
}

fn non_empty(text: String) -> Option<String> {
    (!text.is_empty()).then_some(text)
}

fn fresh(stored_at: Instant, now: Instant) -> bool {
    now.saturating_duration_since(stored_at) < CACHE_LIFETIME
}

// the response, or none once the failure is logged. the log line leaves out
// the request path and url, which carry the player's puuid.
fn loaded<T>(what: &str, result: Result<T>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            log::warn!("could not load a player's {what}: {}", failure(&error));
            None
        }
    }
}

fn failure(error: &Error) -> String {
    match error {
        Error::UnexpectedStatus { status, .. } => {
            format!("the league client returned status {status}")
        }
        Error::Http(error) if error.is_timeout() => "the request timed out".to_string(),
        Error::Http(error) if error.is_decode() => "the response could not be read".to_string(),
        _ => "the request failed".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HISTORY: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/match_history.json"
    ));
    const RANKED: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/ranked_stats.json"
    ));

    // the player whose games match_history.json holds.
    const PUUID: &str = "3f6c2a9e-8b14-4d7e-9a35-c1e0f7b24d68";

    fn matches() -> MatchHistoryList {
        serde_json::from_str(HISTORY).expect("the fixture should parse")
    }

    fn history() -> PlayerHistory {
        PlayerHistory::from_matches(&matches(), PUUID)
    }

    fn ranked() -> RankedStats {
        serde_json::from_str(RANKED).expect("the fixture should parse")
    }

    fn champions(history: &PlayerHistory) -> Vec<i64> {
        history.games.iter().map(|game| game.champion_id).collect()
    }

    // shares and ratios are floats, so they are compared with room for rounding.
    fn close(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-9
    }

    #[test]
    fn aggregates_recent_summoners_rift_games() {
        let history = history();
        // six summoner's rift games, newest first. the 240 second remake and
        // the aram game between them are gone.
        let started: Vec<i64> = history
            .games
            .iter()
            .map(|game| game.started_at_ms)
            .collect();
        assert_eq!(
            started,
            [
                1_795_170_000_000,
                1_795_158_000_000,
                1_795_150_000_000,
                1_795_140_000_000,
                1_795_130_000_000,
                1_795_120_000_000,
            ]
        );
        assert_eq!(champions(&history), [64, 121, 412, 222, 103, 64]);

        // in the second game the player is participant 3, not participant 1
        // (who won that game on another champion).
        assert_eq!(
            history.games[1],
            GameSummary {
                champion_id: 121,
                win: false,
                kills: 2,
                deaths: 8,
                assists: 3,
                started_at_ms: 1_795_158_000_000,
                duration_s: 2101,
                position: Some(Position::Jungle),
                spells: (11, 4),
            }
        );

        // the puuid matches in any case, and the order the client lists the
        // games in does not matter.
        let mut reversed = matches();
        reversed.games.games.reverse();
        assert_eq!(
            PlayerHistory::from_matches(&reversed, &PUUID.to_uppercase()),
            history
        );
    }

    #[test]
    fn falls_back_to_the_only_participant() {
        // without identities a game with one participant is still that
        // player's, while the game with three cannot be told apart.
        let mut list = matches();
        for game in &mut list.games.games {
            game.participant_identities.clear();
        }
        let history = PlayerHistory::from_matches(&list, PUUID);
        assert_eq!(champions(&history), [64, 412, 222, 103, 64]);

        // identities that name only someone else mean the game is not theirs.
        let mut list = matches();
        list.games.games[0].participant_identities[0].player.puuid =
            "a8d3f0c2-5e71-4b96-8c2d-e4f1a7b09c35".to_string();
        let history = PlayerHistory::from_matches(&list, PUUID);
        assert_eq!(champions(&history), [121, 412, 222, 103, 64]);
    }

    #[test]
    fn detects_positions() {
        let history = history();
        let positions: Vec<Option<Position>> =
            history.games.iter().map(|game| game.position).collect();
        assert_eq!(
            positions,
            [
                Some(Position::Jungle),
                Some(Position::Jungle),
                // the bottom lane game with 25 creeps in 27 minutes.
                Some(Position::Utility),
                // the bottom lane game with 222 creeps in 32 minutes.
                Some(Position::Bottom),
                Some(Position::Middle),
                // riot could not tell the lane.
                None,
            ]
        );

        // the rules one by one. 59 creeps in 30 minutes is under two a minute.
        assert_eq!(
            played_position("TOP", "SOLO", 200, 1800),
            Some(Position::Top)
        );
        assert_eq!(
            played_position("JUNGLE", "NONE", 180, 1800),
            Some(Position::Jungle)
        );
        assert_eq!(
            played_position("MID", "SOLO", 200, 1800),
            Some(Position::Middle)
        );
        assert_eq!(
            played_position("MIDDLE", "SOLO", 200, 1800),
            Some(Position::Middle)
        );
        assert_eq!(
            played_position("BOTTOM", "DUO_SUPPORT", 200, 1800),
            Some(Position::Utility)
        );
        assert_eq!(
            played_position("BOT", "SUPPORT", 200, 1800),
            Some(Position::Utility)
        );
        assert_eq!(
            played_position("BOTTOM", "DUO", 59, 1800),
            Some(Position::Utility)
        );
        assert_eq!(
            played_position("BOTTOM", "DUO", 60, 1800),
            Some(Position::Bottom)
        );
        assert_eq!(
            played_position("BOT", "CARRY", 210, 1800),
            Some(Position::Bottom)
        );
        assert_eq!(played_position("NONE", "NONE", 200, 1800), None);
        assert_eq!(played_position("", "", 200, 1800), None);

        // champ select names the assigned position in lowercase.
        assert_eq!(Position::from_assigned("top"), Some(Position::Top));
        assert_eq!(Position::from_assigned("jungle"), Some(Position::Jungle));
        assert_eq!(Position::from_assigned("MIDDLE"), Some(Position::Middle));
        assert_eq!(Position::from_assigned("Bottom"), Some(Position::Bottom));
        assert_eq!(Position::from_assigned("utility"), Some(Position::Utility));
        assert_eq!(Position::from_assigned(""), None);
        assert_eq!(Position::from_assigned("mid"), None);

        let labels: Vec<&str> = Position::ALL
            .iter()
            .map(|position| position.label())
            .collect();
        assert_eq!(labels, ["top", "jungle", "mid", "bot", "support"]);
        assert_eq!(
            serde_json::to_value(Position::ALL).unwrap(),
            serde_json::json!(["top", "jungle", "middle", "bottom", "utility"])
        );
    }

    #[test]
    fn computes_streak_win_rate_and_kda() {
        let history = history();
        // two losses, then a win. the remake and the aram game, both losses,
        // would have made it four.
        assert_eq!(history.loss_streak(), 2);

        let (rate, games) = history.win_rate(20).expect("there are games");
        assert!(close(rate, 0.5), "{rate}");
        assert_eq!(games, 6);
        // (27 kills + 48 assists) / 30 deaths.
        let (kda, games) = history.kda(10).expect("there are games");
        assert!(close(kda, 2.5), "{kda}");
        assert_eq!(games, 6);

        // only the newest games count.
        let (rate, games) = history.win_rate(2).expect("there are games");
        assert!(close(rate, 0.0), "{rate}");
        assert_eq!(games, 2);
        let (kda, games) = history.kda(1).expect("there are games");
        assert!(close(kda, 10.0 / 7.0), "{kda}");
        assert_eq!(games, 1);

        // a game without deaths divides by one.
        let deathless = PlayerHistory {
            games: vec![GameSummary {
                kills: 3,
                assists: 4,
                ..GameSummary::default()
            }],
        };
        let (kda, _) = deathless.kda(10).expect("there is a game");
        assert!(close(kda, 7.0), "{kda}");

        // games and losses since a moment, that moment included.
        assert_eq!(history.games_since(1_795_150_000_000), (3, 2));
        assert_eq!(history.games_since(1_795_170_000_001), (0, 0));

        let empty = PlayerHistory::default();
        assert_eq!(empty.loss_streak(), 0);
        assert_eq!(empty.win_rate(20), None);
        assert_eq!(empty.kda(10), None);
    }

    #[test]
    fn counts_roles_spells_and_champions() {
        let history = history();
        // the game without a lane does not count toward roles.
        assert_eq!(history.positioned_games(), 5);
        assert!(close(history.role_share(Position::Jungle), 0.4));
        assert!(close(history.role_share(Position::Utility), 0.2));
        assert!(close(history.role_share(Position::Top), 0.0));
        // three roles tie at one game each; the most recent of them comes first.
        assert_eq!(history.top_roles(2), [Position::Jungle, Position::Utility]);
        assert_eq!(
            history.top_roles(5),
            [
                Position::Jungle,
                Position::Utility,
                Position::Bottom,
                Position::Middle,
            ]
        );

        // the exact pair, in order.
        assert!(close(history.spell_share((11, 4)), 0.5));
        assert!(close(history.spell_share((4, 11)), 0.0));
        assert!(close(history.spell_share((4, 14)), 2.0 / 6.0));

        // the aram game on champion 22 is gone.
        assert_eq!(history.games_on(64), 2);
        assert_eq!(history.games_on(121), 1);
        assert_eq!(history.games_on(22), 0);

        let empty = PlayerHistory::default();
        assert_eq!(empty.positioned_games(), 0);
        assert!(close(empty.role_share(Position::Jungle), 0.0));
        assert!(close(empty.spell_share((11, 4)), 0.0));
        assert!(empty.top_roles(2).is_empty());
    }

    #[test]
    fn reads_solo_queue_rank() {
        assert_eq!(
            ranked().solo_rank(),
            Some(Rank {
                tier: "GOLD".to_string(),
                division: "II".to_string(),
                lp: 54,
                wins: 45,
                losses: 40,
            })
        );

        // unranked in solo queue: riot still sends the entry, with tier NONE,
        // or with an empty tier.
        for tier in ["NONE", ""] {
            let mut value: serde_json::Value = serde_json::from_str(RANKED).unwrap();
            value["queueMap"]["RANKED_SOLO_5x5"]["tier"] = tier.into();
            value["queues"][0]["tier"] = tier.into();
            let unranked: RankedStats = serde_json::from_value(value).unwrap();
            assert_eq!(unranked.solo_rank(), None, "tier {tier:?}");
        }

        // the queues list stands in for a missing queueMap.
        let mut value: serde_json::Value = serde_json::from_str(RANKED).unwrap();
        value.as_object_mut().unwrap().remove("queueMap");
        let listed: RankedStats = serde_json::from_value(value).unwrap();
        assert_eq!(listed.solo_rank().map(|rank| rank.lp), Some(54));

        assert_eq!(RankedStats::default().solo_rank(), None);
    }

    #[test]
    fn cache_expires_after_ten_minutes() {
        let start = Instant::now();
        let mut cache = StatsCache::default();
        let stats = PlayerStats {
            level: Some(187),
            ..PlayerStats::default()
        };
        cache.insert(PUUID.to_string(), stats.clone(), start);

        assert_eq!(cache.get(PUUID, start), Some(stats.clone()));
        assert_eq!(
            cache.get(&PUUID.to_uppercase(), start + Duration::from_secs(599)),
            Some(stats)
        );
        assert_eq!(cache.get(PUUID, start + Duration::from_secs(601)), None);
        assert_eq!(
            cache.get("a8d3f0c2-5e71-4b96-8c2d-e4f1a7b09c35", start),
            None
        );
    }

    #[test]
    fn leaves_out_bots_customs_and_event_modes() {
        let mut list = matches();
        // the newest game, ranked in the fixture, becomes a game against bots,
        // the next one an urf game.
        let newest = list
            .games
            .games
            .iter_mut()
            .max_by_key(|game| game.game_creation)
            .expect("the fixture has games");
        newest.queue_id = 870;
        let history = PlayerHistory::from_matches(&list, PUUID);
        assert_eq!(champions(&history), [121, 412, 222, 103, 64]);
        for game in &mut list.games.games {
            game.queue_id = 1900;
        }
        assert!(PlayerHistory::from_matches(&list, PUUID).games.is_empty());
    }

    #[test]
    fn tells_unranked_from_unknown() {
        let ranked_without_solo = RankedStats {
            queues: vec![RankedQueue {
                queue_type: "RANKED_FLEX_SR".to_string(),
                tier: "SILVER".to_string(),
                ..RankedQueue::default()
            }],
            ..RankedStats::default()
        };
        let stats = PlayerStats::from_parts(None, Some(ranked_without_solo), None, None, PUUID);
        assert_eq!((stats.rank.is_none(), stats.unranked), (true, true));
        // the ranked request failed: nothing is known either way.
        let stats = PlayerStats::from_parts(None, None, None, None, PUUID);
        assert_eq!((stats.rank.is_none(), stats.unranked), (true, false));
        let stats = PlayerStats::from_parts(None, Some(ranked()), None, None, PUUID);
        assert_eq!((stats.rank.is_some(), stats.unranked), (true, false));
    }

    #[test]
    fn partial_stats_when_history_fails() {
        let summoner = Summoner {
            game_name: "Sample Teammate".to_string(),
            tag_line: "NA1".to_string(),
            summoner_level: 187,
        };
        let mastery = vec![
            ChampionMastery {
                champion_id: 64,
                champion_points: 48_213,
            },
            ChampionMastery {
                champion_id: 121,
                champion_points: 9_870,
            },
        ];

        // the match history request failed; everything else arrived.
        let stats =
            PlayerStats::from_parts(Some(summoner), Some(ranked()), None, Some(mastery), PUUID);
        assert_eq!(stats.level, Some(187));
        assert_eq!(stats.game_name.as_deref(), Some("Sample Teammate"));
        assert_eq!(stats.tag_line.as_deref(), Some("NA1"));
        assert_eq!(
            stats.rank.map(|rank| (rank.tier, rank.lp)),
            Some(("GOLD".to_string(), 54))
        );
        assert_eq!(stats.history, None);
        assert_eq!(
            stats.mastery,
            Some(HashMap::from([(64, 48_213), (121, 9_870)]))
        );

        // and the other way around: only the match history arrived.
        let stats = PlayerStats::from_parts(None, None, Some(matches()), None, PUUID);
        assert_eq!(
            stats,
            PlayerStats {
                history: Some(history()),
                ..PlayerStats::default()
            }
        );

        // empty answers carry nothing: no level, no name, no mastery at all
        // and no games at all mean the data is missing, not that it is zero.
        let blank = Summoner {
            game_name: String::new(),
            tag_line: String::new(),
            summoner_level: 0,
        };
        let stats = PlayerStats::from_parts(
            Some(blank),
            Some(RankedStats::default()),
            Some(MatchHistoryList::default()),
            Some(Vec::new()),
            PUUID,
        );
        assert_eq!(stats, PlayerStats::default());
    }
}
