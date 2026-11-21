// identity matching: which real account sits in each slot of your team.
//
// in anonymous ranked champ select the league client hides every teammate
// outside your party behind an obfuscated puuid, while the riot client's chat
// still lists the real accounts in the room. resolve_exact runs four steps in
// order, each touching only players still unresolved: you, anyone the session
// names, the lobby's scramble key, then elimination. all four are exact, and a
// hidden player is only ever matched to a name the riot client already lists,
// so expose never reveals someone the riot client does not. guess is the fifth
// step, for whoever is left: it places the remaining names where their recent
// games fit best, and every match it makes is marked as a guess.

use std::collections::{HashMap, HashSet};

use serde::Serialize;

use super::session::{is_blank, Session, SessionPlayer};
use super::stats::{PlayerStats, Position};

// a guess places at most one team's worth of players, which keeps the search
// over every pairing small (5! = 120 at most).
const MAX_GUESSED: usize = 5;
// how far the best pairing has to lead the next best one for a guess to count
// as high or medium confidence.
const HIGH_LEAD: f64 = 0.5;
const MEDIUM_LEAD: f64 = 0.2;

// a real account in the champ select chat, as the riot client lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
}

// how a player was matched to their slot. every kind but guess is exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MatchKind {
    // the local player.
    Own,
    // shown by the session itself: a party member, or anyone in a queue that
    // hides nobody.
    Visible,
    // a hidden player decoded with the lobby's scramble key.
    Key,
    // the one hidden player left, given the one name left.
    Elimination,
    // the best fit from recent games, never certain.
    Guess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

// who sits in a slot. exact kinds always carry high confidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
    pub kind: MatchKind,
    pub confidence: Confidence,
}

// what the exact steps found.
#[derive(Debug, Default)]
pub struct Resolved {
    // keyed by player key (see SessionPlayer::key).
    pub identities: HashMap<String, Identity>,
    // the keys of hidden players the scramble key decodes to someone the chat
    // does not list. none of the chat names is theirs, so they are never
    // guessed either.
    pub absent: HashSet<String>,
}

// reads a puuid written as 8-4-4-4-12 hex digits, in any case. anything else
// (no dashes, braces, spaces, a group of the wrong length) is not a puuid.
pub fn parse_uuid(text: &str) -> Option<u128> {
    let mut groups = text.split('-');
    let mut value: u128 = 0;
    for length in [8, 4, 4, 4, 12] {
        let group = groups.next()?;
        if group.len() != length || !group.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        value = (value << (4 * length)) | u128::from_str_radix(group, 16).ok()?;
    }
    groups.next().is_none().then_some(value)
}

// writes a puuid back in the lowercase 8-4-4-4-12 form riot uses.
pub fn format_uuid(value: u128) -> String {
    let hex = format!("{value:032x}");
    format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    )
}

// recovers the lobby's scramble key. riot derives every hidden player's
// obfuscated puuid from the real one with a single 128-bit xor key per lobby,
// so each (hidden, real) pair proposes key = hidden ^ real, and a key scores
// one point for every hidden value it maps onto a real puuid. the best key is
// accepted only when it scores at least two and no other key ties it, so a key
// is used only once the data itself confirms it.
//
// two matches are never enough on their own: swapping them gives a second key
// (key ^ a ^ b, for their real puuids a and b) that maps both players too. in
// practice a key needs three hidden players in the chat.
pub fn recover_key(hidden: &[u128], real: &[u128]) -> Option<u128> {
    // distinct values only, so a repeated entry cannot vote twice. with both
    // sides distinct, the pairs proposing a key are exactly the hidden values
    // it maps onto a real puuid, so counting proposals counts the score.
    let hidden: HashSet<u128> = hidden.iter().copied().collect();
    let real: HashSet<u128> = real.iter().copied().collect();
    let mut scores: HashMap<u128, usize> = HashMap::new();
    for obfuscated in &hidden {
        for puuid in &real {
            *scores.entry(obfuscated ^ puuid).or_default() += 1;
        }
    }

    let best = scores.values().copied().max()?;
    let mut leaders = scores.into_iter().filter(|&(_, score)| score == best);
    let (key, _) = leaders.next()?;
    (best >= 2 && leaders.next().is_none()).then_some(key)
}

