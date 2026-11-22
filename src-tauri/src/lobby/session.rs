// the champ select session the league client reports on
// /lol-champ-select/v1/session, reduced to the fields the lobby reads. every
// field defaults because the client omits or empties fields depending on the
// queue and the phase: a hidden ranked teammate has no puuid and no name, a
// blind pick slot has no position, and so on. a field that arrives as null or
// out of range takes its default as well (see lenient), so one odd value
// never blanks the lobby.

use serde::{Deserialize, Serialize};

use crate::lenient::lenient;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Session {
    // a new lobby (a requeue after a dodge, say) gets a new game id.
    #[serde(deserialize_with = "lenient")]
    pub game_id: i64,
    #[serde(deserialize_with = "lenient")]
    pub queue_id: i64,
    #[serde(deserialize_with = "lenient")]
    pub local_player_cell_id: i64,
    #[serde(deserialize_with = "lenient")]
    pub my_team: Vec<SessionPlayer>,
    #[serde(deserialize_with = "lenient")]
    pub timer: SessionTimer,
    #[serde(deserialize_with = "lenient")]
    pub chat_details: ChatDetails,
    // the bans and picks of champ select in order, each group taken
    // together (simultaneous bans share one group).
    #[serde(deserialize_with = "lenient")]
    pub actions: Vec<Vec<SessionAction>>,
    // theirTeam arrives with identities scrubbed by riot, so it is not read.
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SessionPlayer {
    // the pick-order cell. a pick-order swap moves players between cells, so
    // the cell never tells players apart (see key).
    #[serde(deserialize_with = "lenient")]
    pub cell_id: i64,
    #[serde(deserialize_with = "lenient")]
    pub champion_id: i64,
    // the champion declared during planning, before a pick is locked in.
    #[serde(deserialize_with = "lenient")]
    pub champion_pick_intent: i64,
    #[serde(deserialize_with = "lenient")]
    pub spell1_id: i64,
    #[serde(deserialize_with = "lenient")]
    pub spell2_id: i64,
    // "top", "jungle", "middle", "bottom" or "utility"; empty in blind pick.
    #[serde(deserialize_with = "lenient")]
    pub assigned_position: String,
    #[serde(deserialize_with = "lenient")]
    pub is_autofilled: bool,
    // empty (or zeroed) while the player is hidden.
    #[serde(deserialize_with = "lenient")]
    pub puuid: String,
    // a per-lobby stand-in for the puuid, which hidden players still carry.
    #[serde(deserialize_with = "lenient")]
    pub obfuscated_puuid: String,
    #[serde(deserialize_with = "lenient")]
    pub game_name: String,
    #[serde(deserialize_with = "lenient")]
    pub tag_line: String,
    // "HIDDEN" for an anonymous ranked teammate, "VISIBLE" or "UNHIDDEN" when
    // the name shows (yourself, a party member, a queue that hides nobody).
    #[serde(deserialize_with = "lenient")]
    pub name_visibility_type: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SessionTimer {
    // PLANNING, BAN_PICK, FINALIZATION, and so on.
    #[serde(deserialize_with = "lenient")]
    pub phase: String,
    // milliseconds remaining in the current phase.
    #[serde(deserialize_with = "lenient")]
    pub adjusted_time_left_in_phase: i64,
    // the phase's full length in milliseconds.
    #[serde(deserialize_with = "lenient")]
    pub total_time_in_phase: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SessionAction {
    #[serde(deserialize_with = "lenient")]
    pub actor_cell_id: i64,
    // your team's action, not the enemy's.
    #[serde(deserialize_with = "lenient")]
    pub is_ally_action: bool,
    // on the clock right now.
    #[serde(deserialize_with = "lenient")]
    pub is_in_progress: bool,
    // "ban", "pick", or a step that is no turn, such as "ten_bans_reveal".
    #[serde(rename = "type", deserialize_with = "lenient")]
    pub kind: String,
}

// where champ select stands. unknown when the session does not say, for
// example a pick and ban phase with no action in progress.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    #[default]
    Unknown,
    Planning,
    Ban,
    Pick,
    Finalization,
}

// which team is on a ban or pick turn right now.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Acting {
    #[default]
    None,
    Ally,
    Enemy,
    Both,
}

impl Stage {
    pub fn as_str(self) -> &'static str {
        match self {
            Stage::Unknown => "unknown",
            Stage::Planning => "planning",
            Stage::Ban => "ban",
            Stage::Pick => "pick",
            Stage::Finalization => "finalization",
        }
    }

    fn has_turns(self) -> bool {
        matches!(self, Stage::Ban | Stage::Pick)
    }
}

