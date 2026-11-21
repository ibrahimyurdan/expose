// red flags: the signs in a teammate's recent games that the lobby may go
// badly, and the one line that sums them up for the team. red is a strong
// sign, amber a mild one, and info only says something could not be checked.
// every threshold lives here, next to the label it produces.

use serde::Serialize;

use super::stats::{PlayerStats, Position};

// enough games with a known role to say which two roles a player mains.
const ROLE_GAMES: usize = 8;
// losses in a row that make a streak.
const LOSS_STREAK: usize = 3;
// mastery points under which a champion is still new to the player.
const NEW_TO_CHAMPION_POINTS: i64 = 10_000;
// with no mastery to go on, recent games without the champion that make it new.
const NEW_TO_CHAMPION_GAMES: usize = 10;
// a win rate under 40% across at least 10 of the newest 20 games. the lobby
// view shows the win rate and the kda over these same games.
pub(super) const WIN_RATE_GAMES: usize = 20;
const WIN_RATE_MIN_GAMES: usize = 10;
const LOW_WIN_RATE: f64 = 0.40;
// a kda under 1.5 across at least 5 of the newest 10 games.
pub(super) const KDA_GAMES: usize = 10;
const KDA_MIN_GAMES: usize = 5;
const BAD_KDA: f64 = 1.5;
// 5 games or more in the last 6 hours, more of them lost than won.
const TILT_WINDOW_MS: i64 = 6 * 60 * 60 * 1000;
const TILT_GAMES: usize = 5;
// under level 60 while ranked gold or above.
const FRESH_LEVEL: i64 = 60;
const FRESH_FROM_TIER: &str = "GOLD";
// the ranked tiers, lowest first.
const TIERS: [&str; 10] = [
    "IRON",
    "BRONZE",
    "SILVER",
    "GOLD",
    "PLATINUM",
    "EMERALD",
    "DIAMOND",
    "MASTER",
    "GRANDMASTER",
    "CHALLENGER",
];

// strongest first, so sorting by severity puts red flags first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Red,
    Amber,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FlagKind {
    Autofilled,
    LossStreak,
    FirstTime,
    NewToChampion,
    LowWinRate,
    BadKda,
    OnTilt,
    FreshAccount,
    NoData,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Flag {
    pub kind: FlagKind,
    pub severity: Severity,
    // what the chip says ("lost last 4").
    pub label: String,
}

// one teammate in their slot, with what is known about them.
#[derive(Debug, Clone, Copy)]
pub struct FlagInput<'a> {
    pub position: Option<Position>,
    pub is_autofilled: bool,
    // 0 while the slot has neither picked nor declared a champion.
    pub champion_id: i64,
    pub champion_name: Option<&'a str>,
    pub stats: &'a PlayerStats,
    pub now_ms: i64,
}

// the flags one teammate raises: red first, then amber, then info.
pub fn flags_for(input: &FlagInput) -> Vec<Flag> {
    let stats = input.stats;
    let mut flags: Vec<Flag> = [
        autofilled(input),
        loss_streak(stats),
        champion_experience(input),
        low_win_rate(stats),
        bad_kda(stats),
        on_tilt(stats, input.now_ms),
        fresh_account(stats),
        no_data(stats),
    ]
    .into_iter()
    .flatten()
    .collect();
    // the sort is stable, so flags of one severity keep the order above.
    flags.sort_by_key(|flag| flag.severity);
    flags
}

// the team in one line, from every teammate's flags (never your own).
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub red_flags: usize,
    // teammates with at least one red flag.
    pub red_players: usize,
    // amber flags.
    pub warnings: usize,
    pub label: String,
}

// red flags and the teammates raising them; with no red flag, the warnings.
// info flags count for nothing.
pub fn summarize(per_player: &[Vec<Flag>]) -> Summary {
    let red_flags: usize = per_player
        .iter()
        .map(|flags| count(flags, Severity::Red))
        .sum();
    let red_players = per_player
        .iter()
        .filter(|flags| count(flags, Severity::Red) > 0)
        .count();
    let warnings: usize = per_player
        .iter()
        .map(|flags| count(flags, Severity::Amber))
        .sum();
    let label = if red_flags > 0 {
        format!(
            "{red_flags} red flag{} on {red_players} teammate{}",
            plural(red_flags),
            plural(red_players)
        )
    } else if warnings > 0 {
        format!("{warnings} warning{}", plural(warnings))
    } else {
        "no red flags".to_string()
    };
    Summary {
        red_flags,
        red_players,
        warnings,
        label,
    }
}