// the exact identities in this session, and the hidden players the scramble
// key rules out. candidates are the riot client's champ select chat
// participants.
pub fn resolve_exact(session: &Session, candidates: &[Candidate]) -> Resolved {
    let mut identities = HashMap::new();
    let mut absent = HashSet::new();

    // 1. you.
    if let Some(player) = session.local_player() {
        if let Some(puuid) = player.real_puuid() {
            let identity = from_session(player, puuid, candidates, MatchKind::Own);
            identities.insert(player.key(), identity);
        }
    }

    // 2. anyone else the session shows with a real puuid. a hidden player is
    // never revealed this way, even if a puuid slipped through.
    for player in &session.my_team {
        let key = player.key();
        if player.is_hidden() || identities.contains_key(&key) {
            continue;
        }
        if let Some(puuid) = player.real_puuid() {
            let identity = from_session(player, puuid, candidates, MatchKind::Visible);
            identities.insert(key, identity);
        }
    }

    // the chat names no identity has used yet, each once.
    let used: HashSet<String> = identities
        .values()
        .map(|identity| identity.puuid.to_ascii_lowercase())
        .collect();
    let mut seen = HashSet::new();
    let mut free: Vec<&Candidate> = candidates
        .iter()
        .filter(|candidate| !is_blank(&candidate.puuid))
        .filter(|candidate| {
            let puuid = candidate.puuid.to_ascii_lowercase();
            !used.contains(&puuid) && seen.insert(puuid)
        })
        .collect();

    // 3. the scramble key. it decodes each hidden player's real puuid, and the
    // match counts only when the chat lists that puuid. a player it decodes to
    // anyone else is provably none of the free names, so they stay unmatched
    // and elimination leaves them alone.
    let hidden: Vec<&SessionPlayer> = session
        .my_team
        .iter()
        .filter(|player| player.is_hidden() && !identities.contains_key(&player.key()))
        .collect();
    let obfuscated: Vec<u128> = hidden
        .iter()
        .filter_map(|player| parse_uuid(&player.obfuscated_puuid))
        .collect();
    let listed: Vec<u128> = free
        .iter()
        .filter_map(|candidate| parse_uuid(&candidate.puuid))
        .collect();
    let key = recover_key(&obfuscated, &listed);

    let mut undecoded = Vec::new();
    for player in hidden {
        let decoded = key
            .zip(parse_uuid(&player.obfuscated_puuid))
            .map(|(key, obfuscated)| key ^ obfuscated);
        let Some(decoded) = decoded else {
            undecoded.push(player);
            continue;
        };
        let position = free
            .iter()
            .position(|candidate| parse_uuid(&candidate.puuid) == Some(decoded));
        match position {
            Some(index) => {
                let candidate = free.swap_remove(index);
                identities.insert(player.key(), from_candidate(candidate, MatchKind::Key));
            }
            None => {
                absent.insert(player.key());
            }
        }
    }

    // 4. one hidden player the key could not decode and one free name left:
    // the name is theirs.
    if let ([player], [candidate]) = (undecoded.as_slice(), free.as_slice()) {
        identities.insert(
            player.key(),
            from_candidate(candidate, MatchKind::Elimination),
        );
    }

    Resolved { identities, absent }
}

// an identity for a player the session names. the name comes from the session,
// else from the chat entry with the same puuid; it stays empty when neither has
// one, and the puuid still says exactly who the player is.
fn from_session(
    player: &SessionPlayer,
    puuid: &str,
    candidates: &[Candidate],
    kind: MatchKind,
) -> Identity {
    let (game_name, tag_line) = if player.game_name.is_empty() {
        candidates
            .iter()
            .find(|candidate| candidate.puuid.eq_ignore_ascii_case(puuid))
            .map(|candidate| (candidate.game_name.clone(), candidate.tag_line.clone()))
            .unwrap_or_default()
    } else {
        (player.game_name.clone(), player.tag_line.clone())
    };
    Identity {
        puuid: puuid.to_string(),
        game_name,
        tag_line,
        kind,
        confidence: Confidence::High,
    }
}

fn from_candidate(candidate: &Candidate, kind: MatchKind) -> Identity {
    Identity {
        puuid: candidate.puuid.clone(),
        game_name: candidate.game_name.clone(),
        tag_line: candidate.tag_line.clone(),
        kind,
        confidence: Confidence::High,
    }
}

// what champ select shows about a slot nobody could be matched to exactly.
#[derive(Debug, Clone)]
pub struct SlotHint {
    // the player key (see SessionPlayer::key).
    pub key: String,
    pub position: Option<Position>,
    pub is_autofilled: bool,
    pub spells: (i64, i64),
    // 0 while the slot has neither picked nor declared a champion.
    pub champion_id: i64,
}

