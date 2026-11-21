// the champ select session the league client reports on
// /lol-champ-select/v1/session, reduced to the fields the lobby reads. every
// field defaults because the client omits or empties fields depending on the
// queue and the phase: a hidden ranked teammate has no puuid and no name, a
// blind pick slot has no position, and so on.

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Session {
    // a new lobby (a requeue after a dodge, say) gets a new game id.
    pub game_id: i64,
    pub queue_id: i64,
    pub local_player_cell_id: i64,
    pub my_team: Vec<SessionPlayer>,
    pub timer: SessionTimer,
    pub chat_details: ChatDetails,
    // theirTeam arrives with identities scrubbed by riot, so it is not read.
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SessionPlayer {
    // the pick-order cell. a pick-order swap moves players between cells, so
    // the cell never tells players apart (see key).
    pub cell_id: i64,
    pub champion_id: i64,
    // the champion declared during planning, before a pick is locked in.
    pub champion_pick_intent: i64,
    pub spell1_id: i64,
    pub spell2_id: i64,
    // "top", "jungle", "middle", "bottom" or "utility"; empty in blind pick.
    pub assigned_position: String,
    pub is_autofilled: bool,
    // empty (or zeroed) while the player is hidden.
    pub puuid: String,
    // a per-lobby stand-in for the puuid, which hidden players still carry.
    pub obfuscated_puuid: String,
    pub game_name: String,
    pub tag_line: String,
    // "HIDDEN" for an anonymous ranked teammate, "VISIBLE" or "UNHIDDEN" when
    // the name shows (yourself, a party member, a queue that hides nobody).
    pub name_visibility_type: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SessionTimer {
    // PLANNING, BAN_PICK, FINALIZATION, and so on.
    pub phase: String,
    // milliseconds remaining in the current phase.
    pub adjusted_time_left_in_phase: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ChatDetails {
    // the champ select chat room. the riot client's conversation id for the
    // same room contains it, which is how its chat participants are filtered.
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
fn is_blank(id: &str) -> bool {
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