// riot autofilled the player, or their history says the slot's role is not
// one of the two they play most. the label names the role they do main,
// unless it is this one.
fn autofilled(input: &FlagInput) -> Option<Flag> {
    let history = input.stats.history.as_ref();
    let off_role = match (input.position, history) {
        (Some(position), Some(history)) => {
            history.positioned_games() >= ROLE_GAMES && !history.top_roles(2).contains(&position)
        }
        _ => false,
    };
    if !input.is_autofilled && !off_role {
        return None;
    }
    let main = history.and_then(|history| history.top_roles(1).first().copied());
    let label = match main {
        Some(main) if Some(main) != input.position => {
            format!("autofilled, mains {}", main.label())
        }
        _ => "autofilled".to_string(),
    };
    Some(flag(FlagKind::Autofilled, Severity::Red, label))
}

fn loss_streak(stats: &PlayerStats) -> Option<Flag> {
    let streak = stats.history.as_ref()?.loss_streak();
    (streak >= LOSS_STREAK).then(|| {
        flag(
            FlagKind::LossStreak,
            Severity::Red,
            format!("lost last {streak}"),
        )
    })
}

// a first time on the slot's champion, or barely any time on it. when mastery
// and the recent games disagree, the games win: a game on the champion in the
// history means it is not a first time, whatever the points say.
fn champion_experience(input: &FlagInput) -> Option<Flag> {
    if input.champion_id == 0 {
        return None;
    }
    let champion = input.champion_name.unwrap_or("this champion");
    let history = input.stats.history.as_ref();
    // none when there is no history to look in.
    let recent = history.map(|history| history.games_on(input.champion_id));
    let first_time = || {
        let label = format!("first time on {champion}");
        flag(FlagKind::FirstTime, Severity::Red, label)
    };
    let new_to = || {
        flag(
            FlagKind::NewToChampion,
            Severity::Amber,
            format!("new to {champion}"),
        )
    };
    match &input.stats.mastery {
        Some(mastery) => {
            let points = mastery.get(&input.champion_id).copied().unwrap_or(0);
            if points <= 0 && recent.unwrap_or(0) == 0 {
                Some(first_time())
            } else if points < NEW_TO_CHAMPION_POINTS && recent == Some(0) {
                Some(new_to())
            } else {
                None
            }
        }
        // without mastery only a long enough history can tell.
        None => {
            let games = history?.games.len();
            (games >= NEW_TO_CHAMPION_GAMES && recent == Some(0)).then(new_to)
        }
    }
}

fn low_win_rate(stats: &PlayerStats) -> Option<Flag> {
    let (rate, games) = stats.history.as_ref()?.win_rate(WIN_RATE_GAMES)?;
    (games >= WIN_RATE_MIN_GAMES && rate < LOW_WIN_RATE).then(|| {
        let percent = (rate * 100.0).round() as i64;
        let label = format!("{percent}% win rate, last {games}");
        flag(FlagKind::LowWinRate, Severity::Amber, label)
    })
}

fn bad_kda(stats: &PlayerStats) -> Option<Flag> {
    let (kda, games) = stats.history.as_ref()?.kda(KDA_GAMES)?;
    (games >= KDA_MIN_GAMES && kda < BAD_KDA).then(|| {
        let label = format!("{kda:.1} kda, last {games}");
        flag(FlagKind::BadKda, Severity::Amber, label)
    })
}

// a long session going badly: more losses than wins.
fn on_tilt(stats: &PlayerStats, now_ms: i64) -> Option<Flag> {
    let since_ms = now_ms.saturating_sub(TILT_WINDOW_MS);
    let (games, losses) = stats.history.as_ref()?.games_since(since_ms);
    (games >= TILT_GAMES && losses * 2 > games).then(|| {
        let label = format!("{games} games in 6 hours");
        flag(FlagKind::OnTilt, Severity::Amber, label)
    })
}