// the best guess at who sits in each slot, keyed by player key. candidates are
// the chat names no exact step used, each with their stats. every pairing of
// slots and names is tried, as many pairs as the shorter side allows, and the
// one that fits best overall wins. its lead over the next best pairing says
// how sure the guess is; with nothing to compare against, the guess is low.
pub fn guess(
    slots: &[SlotHint],
    candidates: &[(Candidate, PlayerStats)],
) -> HashMap<String, Identity> {
    let mut seen = HashSet::new();
    let candidates: Vec<&(Candidate, PlayerStats)> = candidates
        .iter()
        .filter(|(candidate, _)| {
            !is_blank(&candidate.puuid) && seen.insert(candidate.puuid.to_ascii_lowercase())
        })
        .collect();
    // more names than a team holds means the chat was not this lobby's alone.
    if slots.len() > MAX_GUESSED || candidates.len() > MAX_GUESSED {
        return HashMap::new();
    }

    let fits: Vec<Vec<f64>> = slots
        .iter()
        .map(|slot| {
            candidates
                .iter()
                .map(|(_, stats)| fit(slot, stats))
                .collect()
        })
        .collect();
    let mut ranked: Vec<(f64, Vec<(usize, usize)>)> = pairings(slots.len(), candidates.len())
        .into_iter()
        .map(|pairs| {
            let total = pairs.iter().map(|&(slot, player)| fits[slot][player]).sum();
            (total, pairs)
        })
        .collect();
    // best first. the sort is stable, so of two equal totals the pairing
    // tried first wins.
    ranked.sort_by(|a, b| b.0.total_cmp(&a.0));
    let Some((best, pairs)) = ranked.first() else {
        return HashMap::new();
    };
    let runner_up = ranked.get(1).map_or(*best, |(total, _)| *total);
    let confidence = lead_confidence(best - runner_up);

    pairs
        .iter()
        .map(|&(slot, player)| {
            let (candidate, _) = candidates[player];
            let identity = Identity {
                confidence,
                ..from_candidate(candidate, MatchKind::Guess)
            };
            (slots[slot].key.clone(), identity)
        })
        .collect()
}

// how well a player's recent games fit a slot: the share of their games in
// the slot's role (a quarter as much when the slot is autofilled, since the
// player is likely off their usual role), half the share with the slot's exact
// summoner spells, and their experience on the slot's champion. anything the
// player has no data for adds nothing.
fn fit(slot: &SlotHint, stats: &PlayerStats) -> f64 {
    let history = stats.history.as_ref();
    let role = match (slot.position, history) {
        (Some(position), Some(history)) => history.role_share(position),
        _ => 0.0,
    };
    let role_weight = if slot.is_autofilled { 0.25 } else { 1.0 };
    let spells = history.map_or(0.0, |history| history.spell_share(slot.spells));
    role_weight * role + 0.5 * spells + champion_fit(slot.champion_id, stats)
}

// games on the champion count for most of it, up to five games; mastery for
// the rest, in full from 10,000 points and by half below that.
fn champion_fit(champion_id: i64, stats: &PlayerStats) -> f64 {
    if champion_id == 0 {
        return 0.0;
    }
    let games = stats
        .history
        .as_ref()
        .map_or(0, |history| history.games_on(champion_id));
    let points = stats
        .mastery
        .as_ref()
        .and_then(|mastery| mastery.get(&champion_id))
        .copied()
        .unwrap_or(0);
    let mastery = if points >= 10_000 {
        1.0
    } else if points > 0 {
        0.5
    } else {
        0.0
    };
    0.7 * (games as f64 / 5.0).min(1.0) + 0.3 * mastery
}

// every way to pair slots with players one to one, as many pairs as the
// smaller side allows, each pairing as (slot, player) index pairs.
fn pairings(slots: usize, players: usize) -> Vec<Vec<(usize, usize)>> {
    let (rows, columns) = (slots.min(players), slots.max(players));
    // every ordered choice of `rows` distinct columns.
    let mut choices: Vec<Vec<usize>> = vec![Vec::new()];
    for _ in 0..rows {
        let mut longer = Vec::new();
        for chosen in &choices {
            for column in (0..columns).filter(|column| !chosen.contains(column)) {
                let mut next = chosen.clone();
                next.push(column);
                longer.push(next);
            }
        }
        choices = longer;
    }
    choices
        .into_iter()
        .map(|chosen| {
            chosen
                .into_iter()
                .enumerate()
                .map(|(row, column)| {
                    if slots <= players {
                        (row, column)
                    } else {
                        (column, row)
                    }
                })
                .collect()
        })
        .collect()
}

