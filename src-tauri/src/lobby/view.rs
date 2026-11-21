// what the frontend shows for a champ select lobby, emitted as expose://lobby
// and answered by get_lobby. these shapes are the contract with
// src/types/index.ts: camelCase fields, and a missing piece is null rather
// than left out. build_view puts the session and what expose has learned
// about the lobby into one view, without any io.

use std::collections::HashSet;

use serde::Serialize;

use super::flags::{flags_for, summarize, Flag, FlagInput, Summary, KDA_GAMES, WIN_RATE_GAMES};
use super::identity::{Confidence, Identity, MatchKind};
use super::session::Session;
use super::stats::{GameSummary, PlayerStats, Position};
use super::{canonical_puuid, LobbyState};
use crate::scout;

// how many recent games a row shows.
const RECENT_GAMES: usize = 10;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LobbyView {
    pub game_id: i64,
    pub queue_id: i64,
    // the raw timer phase: PLANNING, BAN_PICK, FINALIZATION and so on.
    pub phase: String,
    // time left in the phase when the client sent the session.
    pub time_left_ms: i64,
    // riot hides at least one teammate's name.
    pub anonymous: bool,
    // your team in slot order, you included.
    pub players: Vec<PlayerView>,
    // names in the champ select chat that no slot has.
    pub unmatched: Vec<NameView>,
    // from your teammates' flags, never your own.
    pub summary: Summary,
    pub dodge: DodgeView,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerView {
    // stays with the player for the whole lobby (see SessionPlayer::key).
    pub key: String,
    pub cell_id: i64,
    pub is_self: bool,
    // the client's stand-in name, for hidden players only.
    pub alias: Option<String>,
    pub position: Option<Position>,
    pub is_autofilled: bool,
    // picked or hovered, else declared; 0 while there is none.
    pub champion_id: i64,
    pub spells: (i64, i64),
    pub identity: Option<IdentityView>,
    pub stats: Option<StatsView>,
    // matched, and their stats are on the way.
    pub stats_pending: bool,
    // red first, then amber, then info. none on your own row.
    pub flags: Vec<Flag>,
    // their profile on the chosen scout site.
    pub scout_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityView {
    pub game_name: String,
    pub tag_line: String,
    pub kind: MatchKind,
    // always high for the exact kinds.
    pub confidence: Confidence,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NameView {
    pub game_name: String,
    pub tag_line: String,
}

// each piece is none when its request failed or had nothing to say.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsView {
    pub level: Option<i64>,
    // solo/duo; none while unranked there.
    pub rank: Option<RankView>,
    // the newest summoner's rift games, newest first.
    pub recent: Vec<GameView>,
    // over the games the bad kda flag looks at.
    pub kda: Option<f64>,
    // 0 to 1, over the games the low win rate flag looks at.
    pub win_rate: Option<f64>,
    // recent games and mastery points on the slot's champion, none while the
    // slot has no champion.
    pub champion_games: Option<usize>,
    pub champion_points: Option<i64>,
    // most played first.
    pub roles: Vec<RoleShare>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankView {
    // IRON through CHALLENGER.
    pub tier: String,
    pub division: String,
    pub lp: i64,
    pub wins: i64,
    pub losses: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameView {
    pub champion_id: i64,
    pub win: bool,
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub started_at_ms: i64,
    pub duration_s: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleShare {
    pub position: Position,
    // of the games with a known role, 0 to 1.
    pub share: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DodgeView {
    // a last-second dodge is armed for this lobby.
    pub armed: bool,
    pub lead_ms: i64,
}

// where the scout links point.
#[derive(Debug, Clone)]
pub struct ScoutContext {
    // "opgg" or "ugg".
    pub provider: String,
    // the op.gg web slug ("na"); none until the client has said.
    pub region: Option<String>,
}

// the view of the lobby as expose knows it now. the only change it makes to
// the state is pinning the alias of a hidden player seen for the first time.
pub fn build_view(
    session: &Session,
    state: &mut LobbyState,
    champion_name: &dyn Fn(i64) -> Option<String>,
    scout: &ScoutContext,
    dodge: DodgeView,
    now_ms: i64,
) -> LobbyView {
    let mut players = Vec::with_capacity(session.my_team.len());
    for (slot, player) in session.team_in_slot_order().into_iter().enumerate() {
        let key = player.key();
        let is_self = player.cell_id == session.local_player_cell_id;
        // the client names a hidden player after the slot they first held.
        let alias = player
            .is_hidden()
            .then(|| state.aliases.alias(&key, slot).to_string());
        let position = Position::from_assigned(&player.assigned_position);
        let champion_id = player.champion();

        let identity = state.identities.get(&key);
        let puuid = identity.and_then(|identity| canonical_puuid(&identity.puuid));
        let stats = puuid.as_ref().and_then(|puuid| state.stats.get(puuid));
        let stats_pending =
            stats.is_none() && puuid.is_some_and(|puuid| state.pending.contains(&puuid));
        let identity = identity.map(|identity| identity_view(identity, stats));

        // flags speak about teammates, and only once their stats are in.
        let flags = match stats {
            Some(stats) if !is_self => {
                let champion = champion_name(champion_id);
                flags_for(&FlagInput {
                    position,
                    is_autofilled: player.is_autofilled,
                    champion_id,
                    champion_name: champion.as_deref(),
                    stats,
                    now_ms,
                })
            }
            _ => Vec::new(),
        };
        let scout_url = identity
            .as_ref()
            .filter(|identity| !identity.game_name.is_empty())
            .map(|identity| {
                scout::profile_url(
                    &scout.provider,
                    scout.region.as_deref(),
                    &identity.game_name,
                    &identity.tag_line,
                )
            });

        players.push(PlayerView {
            key,
            cell_id: player.cell_id,
            is_self,
            alias,
            position,
            is_autofilled: player.is_autofilled,
            champion_id,
            spells: (player.spell1_id, player.spell2_id),
            identity,
            stats: stats.map(|stats| stats_view(stats, champion_id)),
            stats_pending,
            flags,
            scout_url,
        });
    }

    let teammates: Vec<Vec<Flag>> = players
        .iter()
        .filter(|player| !player.is_self)
        .map(|player| player.flags.clone())
        .collect();
    LobbyView {
        game_id: session.game_id,
        queue_id: session.queue_id,
        phase: session.timer.phase.clone(),
        time_left_ms: session.timer.adjusted_time_left_in_phase,
        anonymous: session.is_anonymous(),
        players,
        unmatched: unmatched(state),
        summary: summarize(&teammates),
        dodge,
    }
}

impl LobbyView {
    // everyone in the lobby with a name, as (name, tag): the slots in order,
    // then the names no slot has, each riot id once.
    pub fn names(&self) -> Vec<(String, String)> {
        let matched = self
            .players
            .iter()
            .filter_map(|player| player.identity.as_ref())
            .map(|identity| (identity.game_name.clone(), identity.tag_line.clone()));
        let left_over = self
            .unmatched
            .iter()
            .map(|name| (name.game_name.clone(), name.tag_line.clone()));
        let mut seen = HashSet::new();
        matched
            .chain(left_over)
            .filter(|(name, _)| !name.is_empty())
            .filter(|(name, tag)| seen.insert((name.to_lowercase(), tag.to_lowercase())))
            .collect()
    }
}

// the name to show. a player the session names can come without one (a party
// member the chat does not list, say), and then their summoner name stands in.
fn identity_view(identity: &Identity, stats: Option<&PlayerStats>) -> IdentityView {
    let (game_name, tag_line) = match stats {
        Some(stats) if identity.game_name.is_empty() => (
            stats.game_name.clone().unwrap_or_default(),
            stats.tag_line.clone().unwrap_or_default(),
        ),
        _ => (identity.game_name.clone(), identity.tag_line.clone()),
    };
    IdentityView {
        game_name,
        tag_line,
        kind: identity.kind,
        confidence: identity.confidence,
    }
}

fn stats_view(stats: &PlayerStats, champion_id: i64) -> StatsView {
    let history = stats.history.as_ref();
    let on_champion = champion_id != 0;
    StatsView {
        level: stats.level,
        rank: stats.rank.as_ref().map(|rank| RankView {
            tier: rank.tier.clone(),
            division: rank.division.clone(),
            lp: rank.lp,
            wins: rank.wins,
            losses: rank.losses,
        }),
        recent: history.map_or_else(Vec::new, |history| {
            history
                .games
                .iter()
                .take(RECENT_GAMES)
                .map(game_view)
                .collect()
        }),
        kda: history
            .and_then(|history| history.kda(KDA_GAMES))
            .map(|(kda, _)| kda),
        win_rate: history
            .and_then(|history| history.win_rate(WIN_RATE_GAMES))
            .map(|(rate, _)| rate),
        champion_games: history
            .filter(|_| on_champion)
            .map(|history| history.games_on(champion_id)),
        champion_points: stats
            .mastery
            .as_ref()
            .filter(|_| on_champion)
            .map(|mastery| mastery.get(&champion_id).copied().unwrap_or(0)),
        roles: history.map_or_else(Vec::new, |history| {
            history
                .top_roles(Position::ALL.len())
                .into_iter()
                .map(|position| RoleShare {
                    position,
                    share: history.role_share(position),
                })
                .collect()
        }),
    }
}

fn game_view(game: &GameSummary) -> GameView {
    GameView {
        champion_id: game.champion_id,
        win: game.win,
        kills: game.kills,
        deaths: game.deaths,
        assists: game.assists,
        started_at_ms: game.started_at_ms,
        duration_s: game.duration_s,
    }
}

// the chat names that no identity uses, in the order the chat lists them.
fn unmatched(state: &LobbyState) -> Vec<NameView> {
    let used: HashSet<String> = state
        .identities
        .values()
        .filter_map(|identity| canonical_puuid(&identity.puuid))
        .collect();
    state
        .candidates
        .iter()
        .filter(|candidate| !candidate.game_name.is_empty())
        .filter(|candidate| {
            canonical_puuid(&candidate.puuid).is_none_or(|puuid| !used.contains(&puuid))
        })
        .map(|candidate| NameView {
            game_name: candidate.game_name.clone(),
            tag_line: candidate.tag_line.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::{json, Value};

    use super::*;
    use crate::lobby::flags::{FlagKind, Severity};
    use crate::lobby::identity::{Candidate, Confidence, Identity, MatchKind};
    use crate::lobby::stats::{GameSummary, PlayerHistory, PlayerStats, Position, Rank};
    use crate::lobby::{resolve, LobbyState};

    const RANKED: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/session_ranked.json"
    ));
    const NOW: i64 = 1_795_180_000_000;
    const HOUR_MS: i64 = 3_600_000;

    // the players in session_ranked.json by key: you by your puuid, the four
    // hidden teammates by their obfuscated puuids.
    const YOU: &str = "d45f9fb2-21da-4b64-954b-261f303c8054";
    const TOP: &str = "8c29b6f9-de1a-44c1-ab55-d6ac857a23e0";
    const JUNGLE: &str = "a1490aab-4a71-4b41-befd-50f2d7c07340";
    const BOTTOM: &str = "6dd19584-53a6-4f75-948c-88ab38d40f21";
    const SUPPORT: &str = "f355dab7-17a7-41a5-908d-73bcf648cecb";
    // made-up real puuids for names in the chat.
    const REAL: [&str; 3] = [
        "5b0e7c1a-93d4-4f62-8a1e-2c7d9f04b6a3",
        "c7f2a915-0d6b-4e38-b4c1-7a5e3d92f0b8",
        "1e9d4b70-6a2c-4c85-9f13-b8e0a6c4d572",
    ];

    fn ranked() -> Session {
        serde_json::from_str(RANKED).expect("the fixture should parse")
    }

    fn scout() -> ScoutContext {
        ScoutContext {
            provider: "opgg".to_string(),
            region: Some("na".to_string()),
        }
    }

    fn champion_name(champion_id: i64) -> Option<String> {
        (champion_id == 266).then(|| "Aatrox".to_string())
    }

    fn build(session: &Session, state: &mut LobbyState) -> LobbyView {
        let dodge = DodgeView {
            armed: false,
            lead_ms: 1500,
        };
        build_view(session, state, &champion_name, &scout(), dodge, NOW)
    }

    fn player<'a>(view: &'a LobbyView, key: &str) -> &'a PlayerView {
        view.players
            .iter()
            .find(|player| player.key == key)
            .expect("the player is in the lobby")
    }

    fn name(view: &LobbyView, key: &str) -> Option<String> {
        player(view, key)
            .identity
            .as_ref()
            .map(|identity| identity.game_name.clone())
    }

    fn matched(puuid: &str, game_name: &str, kind: MatchKind) -> Identity {
        Identity {
            puuid: puuid.to_string(),
            game_name: game_name.to_string(),
            tag_line: "NA1".to_string(),
            kind,
            confidence: Confidence::High,
        }
    }

    fn candidate(puuid: &str, game_name: &str) -> Candidate {
        Candidate {
            puuid: puuid.to_string(),
            game_name: game_name.to_string(),
            tag_line: "NA1".to_string(),
        }
    }

    // a top laner's games on aatrox two days ago, newest first, "W" for a
    // win, each a 3.0 kda.
    fn top_laner(results: &str) -> PlayerStats {
        let games = results
            .chars()
            .map(|result| GameSummary {
                champion_id: 266,
                win: result == 'W',
                kills: 6,
                deaths: 4,
                assists: 6,
                started_at_ms: NOW - 48 * HOUR_MS,
                duration_s: 1800,
                position: Some(Position::Top),
                spells: (12, 4),
            })
            .collect();
        PlayerStats {
            level: Some(212),
            game_name: Some("Sample Teammate".to_string()),
            tag_line: Some("NA1".to_string()),
            rank: Some(Rank {
                tier: "GOLD".to_string(),
                division: "II".to_string(),
                lp: 54,
                wins: 45,
                losses: 40,
            }),
            history: Some(PlayerHistory { games }),
            mastery: Some(HashMap::from([(266, 52_000)])),
        }
    }

    fn close(actual: Option<f64>, expected: f64) -> bool {
        actual.is_some_and(|actual| (actual - expected).abs() < 1e-9)
    }

    #[test]
    fn view_without_participants_keeps_aliases() {
        let session = ranked();
        let mut state = LobbyState::default();
        state.enter(session.game_id);
        state.session = Some(session.clone());
        // what the lobby comes to when the riot client lists nobody.
        resolve(&mut state, true);
        let view = build(&session, &mut state);

        assert_eq!((view.game_id, view.queue_id), (5_123_456_789, 420));
        assert_eq!(
            (view.phase.as_str(), view.time_left_ms),
            ("BAN_PICK", 27_412)
        );
        assert!(view.anonymous);
        let slots: Vec<(i64, Option<&str>, Option<Position>, i64)> = view
            .players
            .iter()
            .map(|player| {
                (
                    player.cell_id,
                    player.alias.as_deref(),
                    player.position,
                    player.champion_id,
                )
            })
            .collect();
        assert_eq!(
            slots,
            [
                (0, Some("Gromp"), Some(Position::Top), 266),
                (1, Some("Krug"), Some(Position::Jungle), 64),
                // your own slot has no alias.
                (2, None, Some(Position::Middle), 103),
                // only declared so far.
                (3, Some("Raptor"), Some(Position::Bottom), 222),
                (4, Some("Scuttle Crab"), Some(Position::Utility), 412),
            ]
        );
        assert!(player(&view, SUPPORT).is_autofilled);
        assert_eq!(player(&view, BOTTOM).spells, (7, 4));

        // no hidden teammate has a name, stats on the way or flags.
        for key in [TOP, JUNGLE, BOTTOM, SUPPORT] {
            let teammate = player(&view, key);
            assert!(!teammate.is_self);
            assert_eq!(teammate.identity, None);
            assert_eq!(teammate.stats, None);
            assert!(!teammate.stats_pending);
            assert!(teammate.flags.is_empty());
            assert_eq!(teammate.scout_url, None);
        }
        // you are still you.
        let me = player(&view, YOU);
        assert!(me.is_self);
        assert_eq!(
            me.identity.as_ref().map(|identity| identity.kind),
            Some(MatchKind::Own)
        );
        assert_eq!(name(&view, YOU).as_deref(), Some("Example Player"));
        assert_eq!(
            me.scout_url.as_deref(),
            Some("https://www.op.gg/summoners/na/Example%20Player-EXP")
        );
        assert!(view.unmatched.is_empty());
        assert_eq!(view.summary.label, "no red flags");
    }

    #[test]
    fn identity_follows_player_after_swap() {
        let mut session = ranked();
        let mut state = LobbyState::default();
        state.identities.insert(
            TOP.to_string(),
            matched(REAL[0], "Top Laner", MatchKind::Key),
        );
        state.identities.insert(
            JUNGLE.to_string(),
            matched(REAL[1], "Jungler", MatchKind::Key),
        );
        state.stats.insert(REAL[0].to_string(), top_laner("WWLW"));
        let before = build(&session, &mut state);
        assert_eq!(player(&before, TOP).alias.as_deref(), Some("Gromp"));
        assert_eq!(player(&before, JUNGLE).alias.as_deref(), Some("Krug"));

        // a pick-order swap: the top laner and the jungler trade cells.
        for teammate in &mut session.my_team {
            teammate.cell_id = match teammate.cell_id {
                0 => 1,
                1 => 0,
                cell => cell,
            };
        }
        let after = build(&session, &mut state);
        // the jungler comes first now,
        assert_eq!(after.players[0].key, JUNGLE);
        // and both keep their alias, their name and their stats.
        let top = player(&after, TOP);
        assert_eq!(top.cell_id, 1);
        assert_eq!(top.alias.as_deref(), Some("Gromp"));
        assert_eq!(name(&after, TOP).as_deref(), Some("Top Laner"));
        assert!(top.stats.is_some());
        let jungle = player(&after, JUNGLE);
        assert_eq!(jungle.cell_id, 0);
        assert_eq!(jungle.alias.as_deref(), Some("Krug"));
        assert_eq!(name(&after, JUNGLE).as_deref(), Some("Jungler"));
        assert_eq!(jungle.stats, None);
    }

    #[test]
    fn new_game_id_resets_lobby() {
        let session = ranked();
        let mut state = LobbyState::default();
        // the first session seen starts a lobby.
        assert!(state.enter(session.game_id));
        state.session = Some(session.clone());
        state.candidates = vec![candidate(REAL[0], "Top Laner")];
        state.identities.insert(
            TOP.to_string(),
            matched(REAL[0], "Top Laner", MatchKind::Key),
        );
        state.stats.insert(REAL[0].to_string(), top_laner("W"));
        state.pending.insert(REAL[1].to_string());
        build(&session, &mut state);

        // later updates of the same lobby keep it.
        assert!(!state.enter(session.game_id));
        assert_eq!(state.identities.len(), 1);
        assert_eq!(state.aliases.alias(TOP, 4), "Gromp");

        // a requeue brings a new game id, and nothing of the last lobby stays.
        assert!(state.enter(session.game_id + 1));
        assert_eq!(state.game_id, session.game_id + 1);
        assert!(state.identities.is_empty());
        assert!(state.candidates.is_empty());
        assert!(state.stats.is_empty());
        assert!(state.pending.is_empty());
        assert!(state.session.is_none());
        // aliases are handed out afresh.
        assert_eq!(state.aliases.alias(TOP, 4), "Scuttle Crab");
    }

    #[test]
    fn self_is_excluded_from_summary() {
        let session = ranked();
        let mut state = LobbyState::default();
        state.identities.insert(
            YOU.to_string(),
            matched(YOU, "Example Player", MatchKind::Own),
        );
        // you lost your last four, a red flag on anyone else.
        state.stats.insert(YOU.to_string(), top_laner("LLLLWWWWWW"));
        let view = build(&session, &mut state);
        let me = player(&view, YOU);
        assert!(me.stats.is_some());
        assert!(me.flags.is_empty());
        assert_eq!(
            view.summary,
            Summary {
                red_flags: 0,
                red_players: 0,
                warnings: 0,
                label: "no red flags".to_string(),
            }
        );

        // the same games on the top laner do count.
        state.identities.insert(
            TOP.to_string(),
            matched(REAL[0], "Top Laner", MatchKind::Key),
        );
        state
            .stats
            .insert(REAL[0].to_string(), top_laner("LLLLWWWWWW"));
        let view = build(&session, &mut state);
        let labels: Vec<&str> = player(&view, TOP)
            .flags
            .iter()
            .map(|flag| flag.label.as_str())
            .collect();
        assert_eq!(labels, ["lost last 4"]);
        assert_eq!(view.summary.label, "1 red flag on 1 teammate");
    }

    #[test]
    fn unmatched_lists_unused_names() {
        let session = ranked();
        let mut state = LobbyState {
            candidates: vec![
                candidate(REAL[0], "Name One"),
                candidate(REAL[1], "Name Two"),
                candidate(REAL[2], "Name Three"),
            ],
            ..LobbyState::default()
        };
        state.identities.insert(
            JUNGLE.to_string(),
            matched(REAL[1], "Name Two", MatchKind::Key),
        );
        let view = build(&session, &mut state);
        assert_eq!(
            view.unmatched,
            [
                NameView {
                    game_name: "Name One".to_string(),
                    tag_line: "NA1".to_string(),
                },
                NameView {
                    game_name: "Name Three".to_string(),
                    tag_line: "NA1".to_string(),
                },
            ]
        );
    }

    #[test]
    fn stats_fill_in_as_they_arrive() {
        let session = ranked();
        let mut state = LobbyState::default();
        let guessed = Identity {
            confidence: Confidence::Medium,
            ..matched(REAL[0], "Top Laner", MatchKind::Guess)
        };
        state.identities.insert(TOP.to_string(), guessed);
        state.pending.insert(REAL[0].to_string());

        // while the stats load, the name and the link already show.
        let view = build(&session, &mut state);
        let top = player(&view, TOP);
        assert!(top.stats_pending);
        assert_eq!(top.stats, None);
        assert!(top.flags.is_empty());
        assert_eq!(
            top.identity,
            Some(IdentityView {
                game_name: "Top Laner".to_string(),
                tag_line: "NA1".to_string(),
                kind: MatchKind::Guess,
                confidence: Confidence::Medium,
            })
        );
        assert_eq!(
            top.scout_url.as_deref(),
            Some("https://www.op.gg/summoners/na/Top%20Laner-NA1")
        );

        // twelve games. the newest is a jungle game on lee sin, and the two
        // oldest have 20 deaths each, which the kda over the last ten skips.
        let mut stats = top_laner("LLLWWWWWWWLW");
        let games = &mut stats.history.as_mut().expect("there are games").games;
        games[0].champion_id = 64;
        games[0].position = Some(Position::Jungle);
        games[10].deaths = 20;
        games[11].deaths = 20;
        state.pending.remove(REAL[0]);
        state.stats.insert(REAL[0].to_string(), stats);

        let view = build(&session, &mut state);
        let top = player(&view, TOP);
        assert!(!top.stats_pending);
        let stats = top.stats.as_ref().expect("the stats arrived");
        assert_eq!(stats.level, Some(212));
        assert_eq!(
            stats.rank,
            Some(RankView {
                tier: "GOLD".to_string(),
                division: "II".to_string(),
                lp: 54,
                wins: 45,
                losses: 40,
            })
        );
        assert_eq!(stats.recent.len(), 10);
        assert_eq!(
            stats.recent[0],
            GameView {
                champion_id: 64,
                win: false,
                kills: 6,
                deaths: 4,
                assists: 6,
                started_at_ms: NOW - 48 * HOUR_MS,
                duration_s: 1800,
            }
        );
        // (6 + 6) / 4 in each of the newest ten.
        assert!(close(stats.kda, 3.0), "{:?}", stats.kda);
        // 8 wins in all 12.
        assert!(close(stats.win_rate, 8.0 / 12.0), "{:?}", stats.win_rate);
        // aatrox is the slot's champion.
        assert_eq!(stats.champion_games, Some(11));
        assert_eq!(stats.champion_points, Some(52_000));
        let roles: Vec<Position> = stats.roles.iter().map(|role| role.position).collect();
        assert_eq!(roles, [Position::Top, Position::Jungle]);
        assert!(close(Some(stats.roles[0].share), 11.0 / 12.0));
        assert!(close(Some(stats.roles[1].share), 1.0 / 12.0));

        let labels: Vec<&str> = top.flags.iter().map(|flag| flag.label.as_str()).collect();
        assert_eq!(labels, ["lost last 3"]);
        assert_eq!(view.summary.label, "1 red flag on 1 teammate");
    }

    #[test]
    fn champion_figures_wait_for_a_champion() {
        let mut session = ranked();
        for teammate in &mut session.my_team {
            teammate.champion_id = 0;
            teammate.champion_pick_intent = 0;
        }
        let mut state = LobbyState::default();
        state.identities.insert(
            TOP.to_string(),
            matched(REAL[0], "Top Laner", MatchKind::Key),
        );
        state.stats.insert(REAL[0].to_string(), top_laner("WLW"));
        let view = build(&session, &mut state);
        let stats = player(&view, TOP).stats.clone().expect("stats");
        assert_eq!(stats.champion_games, None);
        assert_eq!(stats.champion_points, None);

        // with no mastery at all there are no points to show, even on a pick.
        let mut stats = top_laner("WLW");
        stats.mastery = None;
        state.stats.insert(REAL[0].to_string(), stats);
        let view = build(&ranked(), &mut state);
        let stats = player(&view, TOP).stats.clone().expect("stats");
        assert_eq!(stats.champion_games, Some(3));
        assert_eq!(stats.champion_points, None);
    }

    #[test]
    fn a_nameless_match_takes_the_summoner_name() {
        let session = ranked();
        let mut state = LobbyState::default();
        // neither the session nor the chat named you, say.
        state
            .identities
            .insert(YOU.to_string(), matched(YOU, "", MatchKind::Own));
        let view = build(&session, &mut state);
        assert_eq!(name(&view, YOU).as_deref(), Some(""));
        assert_eq!(player(&view, YOU).scout_url, None);

        state.stats.insert(YOU.to_string(), top_laner("W"));
        let view = build(&session, &mut state);
        let me = player(&view, YOU);
        assert_eq!(
            me.identity
                .as_ref()
                .map(|identity| (identity.game_name.as_str(), identity.tag_line.as_str())),
            Some(("Sample Teammate", "NA1"))
        );
        assert_eq!(
            me.scout_url.as_deref(),
            Some("https://www.op.gg/summoners/na/Sample%20Teammate-NA1")
        );
    }

    #[test]
    fn names_for_the_scout_site() {
        let session = ranked();
        let mut state = LobbyState {
            candidates: vec![
                candidate(REAL[0], "Top Laner"),
                candidate(REAL[1], "Name Two"),
                candidate(REAL[2], ""),
            ],
            ..LobbyState::default()
        };
        state.identities.insert(
            YOU.to_string(),
            matched(YOU, "Example Player", MatchKind::Own),
        );
        state.identities.insert(
            SUPPORT.to_string(),
            matched(REAL[0], "Top Laner", MatchKind::Key),
        );
        // a second slot guessed to the same riot id is listed once.
        state.identities.insert(
            BOTTOM.to_string(),
            matched(REAL[0], "top laner", MatchKind::Guess),
        );
        let view = build(&session, &mut state);
        let names = view.names();
        let names: Vec<(&str, &str)> = names
            .iter()
            .map(|(name, tag)| (name.as_str(), tag.as_str()))
            .collect();
        // the slots in order, then the names no slot has; never a blank one.
        assert_eq!(
            names,
            [
                ("Example Player", "NA1"),
                ("top laner", "NA1"),
                ("Name Two", "NA1"),
            ]
        );
    }

    // an object's keys, sorted.
    fn keys(value: &Value) -> Vec<&str> {
        let mut keys: Vec<&str> = value
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        keys
    }

    #[test]
    fn serializes_like_the_frontend_expects() {
        let teammate = PlayerView {
            key: SUPPORT.to_string(),
            cell_id: 4,
            is_self: false,
            alias: Some("Scuttle Crab".to_string()),
            position: Some(Position::Utility),
            is_autofilled: true,
            champion_id: 412,
            spells: (4, 3),
            identity: Some(IdentityView {
                game_name: "Sample Teammate".to_string(),
                tag_line: "NA1".to_string(),
                kind: MatchKind::Guess,
                confidence: Confidence::Medium,
            }),
            stats: Some(StatsView {
                level: Some(41),
                rank: Some(RankView {
                    tier: "PLATINUM".to_string(),
                    division: "IV".to_string(),
                    lp: 22,
                    wins: 15,
                    losses: 9,
                }),
                recent: vec![GameView {
                    champion_id: 412,
                    win: false,
                    kills: 1,
                    deaths: 6,
                    assists: 9,
                    started_at_ms: NOW - HOUR_MS,
                    duration_s: 1800,
                }],
                kda: Some(1.5),
                win_rate: Some(0.6),
                champion_games: Some(7),
                champion_points: Some(96_700),
                roles: vec![RoleShare {
                    position: Position::Utility,
                    share: 0.95,
                }],
            }),
            stats_pending: false,
            flags: vec![Flag {
                kind: FlagKind::LossStreak,
                severity: Severity::Red,
                label: "lost last 4".to_string(),
            }],
            scout_url: Some("https://www.op.gg/summoners/na/Sample%20Teammate-NA1".to_string()),
        };
        let me = PlayerView {
            key: YOU.to_string(),
            cell_id: 2,
            is_self: true,
            alias: None,
            position: None,
            is_autofilled: false,
            champion_id: 0,
            spells: (4, 14),
            identity: None,
            stats: None,
            stats_pending: true,
            flags: Vec::new(),
            scout_url: None,
        };
        let view = LobbyView {
            game_id: 5_123_456_789,
            queue_id: 420,
            phase: "FINALIZATION".to_string(),
            time_left_ms: 9_000,
            anonymous: true,
            players: vec![teammate, me],
            unmatched: vec![NameView {
                game_name: "Name One".to_string(),
                tag_line: "NA1".to_string(),
            }],
            summary: Summary {
                red_flags: 1,
                red_players: 1,
                warnings: 0,
                label: "1 red flag on 1 teammate".to_string(),
            },
            dodge: DodgeView {
                armed: true,
                lead_ms: 1500,
            },
        };
        let json = serde_json::to_value(&view).unwrap();

        assert_eq!(
            keys(&json),
            [
                "anonymous",
                "dodge",
                "gameId",
                "phase",
                "players",
                "queueId",
                "summary",
                "timeLeftMs",
                "unmatched",
            ]
        );
        assert_eq!(json["gameId"], json!(5_123_456_789_i64));
        assert_eq!(json["timeLeftMs"], json!(9_000));

        let teammate = &json["players"][0];
        assert_eq!(
            keys(teammate),
            [
                "alias",
                "cellId",
                "championId",
                "flags",
                "identity",
                "isAutofilled",
                "isSelf",
                "key",
                "position",
                "scoutUrl",
                "spells",
                "stats",
                "statsPending",
            ]
        );
        assert_eq!(teammate["position"], "utility");
        assert_eq!(teammate["spells"], json!([4, 3]));
        assert_eq!(teammate["isAutofilled"], true);

        let identity = &teammate["identity"];
        assert_eq!(
            keys(identity),
            ["confidence", "gameName", "kind", "tagLine"]
        );
        assert_eq!(identity["kind"], "guess");
        assert_eq!(identity["confidence"], "medium");

        let stats = &teammate["stats"];
        assert_eq!(
            keys(stats),
            [
                "championGames",
                "championPoints",
                "kda",
                "level",
                "rank",
                "recent",
                "roles",
                "winRate",
            ]
        );
        assert_eq!(
            keys(&stats["rank"]),
            ["division", "losses", "lp", "tier", "wins"]
        );
        assert_eq!(
            keys(&stats["recent"][0]),
            [
                "assists",
                "championId",
                "deaths",
                "durationS",
                "kills",
                "startedAtMs",
                "win",
            ]
        );
        assert_eq!(keys(&stats["roles"][0]), ["position", "share"]);
        assert_eq!(stats["roles"][0]["position"], "utility");
        assert_eq!(stats["winRate"], json!(0.6));

        let flag = &teammate["flags"][0];
        assert_eq!(keys(flag), ["kind", "label", "severity"]);
        assert_eq!(flag["kind"], "lossStreak");
        assert_eq!(flag["severity"], "red");

        // missing pieces arrive as null, never left out.
        let me = &json["players"][1];
        assert_eq!(keys(me), keys(teammate));
        for field in ["alias", "position", "identity", "stats", "scoutUrl"] {
            assert!(me[field].is_null(), "{field}");
        }
        assert_eq!(me["isSelf"], true);
        assert_eq!(me["statsPending"], true);
        assert_eq!(me["flags"], json!([]));

        assert_eq!(keys(&json["unmatched"][0]), ["gameName", "tagLine"]);
        assert_eq!(
            keys(&json["summary"]),
            ["label", "redFlags", "redPlayers", "warnings"]
        );
        assert_eq!(keys(&json["dodge"]), ["armed", "leadMs"]);
        assert_eq!(json["dodge"]["leadMs"], json!(1500));

        // every enum, spelled the way src/types/index.ts spells it.
        assert_eq!(
            serde_json::to_value(Position::ALL).unwrap(),
            json!(["top", "jungle", "middle", "bottom", "utility"])
        );
        assert_eq!(
            serde_json::to_value([
                MatchKind::Own,
                MatchKind::Visible,
                MatchKind::Key,
                MatchKind::Elimination,
                MatchKind::Guess,
            ])
            .unwrap(),
            json!(["own", "visible", "key", "elimination", "guess"])
        );
        assert_eq!(
            serde_json::to_value([Confidence::High, Confidence::Medium, Confidence::Low]).unwrap(),
            json!(["high", "medium", "low"])
        );
        assert_eq!(
            serde_json::to_value([Severity::Red, Severity::Amber, Severity::Info]).unwrap(),
            json!(["red", "amber", "info"])
        );
        assert_eq!(
            serde_json::to_value([
                FlagKind::Autofilled,
                FlagKind::LossStreak,
                FlagKind::FirstTime,
                FlagKind::NewToChampion,
                FlagKind::LowWinRate,
                FlagKind::BadKda,
                FlagKind::OnTilt,
                FlagKind::FreshAccount,
                FlagKind::NoData,
            ])
            .unwrap(),
            json!([
                "autofilled",
                "lossStreak",
                "firstTime",
                "newToChampion",
                "lowWinRate",
                "badKda",
                "onTilt",
                "freshAccount",
                "noData",
            ])
        );
    }
}