// a low level account ranked this high was possibly bought or boosted.
fn fresh_account(stats: &PlayerStats) -> Option<Flag> {
    let level = stats.level?;
    let rank = stats.rank.as_ref()?;
    let ranked_high = tier_index(&rank.tier)? >= tier_index(FRESH_FROM_TIER)?;
    (level < FRESH_LEVEL && ranked_high).then(|| {
        let label = format!("level {level} in {}", rank.tier.to_ascii_lowercase());
        flag(FlagKind::FreshAccount, Severity::Amber, label)
    })
}

fn no_data(stats: &PlayerStats) -> Option<Flag> {
    stats
        .history
        .is_none()
        .then(|| flag(FlagKind::NoData, Severity::Info, "no match history"))
}

fn tier_index(tier: &str) -> Option<usize> {
    TIERS
        .iter()
        .position(|known| known.eq_ignore_ascii_case(tier))
}

fn flag(kind: FlagKind, severity: Severity, label: impl Into<String>) -> Flag {
    Flag {
        kind,
        severity,
        label: label.into(),
    }
}

fn count(flags: &[Flag], severity: Severity) -> usize {
    flags
        .iter()
        .filter(|flag| flag.severity == severity)
        .count()
}

fn plural(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::lobby::stats::{GameSummary, PlayerHistory, Rank};

    const NOW: i64 = 1_795_180_000_000;
    const HOUR_MS: i64 = 3_600_000;
    const AHRI: i64 = 103;
    const JINX: i64 = 222;

    // a mid game on ahri two days ago: a 4.0 kda, flash and ignite.
    fn game() -> GameSummary {
        GameSummary {
            champion_id: AHRI,
            win: true,
            kills: 5,
            deaths: 3,
            assists: 7,
            started_at_ms: NOW - 48 * HOUR_MS,
            duration_s: 1800,
            position: Some(Position::Middle),
            spells: (4, 14),
        }
    }

    // games from a record written newest first, "W" for a win.
    fn record(results: &str) -> Vec<GameSummary> {
        results
            .chars()
            .map(|result| GameSummary {
                win: result == 'W',
                ..game()
            })
            .collect()
    }

    // a game this long ago.
    fn ago(ms: i64, win: bool) -> GameSummary {
        GameSummary {
            win,
            started_at_ms: NOW - ms,
            ..game()
        }
    }

    fn rank(tier: &str) -> Rank {
        Rank {
            tier: tier.to_string(),
            division: "II".to_string(),
            lp: 54,
            wins: 45,
            losses: 40,
        }
    }

    // a solid player: twenty mid games, half of them won, a 4.0 kda, plenty
    // of mastery, well past level 60. nothing should fire for them, so each
    // test changes one thing.
    fn solid() -> PlayerStats {
        PlayerStats {
            level: Some(212),
            game_name: Some("Sample Teammate".to_string()),
            tag_line: Some("NA1".to_string()),
            rank: Some(rank("GOLD")),
            history: Some(PlayerHistory {
                games: record("WLWLWLWLWLWLWLWLWLWL"),
            }),
            mastery: Some(HashMap::from([(AHRI, 52_000)])),
        }
    }

    fn with_games(games: Vec<GameSummary>) -> PlayerStats {
        PlayerStats {
            history: Some(PlayerHistory { games }),
            ..solid()
        }
    }

    // the mid slot on ahri.
    fn input(stats: &PlayerStats) -> FlagInput<'_> {
        FlagInput {
            position: Some(Position::Middle),
            is_autofilled: false,
            champion_id: AHRI,
            champion_name: Some("Ahri"),
            stats,
            now_ms: NOW,
        }
    }

    // the same slot in another role.
    fn playing(stats: &PlayerStats, position: Position) -> FlagInput<'_> {
        FlagInput {
            position: Some(position),
            ..input(stats)
        }
    }

    // the same slot on jinx.
    fn on_jinx(stats: &PlayerStats) -> FlagInput<'_> {
        FlagInput {
            champion_id: JINX,
            champion_name: Some("Jinx"),
            ..input(stats)
        }
    }

    // the label of the flag of this kind, if it fired.
    fn label(flags: &[Flag], kind: FlagKind) -> Option<String> {
        flags
            .iter()
            .find(|flag| flag.kind == kind)
            .map(|flag| flag.label.clone())
    }

    fn flag(kind: FlagKind, severity: Severity) -> Flag {
        Flag {
            kind,
            severity,
            label: String::new(),
        }
    }

    #[test]
    fn a_solid_player_raises_nothing() {
        assert_eq!(flags_for(&input(&solid())), []);
    }

    #[test]
    fn autofilled_by_the_client_or_by_the_history() {
        let stats = solid();
        // the client says so, in a role the player does not main.
        let flags = flags_for(&FlagInput {
            is_autofilled: true,
            ..playing(&stats, Position::Utility)
        });
        assert_eq!(
            flags,
            [Flag {
                kind: FlagKind::Autofilled,
                severity: Severity::Red,
                label: "autofilled, mains mid".to_string(),
            }]
        );
        // the client says so in the role they main anyway.
        let flags = flags_for(&FlagInput {
            is_autofilled: true,
            ..input(&stats)
        });
        assert_eq!(
            label(&flags, FlagKind::Autofilled).as_deref(),
            Some("autofilled")
        );
        // the client says so and the history is unknown.
        let unknown = PlayerStats {
            history: None,
            ..solid()
        };
        let flags = flags_for(&FlagInput {
            is_autofilled: true,
            ..input(&unknown)
        });
        assert_eq!(
            label(&flags, FlagKind::Autofilled).as_deref(),
            Some("autofilled")
        );

        // the history says so: eight games with a known role, none of them in
        // the slot's role. seven are too few to tell.
        let mut games = record("WLWLWLW");
        games.extend(record("WLW").into_iter().map(|game| GameSummary {
            position: None,
            ..game
        }));
        let seven = with_games(games.clone());
        let flags = flags_for(&playing(&seven, Position::Utility));
        assert_eq!(label(&flags, FlagKind::Autofilled), None);
        games.push(game());
        let eight = with_games(games);
        let flags = flags_for(&playing(&eight, Position::Utility));
        assert_eq!(
            label(&flags, FlagKind::Autofilled).as_deref(),
            Some("autofilled, mains mid")
        );

        // the slot's role is the player's second most played: no flag.
        let mut games = record("WLWLWLWLWL");
        games.extend(record("WLWLW").into_iter().map(|game| GameSummary {
            position: Some(Position::Jungle),
            ..game
        }));
        let jungle_second = with_games(games);
        let flags = flags_for(&playing(&jungle_second, Position::Jungle));
        assert_eq!(label(&flags, FlagKind::Autofilled), None);
    }

    #[test]
    fn loss_streak_from_three() {
        let two = with_games(record("LLWWLWLWLWLWLWLWLWLW"));
        assert_eq!(label(&flags_for(&input(&two)), FlagKind::LossStreak), None);

        let three = with_games(record("LLLWWLWLWLWLWLWLWLWL"));
        assert_eq!(
            flags_for(&input(&three)),
            [Flag {
                kind: FlagKind::LossStreak,
                severity: Severity::Red,
                label: "lost last 3".to_string(),
            }]
        );
    }

    #[test]
    fn first_time_and_new_to_champion() {
        let with_mastery = |points: i64| PlayerStats {
            mastery: Some(HashMap::from([(AHRI, 52_000), (JINX, points)])),
            ..solid()
        };

        // no points on the champion at all.
        let flags = flags_for(&on_jinx(&solid()));
        assert_eq!(
            flags,
            [Flag {
                kind: FlagKind::FirstTime,
                severity: Severity::Red,
                label: "first time on Jinx".to_string(),
            }]
        );
        let flags = flags_for(&FlagInput {
            champion_name: None,
            ..on_jinx(&solid())
        });
        assert_eq!(
            label(&flags, FlagKind::FirstTime).as_deref(),
            Some("first time on this champion")
        );
        // no points, yet a recent game on it: the games win over the points.
        let mut played = with_mastery(0);
        played.history.as_mut().unwrap().games[3].champion_id = JINX;
        assert_eq!(flags_for(&on_jinx(&played)), []);

        // a few points and no recent games, up to 10,000 points.
        let flags = flags_for(&on_jinx(&with_mastery(9_999)));
        assert_eq!(
            flags,
            [Flag {
                kind: FlagKind::NewToChampion,
                severity: Severity::Amber,
                label: "new to Jinx".to_string(),
            }]
        );
        assert_eq!(flags_for(&on_jinx(&with_mastery(10_000))), []);
        // under 10,000 points with a recent game on it is fine.
        let mut played = with_mastery(9_999);
        played.history.as_mut().unwrap().games[3].champion_id = JINX;
        assert_eq!(flags_for(&on_jinx(&played)), []);
        // and without a history there are no recent games to miss.
        let no_history = PlayerStats {
            history: None,
            ..with_mastery(9_999)
        };
        assert_eq!(
            label(&flags_for(&on_jinx(&no_history)), FlagKind::NewToChampion),
            None
        );

        // without mastery, ten recent games and none on the champion.
        let unknown = |games: &str| PlayerStats {
            mastery: None,
            ..with_games(record(games))
        };
        let ten = unknown("WLWLWLWLWL");
        assert_eq!(
            label(&flags_for(&on_jinx(&ten)), FlagKind::NewToChampion).as_deref(),
            Some("new to Jinx")
        );
        assert_eq!(flags_for(&on_jinx(&unknown("WLWLWLWLW"))), []);

        // and nothing at all without a champion.
        let player = solid();
        let no_champion = FlagInput {
            champion_id: 0,
            champion_name: None,
            ..input(&player)
        };
        assert_eq!(flags_for(&no_champion), []);
        assert_eq!(
            flags_for(&FlagInput {
                stats: &ten,
                ..no_champion
            }),
            []
        );
    }

    #[test]
    fn low_win_rate_under_forty_percent() {
        // four wins in ten is not under 40%.
        let forty = with_games(record("WWWLWLLLLL"));
        assert_eq!(
            label(&flags_for(&input(&forty)), FlagKind::LowWinRate),
            None
        );
        let thirty = with_games(record("WWWLLLLLLL"));
        assert_eq!(
            flags_for(&input(&thirty)),
            [Flag {
                kind: FlagKind::LowWinRate,
                severity: Severity::Amber,
                label: "30% win rate, last 10".to_string(),
            }]
        );
        // nine games are too few, however they went.
        let nine = with_games(record("WLLLLLLLL"));
        assert_eq!(label(&flags_for(&input(&nine)), FlagKind::LowWinRate), None);
        // the percentage is rounded: 7 of 19 is 36.8%.
        let nineteen = with_games(record(&("W".repeat(7) + &"L".repeat(12))));
        assert_eq!(
            label(&flags_for(&input(&nineteen)), FlagKind::LowWinRate).as_deref(),
            Some("37% win rate, last 19")
        );
        // only the newest twenty count: 7 wins in them, then two more.
        let older_wins = with_games(record(&("W".repeat(7) + &"L".repeat(13) + "WW")));
        assert_eq!(
            label(&flags_for(&input(&older_wins)), FlagKind::LowWinRate).as_deref(),
            Some("35% win rate, last 20")
        );
    }

    #[test]
    fn bad_kda_under_one_and_a_half() {
        let scored = |kills, deaths, assists, count| -> Vec<GameSummary> {
            std::iter::repeat_n(
                GameSummary {
                    kills,
                    deaths,
                    assists,
                    ..game()
                },
                count,
            )
            .collect()
        };
        // (1 + 5) / 4 is not under 1.5.
        let even = with_games(scored(1, 4, 5, 5));
        assert_eq!(label(&flags_for(&input(&even)), FlagKind::BadKda), None);
        let under = with_games(scored(2, 5, 5, 5));
        assert_eq!(
            flags_for(&input(&under)),
            [Flag {
                kind: FlagKind::BadKda,
                severity: Severity::Amber,
                label: "1.4 kda, last 5".to_string(),
            }]
        );
        // four games are too few.
        let four = with_games(scored(0, 10, 0, 4));
        assert_eq!(label(&flags_for(&input(&four)), FlagKind::BadKda), None);
        // only the newest ten count: the two great games before them do not.
        let mut games = scored(1, 5, 2, 10);
        games.extend(scored(20, 0, 20, 2));
        let slump = with_games(games);
        assert_eq!(
            label(&flags_for(&input(&slump)), FlagKind::BadKda).as_deref(),
            Some("0.6 kda, last 10")
        );
    }

    #[test]
    fn on_tilt_after_five_games_in_six_hours() {
        let tilt = |recent: Vec<GameSummary>| {
            let mut games = recent;
            games.extend(record("WWWWWWWWWW"));
            label(&flags_for(&input(&with_games(games))), FlagKind::OnTilt)
        };
        let hourly = |results: &str| -> Vec<GameSummary> {
            results
                .chars()
                .zip(1..)
                .map(|(result, hours)| ago(hours * HOUR_MS, result == 'W'))
                .collect()
        };
        assert_eq!(tilt(hourly("LWLWL")).as_deref(), Some("5 games in 6 hours"));
        // four games, even all lost, are not a session of tilt.
        assert_eq!(tilt(hourly("LLLL")), None);
        // more wins than losses, or as many.
        assert_eq!(tilt(hourly("WLWLW")), None);
        assert_eq!(tilt(hourly("LWLWLW")), None);

        // the window is six hours, its first moment included.
        let mut games = hourly("LWLW");
        games.push(ago(6 * HOUR_MS, false));
        assert_eq!(tilt(games).as_deref(), Some("5 games in 6 hours"));
        let mut games = hourly("LWLW");
        games.push(ago(6 * HOUR_MS + 1, false));
        assert_eq!(tilt(games), None);
    }

    #[test]
    fn fresh_account_below_level_sixty_from_gold_up() {
        let account = |level, tier: &str| PlayerStats {
            level: Some(level),
            rank: Some(rank(tier)),
            ..solid()
        };
        assert_eq!(
            flags_for(&input(&account(59, "GOLD"))),
            [Flag {
                kind: FlagKind::FreshAccount,
                severity: Severity::Amber,
                label: "level 59 in gold".to_string(),
            }]
        );
        assert_eq!(flags_for(&input(&account(60, "GOLD"))), []);
        assert_eq!(flags_for(&input(&account(59, "SILVER"))), []);
        assert_eq!(
            label(
                &flags_for(&input(&account(12, "CHALLENGER"))),
                FlagKind::FreshAccount
            )
            .as_deref(),
            Some("level 12 in challenger")
        );
        // an unknown level or rank says nothing.
        let no_level = PlayerStats {
            level: None,
            ..account(12, "DIAMOND")
        };
        let unranked = PlayerStats {
            rank: None,
            ..account(12, "DIAMOND")
        };
        assert_eq!(flags_for(&input(&no_level)), []);
        assert_eq!(flags_for(&input(&unranked)), []);
    }

    #[test]
    fn no_data_without_match_history() {
        let stats = PlayerStats {
            history: None,
            ..solid()
        };
        assert_eq!(
            flags_for(&input(&stats)),
            [Flag {
                kind: FlagKind::NoData,
                severity: Severity::Info,
                label: "no match history".to_string(),
            }]
        );
    }

    #[test]
    fn flags_are_sorted_by_severity() {
        // fresh, losing, off role and new to the champion, all at once.
        let mut games = vec![
            ago(HOUR_MS, false),
            ago(2 * HOUR_MS, false),
            ago(3 * HOUR_MS, false),
            ago(4 * HOUR_MS, true),
            ago(5 * HOUR_MS, false),
        ];
        games.extend(record("LWLLLWLLLW"));
        let stats = PlayerStats {
            level: Some(41),
            mastery: Some(HashMap::from([(AHRI, 52_000), (JINX, 2_400)])),
            ..with_games(games)
        };
        let flags = flags_for(&FlagInput {
            position: Some(Position::Utility),
            champion_id: JINX,
            champion_name: Some("Jinx"),
            ..input(&stats)
        });
        let kinds: Vec<FlagKind> = flags.iter().map(|flag| flag.kind).collect();
        assert_eq!(
            kinds,
            [
                FlagKind::Autofilled,
                FlagKind::LossStreak,
                FlagKind::NewToChampion,
                FlagKind::LowWinRate,
                FlagKind::OnTilt,
                FlagKind::FreshAccount,
            ]
        );
        // and without a history, the info flag comes last.
        let unknown = PlayerStats {
            history: None,
            mastery: Some(HashMap::new()),
            ..stats
        };
        let flags = flags_for(&FlagInput {
            is_autofilled: true,
            champion_id: JINX,
            champion_name: Some("Jinx"),
            ..input(&unknown)
        });
        let severities: Vec<Severity> = flags.iter().map(|flag| flag.severity).collect();
        assert_eq!(
            severities,
            [
                Severity::Red,
                Severity::Red,
                Severity::Amber,
                Severity::Info,
            ]
        );
    }

    #[test]
    fn summary_counts_red_flags_and_players() {
        let summary = summarize(&[
            vec![
                flag(FlagKind::Autofilled, Severity::Red),
                flag(FlagKind::LossStreak, Severity::Red),
                flag(FlagKind::LowWinRate, Severity::Amber),
            ],
            vec![flag(FlagKind::FirstTime, Severity::Red)],
            vec![flag(FlagKind::BadKda, Severity::Amber)],
            vec![],
        ]);
        assert_eq!(
            summary,
            Summary {
                red_flags: 3,
                red_players: 2,
                warnings: 2,
                label: "3 red flags on 2 teammates".to_string(),
            }
        );
        let one = summarize(&[vec![flag(FlagKind::LossStreak, Severity::Red)], vec![]]);
        assert_eq!(one.label, "1 red flag on 1 teammate");
    }

    #[test]
    fn summary_falls_back_to_warnings() {
        let summary = summarize(&[
            vec![flag(FlagKind::OnTilt, Severity::Amber)],
            vec![
                flag(FlagKind::FreshAccount, Severity::Amber),
                flag(FlagKind::NoData, Severity::Info),
            ],
            vec![],
        ]);
        assert_eq!(
            summary,
            Summary {
                red_flags: 0,
                red_players: 0,
                warnings: 2,
                label: "2 warnings".to_string(),
            }
        );
        let one = summarize(&[vec![flag(FlagKind::BadKda, Severity::Amber)]]);
        assert_eq!(one.label, "1 warning");
    }

    #[test]
    fn summary_with_no_flags() {
        let quiet = Summary {
            label: "no red flags".to_string(),
            ..Summary::default()
        };
        assert_eq!(summarize(&[]), quiet);
        // an info flag is no warning.
        assert_eq!(
            summarize(&[vec![flag(FlagKind::NoData, Severity::Info)], vec![]]),
            quiet
        );
    }

    #[test]
    fn flags_serialize_like_the_frontend_expects() {
        let kinds = [
            FlagKind::Autofilled,
            FlagKind::LossStreak,
            FlagKind::FirstTime,
            FlagKind::NewToChampion,
            FlagKind::LowWinRate,
            FlagKind::BadKda,
            FlagKind::OnTilt,
            FlagKind::FreshAccount,
            FlagKind::NoData,
        ];
        assert_eq!(
            serde_json::to_value(kinds).unwrap(),
            serde_json::json!([
                "autofilled",
                "lossStreak",
                "firstTime",
                "newToChampion",
                "lowWinRate",
                "badKda",
                "onTilt",
                "freshAccount",
                "noData"
            ])
        );
        assert_eq!(
            serde_json::to_value(Flag {
                kind: FlagKind::LossStreak,
                severity: Severity::Red,
                label: "lost last 3".to_string(),
            })
            .unwrap(),
            serde_json::json!({"kind": "lossStreak", "severity": "red", "label": "lost last 3"})
        );
        assert_eq!(
            serde_json::to_value([Severity::Red, Severity::Amber, Severity::Info]).unwrap(),
            serde_json::json!(["red", "amber", "info"])
        );
        assert_eq!(
            serde_json::to_value(summarize(&[vec![flag(FlagKind::OnTilt, Severity::Amber)]]))
                .unwrap(),
            serde_json::json!({"redFlags": 0, "redPlayers": 0, "warnings": 1, "label": "1 warning"})
        );
    }
}
