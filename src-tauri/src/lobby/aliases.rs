// the names the client shows in place of hidden ranked teammates ("Gromp",
// "Krug", ...). the client gives each player the name for the slot they held
// when the lobby formed, so a player keeps their alias after a pick-order swap
// moves them. the names and their order come from the 16.23 client's
// MY_TEAM_HIDDEN_NAME_KEYS and its en_us strings.

use std::collections::HashMap;

pub const ALIASES: [&str; 20] = [
    "Gromp",
    "Krug",
    "Murk Wolf",
    "Raptor",
    "Scuttle Crab",
    "Minion",
    "Void Grub",
    "Sentinel",
    "Brambleback",
    "Wraith",
    "Poro",
    "Spiderling",
    "Meep",
    "Stellacorn",
    "Sprite",
    "Silverwing",
    "Drake-Hound",
    "Basilisk",
    "Tuskbeast",
    "Wyvern",
];

// the slot each player held when expose first saw them in this lobby, keyed by
// the player's key (see SessionPlayer::key). a new lobby gets a new book.
#[derive(Debug, Default)]
pub struct AliasBook {
    first_slots: HashMap<String, usize>,
}

impl AliasBook {
    // the first call for a key pins its slot; later calls ignore current_slot,
    // so the alias survives a swap.
    pub fn alias(&mut self, key: &str, current_slot: usize) -> &'static str {
        let slot = *self
            .first_slots
            .entry(key.to_string())
            .or_insert(current_slot);
        ALIASES[slot % ALIASES.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_slot_decides_the_alias() {
        let mut book = AliasBook::default();
        let first_five: Vec<&str> = (0..5)
            .map(|slot| book.alias(&format!("player-{slot}"), slot))
            .collect();
        assert_eq!(
            first_five,
            ["Gromp", "Krug", "Murk Wolf", "Raptor", "Scuttle Crab"]
        );
        assert_eq!(book.alias("player-7", 7), "Sentinel");
        // the client wraps around the twenty names.
        assert_eq!(book.alias("player-23", 23), "Raptor");
    }

    #[test]
    fn alias_survives_cell_swap() {
        let mut book = AliasBook::default();
        assert_eq!(book.alias("a", 0), "Gromp");
        // a pick-order swap moves the player to another slot; the alias stays.
        assert_eq!(book.alias("a", 4), "Gromp");
        // the next lobby's book starts over.
        let mut book = AliasBook::default();
        assert_eq!(book.alias("a", 4), "Scuttle Crab");
    }
}