impl Acting {
    pub fn as_str(self) -> &'static str {
        match self {
            Acting::None => "none",
            Acting::Ally => "ally",
            Acting::Enemy => "enemy",
            Acting::Both => "both",
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ChatDetails {
    // the champ select chat room. the riot client's conversation id for the
    // same room contains it, which is how its chat participants are filtered.
    #[serde(deserialize_with = "lenient")]
    pub multi_user_chat_id: String,
}

impl Session {
    // the team in the order the client lays it out, by cell. aliases are handed
    // out by position in this order.
    pub fn team_in_slot_order(&self) -> Vec<&SessionPlayer> {
        let mut team: Vec<&SessionPlayer> = self.my_team.iter().collect();
        team.sort_by_key(|player| player.cell_id);
        team
    }

    // anonymous champ select: riot hides at least one teammate's name.
    pub fn is_anonymous(&self) -> bool {
        self.my_team.iter().any(SessionPlayer::is_hidden)
    }

    pub fn local_player(&self) -> Option<&SessionPlayer> {
        self.my_team
            .iter()
            .find(|player| player.cell_id == self.local_player_cell_id)
    }

    // the stage from the timer, and in pick and ban from the turns on the
    // clock: bans before picks.
    pub fn stage(&self) -> Stage {
        match self.timer.phase.as_str() {
            "PLANNING" => Stage::Planning,
            "FINALIZATION" | "GAME_STARTING" => Stage::Finalization,
            "BAN_PICK" => {
                if self.turns().any(|action| action.kind == "ban") {
                    Stage::Ban
                } else if self.turns().any(|action| action.kind == "pick") {
                    Stage::Pick
                } else {
                    Stage::Unknown
                }
            }
            _ => Stage::Unknown,
        }
    }

    pub fn acting(&self) -> Acting {
        if !self.stage().has_turns() {
            return Acting::None;
        }
        let ally = self.turns().any(|action| action.is_ally_action);
        let enemy = self.turns().any(|action| !action.is_ally_action);
        match (ally, enemy) {
            (true, true) => Acting::Both,
            (true, false) => Acting::Ally,
            (false, true) => Acting::Enemy,
            (false, false) => Acting::None,
        }
    }

    // a teammate in this cell is banning or picking right now.
    pub fn is_cell_acting(&self, cell_id: i64) -> bool {
        self.stage().has_turns()
            && self
                .turns()
                .any(|action| action.is_ally_action && action.actor_cell_id == cell_id)
    }

    // the bans and picks on the clock right now.
    fn turns(&self) -> impl Iterator<Item = &SessionAction> {
        self.actions.iter().flatten().filter(|action| {
            action.is_in_progress && (action.kind == "ban" || action.kind == "pick")
        })
    }
}

impl SessionPlayer {
    // what tells players apart for the whole lobby. a hidden player keeps their
    // obfuscated puuid when a swap moves them to another cell; a visible player
    // falls back to the real puuid, and a slot with neither (a bot) to its cell.
    pub fn key(&self) -> String {
        if !is_blank(&self.obfuscated_puuid) {
            self.obfuscated_puuid.clone()
        } else if let Some(puuid) = self.real_puuid() {
            puuid.to_string()
        } else {
            format!("cell-{}", self.cell_id)
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.name_visibility_type == "HIDDEN"
    }

    // the picked or hovered champion, else the one declared during planning.
    pub fn champion(&self) -> i64 {
        if self.champion_id != 0 {
            self.champion_id
        } else {
            self.champion_pick_intent
        }
    }

    // the real puuid, when the session shows one.
    pub fn real_puuid(&self) -> Option<&str> {
        (!is_blank(&self.puuid)).then_some(self.puuid.as_str())
    }
}

// riot empties or zeroes the puuid of a hidden player and leaves it empty for a
// bot or an unfilled slot; either way it names nobody.
pub fn is_blank(id: &str) -> bool {
    id.chars().all(|c| c == '0' || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    const RANKED: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/session_ranked.json"
    ));

    fn ranked() -> Session {
        serde_json::from_str(RANKED).expect("the fixture should parse")
    }

    fn player(session: &Session, cell_id: i64) -> &SessionPlayer {
        session
            .my_team
            .iter()
            .find(|player| player.cell_id == cell_id)
            .expect("the cell is on the team")
    }

    #[test]
    fn parses_a_hidden_ranked_session() {
        let session = ranked();
        assert_eq!(session.game_id, 5_123_456_789);
        assert_eq!(session.queue_id, 420);
        assert_eq!(session.my_team.len(), 5);
        assert_eq!(session.timer.phase, "BAN_PICK");
        assert_eq!(session.timer.adjusted_time_left_in_phase, 27_412);
        assert_eq!(
            session.chat_details.multi_user_chat_id,
            "c1~54a43c1848168346a1f1d305e999e202"
        );
        assert!(session.is_anonymous());

        let me = session
            .local_player()
            .expect("the local player is on the team");
        assert_eq!(me.cell_id, 2);
        assert!(!me.is_hidden());
        assert_eq!(me.game_name, "Example Player");
        assert_eq!(me.key(), "d45f9fb2-21da-4b64-954b-261f303c8054");

        let hidden: Vec<i64> = session
            .my_team
            .iter()
            .filter(|player| player.is_hidden())
            .map(|player| player.cell_id)
            .collect();
        assert_eq!(hidden, [0, 1, 3, 4]);
        let top = player(&session, 0);
        assert!(top.puuid.is_empty());
        assert_eq!(top.key(), "8c29b6f9-de1a-44c1-ab55-d6ac857a23e0");
        assert_eq!(top.key(), top.obfuscated_puuid);

        // cell 3 has only declared a champion so far.
        let bottom = player(&session, 3);
        assert_eq!(bottom.champion_id, 0);
        assert_eq!(bottom.champion(), 222);
        assert_eq!(player(&session, 1).champion(), 64);

        let support = player(&session, 4);
        assert!(support.is_autofilled);
        assert_eq!(support.assigned_position, "utility");
        assert_eq!((support.spell1_id, support.spell2_id), (4, 3));
        let autofilled = session
            .my_team
            .iter()
            .filter(|player| player.is_autofilled)
            .count();
        assert_eq!(autofilled, 1);
    }

    // the ranked fixture in another phase, with these actions.
    fn session(phase: &str, actions: serde_json::Value) -> Session {
        let mut value: serde_json::Value =
            serde_json::from_str(RANKED).expect("the fixture should parse");
        value["timer"]["phase"] = serde_json::json!(phase);
        value["actions"] = actions;
        serde_json::from_value(value).expect("the session should parse")
    }

    fn action(cell: i64, kind: &str, ally: bool, in_progress: bool) -> serde_json::Value {
        serde_json::json!({
            "actorCellId": cell,
            "championId": 0,
            "completed": !in_progress,
            "id": cell + 1,
            "isAllyAction": ally,
            "isInProgress": in_progress,
            "pickTurn": 1,
            "type": kind,
        })
    }

    fn acting_cells(session: &Session) -> Vec<i64> {
        (0..10)
            .filter(|&cell| session.is_cell_acting(cell))
            .collect()
    }

    #[test]
    fn reads_the_phase_length_and_actions() {
        let session = ranked();
        assert_eq!(session.timer.total_time_in_phase, 30_000);
        assert_eq!(session.actions.len(), 2);
        assert_eq!(session.actions[0][1].actor_cell_id, 1);
        assert_eq!(session.actions[1][0].kind, "pick");
        assert!(session.actions[0][0].is_ally_action);
        // every action in the fixture is done, so nobody is on a turn.
        assert_eq!(session.stage(), Stage::Unknown);
        assert_eq!(session.acting(), Acting::None);
    }

    #[test]
    fn planning_has_no_turn() {
        let session = session("PLANNING", serde_json::json!([]));
        assert_eq!(session.stage(), Stage::Planning);
        assert_eq!(session.acting(), Acting::None);
        assert!(acting_cells(&session).is_empty());
    }

    #[test]
    fn simultaneous_bans_are_both_teams() {
        let session = session(
            "BAN_PICK",
            serde_json::json!([[
                action(0, "ban", true, true),
                action(1, "ban", true, true),
                action(5, "ban", false, true),
            ]]),
        );
        assert_eq!(session.stage(), Stage::Ban);
        assert_eq!(session.acting(), Acting::Both);
        assert_eq!(acting_cells(&session), [0, 1]);
    }

    #[test]
    fn your_teams_pick() {
        let session = session(
            "BAN_PICK",
            serde_json::json!([
                [action(0, "ban", true, false)],
                [action(2, "pick", true, true)],
            ]),
        );
        assert_eq!(session.stage(), Stage::Pick);
        assert_eq!(session.acting(), Acting::Ally);
        assert_eq!(acting_cells(&session), [2]);
    }

    #[test]
    fn enemy_pick() {
        let session = session(
            "BAN_PICK",
            serde_json::json!([[action(7, "pick", false, true)]]),
        );
        assert_eq!(session.stage(), Stage::Pick);
        assert_eq!(session.acting(), Acting::Enemy);
        assert!(acting_cells(&session).is_empty());
    }

    #[test]
    fn blind_pick_is_your_teams_pick() {
        let picks: Vec<serde_json::Value> = (0..5)
            .map(|cell| action(cell, "pick", true, true))
            .collect();
        let session = session("BAN_PICK", serde_json::json!([picks]));
        assert_eq!(session.stage(), Stage::Pick);
        assert_eq!(session.acting(), Acting::Ally);
        assert_eq!(acting_cells(&session), [0, 1, 2, 3, 4]);
    }

    #[test]
    fn finalization_has_no_turn() {
        for phase in ["FINALIZATION", "GAME_STARTING"] {
            let session = session(phase, serde_json::json!([[action(0, "pick", true, false)]]));
            assert_eq!(session.stage(), Stage::Finalization);
            assert_eq!(session.acting(), Acting::None);
            assert!(acting_cells(&session).is_empty());
        }
    }

    #[test]
    fn ignores_actions_that_are_not_turns() {
        let session = session(
            "BAN_PICK",
            serde_json::json!([[action(0, "ten_bans_reveal", true, true)]]),
        );
        assert_eq!(session.stage(), Stage::Unknown);
        assert_eq!(session.acting(), Acting::None);
        assert!(acting_cells(&session).is_empty());
    }

    #[test]
    fn odd_actions_do_not_break_the_session() {
        for actions in [
            serde_json::json!(null),
            serde_json::json!("x"),
            serde_json::json!([[{"type": 5}]]),
        ] {
            let session = session("BAN_PICK", actions);
            assert_eq!(session.game_id, 5_123_456_789);
            assert_eq!(session.stage(), Stage::Unknown);
            assert_eq!(session.acting(), Acting::None);
        }
    }

    #[test]
    fn slot_order_sorts_by_cell() {
        let session = Session {
            my_team: [4, 0, 2]
                .into_iter()
                .map(|cell_id| SessionPlayer {
                    cell_id,
                    ..SessionPlayer::default()
                })
                .collect(),
            ..Session::default()
        };
        let cells: Vec<i64> = session
            .team_in_slot_order()
            .iter()
            .map(|player| player.cell_id)
            .collect();
        assert_eq!(cells, [0, 2, 4]);
    }

    #[test]
    fn key_falls_back_to_puuid_then_cell() {
        let visible = SessionPlayer {
            cell_id: 1,
            puuid: "d45f9fb2-21da-4b64-954b-261f303c8054".to_string(),
            ..SessionPlayer::default()
        };
        assert_eq!(visible.key(), "d45f9fb2-21da-4b64-954b-261f303c8054");
        assert_eq!(
            visible.real_puuid(),
            Some("d45f9fb2-21da-4b64-954b-261f303c8054")
        );

        // a bot or an unfilled slot names nobody, and neither does a zeroed puuid.
        let bot = SessionPlayer {
            cell_id: 3,
            ..SessionPlayer::default()
        };
        assert_eq!(bot.key(), "cell-3");
        let zeroed = SessionPlayer {
            cell_id: 4,
            puuid: "00000000-0000-0000-0000-000000000000".to_string(),
            ..SessionPlayer::default()
        };
        assert_eq!(zeroed.key(), "cell-4");
        assert_eq!(zeroed.real_puuid(), None);
    }
}