fn lead_confidence(lead: f64) -> Confidence {
    // a hair of slack, so a lead of exactly 0.5 or 0.2 is not lost to the
    // rounding of the shares it is made of.
    const SLACK: f64 = 1e-9;
    if lead >= HIGH_LEAD - SLACK {
        Confidence::High
    } else if lead >= MEDIUM_LEAD - SLACK {
        Confidence::Medium
    } else {
        Confidence::Low
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lobby::stats::{GameSummary, PlayerHistory};

    // random-looking stand-ins: the lobby's scramble key, a second key for the
    // ambiguity test, you, a duo partner, four hidden teammates' real puuids,
    // and two chat names that belong to nobody in the lobby.
    const KEY: u128 = 0x84cc_8700_e153_8ee5_3421_f8cd_18fb_2784;
    const OTHER_KEY: u128 = 0x97d4_458c_282c_ad5c_22e5_ebcb_da46_8f06;
    const YOU: u128 = 0xc89b_b161_8b7b_023c_7186_26eb_2edc_d61f;
    const DUO: u128 = 0xabff_fb5c_6800_1666_5f5c_d115_31bd_4bcf;
    const REAL: [u128; 4] = [
        0x964f_eb34_601c_374e_8943_ac6e_9ae3_2dbe,
        0x499d_6a12_a482_fe2e_6e63_9d64_65fd_cf78,
        0x9e3f_4101_125b_f4b1_f679_5b2d_67da_3e32,
        0x3b09_691e_a3e1_8152_935b_b49b_24d2_d57f,
    ];
    const STRANGERS: [u128; 2] = [
        0xa7a8_ad86_a089_9a8c_75c5_0d04_ed48_757c,
        0x281d_9854_8b1e_fd0b_8bbd_b4d1_0407_78cc,
    ];
    const NAMES: [&str; 4] = ["Player One", "Player Two", "Player Three", "Player Four"];

    fn you() -> SessionPlayer {
        SessionPlayer {
            cell_id: 2,
            puuid: format_uuid(YOU),
            game_name: "Example Player".to_string(),
            tag_line: "EXP".to_string(),
            name_visibility_type: "VISIBLE".to_string(),
            ..SessionPlayer::default()
        }
    }

    fn hidden(cell_id: i64, obfuscated: u128) -> SessionPlayer {
        SessionPlayer {
            cell_id,
            obfuscated_puuid: format_uuid(obfuscated),
            name_visibility_type: "HIDDEN".to_string(),
            ..SessionPlayer::default()
        }
    }

    fn visible(cell_id: i64, puuid: u128) -> SessionPlayer {
        SessionPlayer {
            cell_id,
            puuid: format_uuid(puuid),
            name_visibility_type: "UNHIDDEN".to_string(),
            ..SessionPlayer::default()
        }
    }

    fn lobby(my_team: Vec<SessionPlayer>) -> Session {
        Session {
            game_id: 1,
            local_player_cell_id: 2,
            my_team,
            ..Session::default()
        }
    }

    fn candidate(puuid: u128, game_name: &str) -> Candidate {
        Candidate {
            puuid: format_uuid(puuid),
            game_name: game_name.to_string(),
            tag_line: "NA1".to_string(),
        }
    }

    // you plus the four hidden teammates as the riot client lists them.
    fn chat() -> Vec<Candidate> {
        let mut chat = vec![candidate(YOU, "Example Player")];
        chat.extend(
            REAL.iter()
                .zip(NAMES)
                .map(|(&real, name)| candidate(real, name)),
        );
        chat
    }

    // how each slot was matched, in cell order; none when it was not.
    fn kinds(session: &Session, identities: &HashMap<String, Identity>) -> Vec<Option<MatchKind>> {
        session
            .team_in_slot_order()
            .iter()
            .map(|player| identities.get(&player.key()).map(|identity| identity.kind))
            .collect()
    }

    fn name_in(session: &Session, identities: &HashMap<String, Identity>, cell_id: i64) -> String {
        let player = session
            .my_team
            .iter()
            .find(|player| player.cell_id == cell_id)
            .expect("the cell is on the team");
        identities
            .get(&player.key())
            .map(|identity| identity.game_name.clone())
            .unwrap_or_default()
    }

    #[test]
    fn parses_and_formats_uuids() {
        let text = "8c29b6f9-de1a-44c1-ab55-d6ac857a23e0";
        let value = parse_uuid(text).expect("a well formed uuid");
        assert_eq!(value, 0x8c29_b6f9_de1a_44c1_ab55_d6ac_857a_23e0);
        assert_eq!(format_uuid(value), text);
        assert_eq!(parse_uuid(&text.to_uppercase()), Some(value));
        assert_eq!(format_uuid(0), "00000000-0000-0000-0000-000000000000");
        assert_eq!(
            format_uuid(u128::MAX),
            "ffffffff-ffff-ffff-ffff-ffffffffffff"
        );

        assert_eq!(parse_uuid("not-a-uuid"), None);
        // 31 digits, dashed or not.
        assert_eq!(parse_uuid("8c29b6f-de1a-44c1-ab55-d6ac857a23e0"), None);
        assert_eq!(parse_uuid("8c29b6fde1a44c1ab55d6ac857a23e0"), None);
        // the right digits in the wrong shape.
        assert_eq!(parse_uuid("8c29b6f9de1a44c1ab55d6ac857a23e0"), None);
        assert_eq!(parse_uuid("8c29b6f9de1a-44c1-ab55-d6ac-857a23e0"), None);
        assert_eq!(parse_uuid("8c29b6f9-de1a-44c1-ab55-d6ac857a23e0-00"), None);
        assert_eq!(parse_uuid("{8c29b6f9-de1a-44c1-ab55-d6ac857a23e0}"), None);
        assert_eq!(parse_uuid(" 8c29b6f9-de1a-44c1-ab55-d6ac857a23e0"), None);
        // a sign or a stray letter where a digit belongs.
        assert_eq!(parse_uuid("+c29b6f9-de1a-44c1-ab55-d6ac857a23e0"), None);
        assert_eq!(parse_uuid("8c29b6f9-de1a-44c1-ab55-d6ac857a23eg"), None);
        assert_eq!(parse_uuid(""), None);
    }

    #[test]
    fn recovers_the_shared_key() {
        let hidden: Vec<u128> = REAL.iter().map(|real| real ^ KEY).collect();
        // the chat lists you as well, and in no particular order.
        let real = [REAL[2], YOU, REAL[0], REAL[3], REAL[1]];
        assert_eq!(recover_key(&hidden, &real), Some(KEY));
    }

    #[test]
    fn rejects_a_single_hidden_player() {
        // every key scores one, which is not enough.
        assert_eq!(recover_key(&[REAL[0] ^ KEY], &[YOU, REAL[0]]), None);
        assert_eq!(recover_key(&[], &[YOU]), None);
        assert_eq!(recover_key(&[REAL[0] ^ KEY], &[]), None);
    }

    #[test]
    fn tolerates_a_player_missing_from_chat() {
        let hidden: Vec<u128> = REAL.iter().map(|real| real ^ KEY).collect();
        let real = [YOU, REAL[0], REAL[1], REAL[2]];
        assert_eq!(recover_key(&hidden, &real), Some(KEY));
    }

    #[test]
    fn rejects_ambiguous_keys() {
        // two keys, each mapping two of the four hidden players into the chat.
        let hidden = [
            REAL[0] ^ KEY,
            REAL[1] ^ KEY,
            REAL[2] ^ OTHER_KEY,
            REAL[3] ^ OTHER_KEY,
        ];
        let real = [YOU, REAL[0], REAL[1], REAL[2], REAL[3]];
        assert_eq!(recover_key(&hidden, &real), None);

        // two matches alone always tie: swapping them gives a second key,
        // KEY ^ REAL[0] ^ REAL[1], that maps both hidden players into the chat.
        let hidden = [REAL[0] ^ KEY, REAL[1] ^ KEY];
        let real = [YOU, REAL[0], REAL[1]];
        assert_eq!(recover_key(&hidden, &real), None);
    }

    #[test]
    fn resolves_own_visible_key_and_elimination() {
        // a key needs three players in the chat (see rejects_ambiguous_keys),
        // so the five kinds take two lobbies. first: you, a duo partner the
        // session shows without a name, and three hidden players.
        let session = lobby(vec![
            visible(0, DUO),
            hidden(1, REAL[0] ^ KEY),
            you(),
            hidden(3, REAL[1] ^ KEY),
            hidden(4, REAL[2] ^ KEY),
        ]);
        let candidates = [
            candidate(YOU, "Example Player"),
            candidate(DUO, "Duo Partner"),
            candidate(REAL[0], NAMES[0]),
            candidate(REAL[1], NAMES[1]),
            candidate(REAL[2], NAMES[2]),
        ];
        let identities = resolve_exact(&session, &candidates).identities;
        assert_eq!(
            kinds(&session, &identities),
            [
                Some(MatchKind::Visible),
                Some(MatchKind::Key),
                Some(MatchKind::Own),
                Some(MatchKind::Key),
                Some(MatchKind::Key),
            ]
        );
        // your name comes from the session, the duo's from the chat.
        assert_eq!(
            identities[&format_uuid(YOU)],
            Identity {
                puuid: format_uuid(YOU),
                game_name: "Example Player".to_string(),
                tag_line: "EXP".to_string(),
                kind: MatchKind::Own,
                confidence: Confidence::High,
            }
        );
        assert_eq!(name_in(&session, &identities, 0), "Duo Partner");
        assert_eq!(
            identities[&format_uuid(REAL[1] ^ KEY)],
            Identity {
                puuid: format_uuid(REAL[1]),
                game_name: NAMES[1].to_string(),
                tag_line: "NA1".to_string(),
                kind: MatchKind::Key,
                confidence: Confidence::High,
            }
        );
        assert_eq!(name_in(&session, &identities, 1), NAMES[0]);
        assert_eq!(name_in(&session, &identities, 4), NAMES[2]);

        // second: three hidden players the key decodes, and a fourth riot sent
        // without an obfuscated puuid, left with the one unused name.
        let session = lobby(vec![
            hidden(0, REAL[0] ^ KEY),
            hidden(1, REAL[1] ^ KEY),
            you(),
            hidden(3, REAL[2] ^ KEY),
            SessionPlayer {
                obfuscated_puuid: String::new(),
                ..hidden(4, 0)
            },
        ]);
        let identities = resolve_exact(&session, &chat()).identities;
        assert_eq!(
            kinds(&session, &identities),
            [
                Some(MatchKind::Key),
                Some(MatchKind::Key),
                Some(MatchKind::Own),
                Some(MatchKind::Key),
                Some(MatchKind::Elimination),
            ]
        );
        assert_eq!(name_in(&session, &identities, 4), NAMES[3]);
        assert!(identities
            .values()
            .all(|identity| identity.confidence == Confidence::High));
    }

    #[test]
    fn eliminates_without_a_key() {
        // one hidden player is too few for a key, but the one name nobody
        // else has used is theirs.
        let session = lobby(vec![
            visible(0, REAL[1]),
            visible(1, REAL[2]),
            you(),
            visible(3, REAL[3]),
            hidden(4, REAL[0] ^ KEY),
        ]);
        let identities = resolve_exact(&session, &chat()).identities;
        assert_eq!(
            kinds(&session, &identities),
            [
                Some(MatchKind::Visible),
                Some(MatchKind::Visible),
                Some(MatchKind::Own),
                Some(MatchKind::Visible),
                Some(MatchKind::Elimination),
            ]
        );
        assert_eq!(name_in(&session, &identities, 4), NAMES[0]);
    }

    #[test]
    fn never_reveals_players_absent_from_chat() {
        // the fourth hidden player is not in the chat (riot's "hide my name",
        // say), and two chat names belong to nobody the key decodes.
        let session = lobby(vec![
            hidden(0, REAL[0] ^ KEY),
            hidden(1, REAL[1] ^ KEY),
            you(),
            hidden(3, REAL[2] ^ KEY),
            hidden(4, REAL[3] ^ KEY),
        ]);
        let mut candidates = vec![
            candidate(YOU, "Example Player"),
            candidate(REAL[0], NAMES[0]),
            candidate(REAL[1], NAMES[1]),
            candidate(REAL[2], NAMES[2]),
            candidate(STRANGERS[0], "Stranger One"),
            candidate(STRANGERS[1], "Stranger Two"),
        ];
        let resolved = resolve_exact(&session, &candidates);
        assert_eq!(
            kinds(&session, &resolved.identities),
            [
                Some(MatchKind::Key),
                Some(MatchKind::Key),
                Some(MatchKind::Own),
                Some(MatchKind::Key),
                None,
            ]
        );
        // and says so, so a guess leaves them alone too.
        assert_eq!(resolved.absent, HashSet::from([format_uuid(REAL[3] ^ KEY)]));

        // with one name left over the answer is the same: the key already
        // shows that player is someone the chat does not list.
        candidates.pop();
        let resolved = resolve_exact(&session, &candidates);
        assert_eq!(kinds(&session, &resolved.identities)[4], None);
        assert_eq!(resolved.absent.len(), 1);

        // without a key nobody is ruled out: they may be any free name.
        let resolved = resolve_exact(&session, &candidates[..2]);
        assert!(resolved.absent.is_empty());
    }

    #[test]
    fn kinds_serialize_like_the_frontend_expects() {
        let kinds = [
            MatchKind::Own,
            MatchKind::Visible,
            MatchKind::Key,
            MatchKind::Elimination,
            MatchKind::Guess,
        ];
        assert_eq!(
            serde_json::to_value(kinds).unwrap(),
            serde_json::json!(["own", "visible", "key", "elimination", "guess"])
        );
        let levels = [Confidence::High, Confidence::Medium, Confidence::Low];
        assert_eq!(
            serde_json::to_value(levels).unwrap(),
            serde_json::json!(["high", "medium", "low"])
        );
    }

    // a slot that has picked no champion yet, with flash and teleport.
    fn slot(key: &str, position: Position) -> SlotHint {
        SlotHint {
            key: key.to_string(),
            position: Some(position),
            is_autofilled: false,
            spells: (4, 12),
            champion_id: 0,
        }
    }

    // recent games, so many in each role, on champion 1 with flash and ignite.
    fn played(roles: &[(Position, usize)]) -> PlayerStats {
        let games = roles
            .iter()
            .flat_map(|&(position, count)| {
                let game = GameSummary {
                    champion_id: 1,
                    position: Some(position),
                    spells: (4, 14),
                    ..GameSummary::default()
                };
                std::iter::repeat_n(game, count)
            })
            .collect();
        PlayerStats {
            history: Some(PlayerHistory { games }),
            ..PlayerStats::default()
        }
    }

    fn on_champion(mut stats: PlayerStats, champion_id: i64) -> PlayerStats {
        for game in &mut stats.history.as_mut().expect("there are games").games {
            game.champion_id = champion_id;
        }
        stats
    }

    fn with_spells(mut stats: PlayerStats, spells: (i64, i64)) -> PlayerStats {
        for game in &mut stats.history.as_mut().expect("there are games").games {
            game.spells = spells;
        }
        stats
    }

    // who was guessed into which slot, sorted by slot key.
    fn placed(guesses: &HashMap<String, Identity>) -> Vec<(&str, &str)> {
        let mut placed: Vec<(&str, &str)> = guesses
            .iter()
            .map(|(key, identity)| (key.as_str(), identity.game_name.as_str()))
            .collect();
        placed.sort_unstable();
        placed
    }

    #[test]
    fn places_a_jungle_main_in_the_jungle_slot() {
        let slots = [slot("top", Position::Top), slot("jungle", Position::Jungle)];
        // the jungle main is listed first, so pairing in order would be wrong.
        let candidates = [
            (
                candidate(REAL[0], NAMES[0]),
                played(&[(Position::Jungle, 18), (Position::Top, 2)]),
            ),
            (
                candidate(REAL[1], NAMES[1]),
                played(&[(Position::Top, 19), (Position::Jungle, 1)]),
            ),
        ];
        let guesses = guess(&slots, &candidates);
        assert_eq!(placed(&guesses), [("jungle", NAMES[0]), ("top", NAMES[1])]);
        assert_eq!(
            guesses["jungle"],
            Identity {
                puuid: format_uuid(REAL[0]),
                game_name: NAMES[0].to_string(),
                tag_line: "NA1".to_string(),
                kind: MatchKind::Guess,
                confidence: Confidence::High,
            }
        );
        assert_eq!(guesses["top"].kind, MatchKind::Guess);
        assert_eq!(guesses["top"].confidence, Confidence::High);
    }

    #[test]
    fn champion_breaks_a_role_tie() {
        // both play mid and top equally often, but only one of them has games
        // on the champion the mid slot picked.
        let slots = [
            SlotHint {
                champion_id: 103,
                ..slot("mid", Position::Middle)
            },
            slot("top", Position::Top),
        ];
        let even = [(Position::Middle, 6), (Position::Top, 6)];
        let candidates = [
            (candidate(REAL[0], NAMES[0]), played(&even)),
            (
                candidate(REAL[1], NAMES[1]),
                on_champion(played(&even), 103),
            ),
        ];
        let guesses = guess(&slots, &candidates);
        assert_eq!(placed(&guesses), [("mid", NAMES[1]), ("top", NAMES[0])]);
        assert_eq!(guesses["mid"].confidence, Confidence::High);

        // mastery alone tips it too, by less.
        let mut mastered = played(&even);
        mastered.mastery = Some(HashMap::from([(103, 12_000)]));
        let candidates = [
            (candidate(REAL[0], NAMES[0]), played(&even)),
            (candidate(REAL[1], NAMES[1]), mastered),
        ];
        let guesses = guess(&slots, &candidates);
        assert_eq!(placed(&guesses), [("mid", NAMES[1]), ("top", NAMES[0])]);
        assert_eq!(guesses["mid"].confidence, Confidence::Medium);
    }

    #[test]
    fn autofilled_slot_trusts_role_less() {
        // a support main who runs exhaust, and a mid main who runs ignite,
        // the spell the support slot took.
        let support = SlotHint {
            spells: (4, 14),
            ..slot("support", Position::Utility)
        };
        let candidates = [
            (
                candidate(REAL[0], NAMES[0]),
                with_spells(played(&[(Position::Utility, 10)]), (4, 3)),
            ),
            (
                candidate(REAL[1], NAMES[1]),
                played(&[(Position::Middle, 10)]),
            ),
        ];

        // in a slot of their own choosing the role decides: 1.0 against 0.5.
        let guesses = guess(std::slice::from_ref(&support), &candidates);
        assert_eq!(placed(&guesses), [("support", NAMES[0])]);
        assert_eq!(guesses["support"].confidence, Confidence::High);

        // autofilled, the role counts a quarter: 0.25 against the spells' 0.5.
        let autofilled = SlotHint {
            is_autofilled: true,
            ..support
        };
        let guesses = guess(&[autofilled], &candidates);
        assert_eq!(placed(&guesses), [("support", NAMES[1])]);
        assert_eq!(guesses["support"].confidence, Confidence::Medium);
    }

    #[test]
    fn low_confidence_when_close() {
        let slots = [slot("top", Position::Top), slot("jungle", Position::Jungle)];
        let candidates = [
            (
                candidate(REAL[0], NAMES[0]),
                played(&[(Position::Top, 10), (Position::Jungle, 10)]),
            ),
            (
                candidate(REAL[1], NAMES[1]),
                played(&[(Position::Top, 11), (Position::Jungle, 9)]),
            ),
        ];
        let guesses = guess(&slots, &candidates);
        // the slightly more top-heavy player goes top, but only just.
        assert_eq!(placed(&guesses), [("jungle", NAMES[0]), ("top", NAMES[1])]);
        assert!(guesses
            .values()
            .all(|identity| identity.kind == MatchKind::Guess
                && identity.confidence == Confidence::Low));
    }

    #[test]
    fn places_everyone_when_counts_differ() {
        let slots = [slot("top", Position::Top), slot("jungle", Position::Jungle)];
        let jungler = (
            candidate(REAL[0], NAMES[0]),
            played(&[(Position::Jungle, 20)]),
        );
        let top_laner = (candidate(REAL[1], NAMES[1]), played(&[(Position::Top, 20)]));
        let mid_laner = (
            candidate(REAL[2], NAMES[2]),
            played(&[(Position::Middle, 20)]),
        );

        // more names than slots: the two that fit best are placed.
        let guesses = guess(
            &slots,
            &[mid_laner.clone(), jungler.clone(), top_laner.clone()],
        );
        assert_eq!(placed(&guesses), [("jungle", NAMES[0]), ("top", NAMES[1])]);
        assert!(guesses
            .values()
            .all(|identity| identity.confidence == Confidence::High));

        // more slots than names: the one name goes where it fits best.
        let guesses = guess(&slots, std::slice::from_ref(&jungler));
        assert_eq!(placed(&guesses), [("jungle", NAMES[0])]);

        // a player without any data still takes the slot that is left.
        let unknown = (candidate(REAL[3], NAMES[3]), PlayerStats::default());
        let guesses = guess(&slots, &[unknown, top_laner]);
        assert_eq!(placed(&guesses), [("jungle", NAMES[3]), ("top", NAMES[1])]);
    }

    #[test]
    fn guesses_only_what_it_can_compare() {
        let slots = [slot("top", Position::Top), slot("jungle", Position::Jungle)];
        let top_laner = (candidate(REAL[1], NAMES[1]), played(&[(Position::Top, 20)]));
        assert!(guess(&[], std::slice::from_ref(&top_laner)).is_empty());
        assert!(guess(&slots, &[]).is_empty());

        // one slot and one name leave nothing to compare against.
        let guesses = guess(&slots[..1], std::slice::from_ref(&top_laner));
        assert_eq!(placed(&guesses), [("top", NAMES[1])]);
        assert_eq!(guesses["top"].confidence, Confidence::Low);

        // the same player listed twice is placed once, and a blank puuid
        // names nobody.
        let blank = (
            Candidate {
                puuid: String::new(),
                ..candidate(REAL[2], NAMES[2])
            },
            played(&[(Position::Jungle, 20)]),
        );
        let guesses = guess(&slots, &[top_laner.clone(), top_laner.clone(), blank]);
        assert_eq!(placed(&guesses), [("top", NAMES[1])]);

        // more names than a team holds means the chat was not this lobby's.
        let crowd: Vec<(Candidate, PlayerStats)> = (0..6u128)
            .map(|index| {
                (
                    candidate(STRANGERS[0] ^ index, "Stranger"),
                    PlayerStats::default(),
                )
            })
            .collect();
        assert!(guess(&slots, &crowd).is_empty());
    }

    #[test]
    fn confidence_follows_the_lead() {
        assert_eq!(lead_confidence(1.7), Confidence::High);
        assert_eq!(lead_confidence(0.5), Confidence::High);
        // a lead of 0.5 made of rounded shares still counts as 0.5.
        assert_eq!(lead_confidence(0.7 - 0.2), Confidence::High);
        assert_eq!(lead_confidence(0.49), Confidence::Medium);
        assert_eq!(lead_confidence(0.2), Confidence::Medium);
        assert_eq!(lead_confidence(0.3 - 0.1), Confidence::Medium);
        assert_eq!(lead_confidence(0.19), Confidence::Low);
        assert_eq!(lead_confidence(0.0), Confidence::Low);
    }
}
