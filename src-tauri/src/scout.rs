// builds a multi-search url that opens every teammate at once on the chosen
// stats site. region is the op.gg style web slug (for example "na").

use crate::models::Teammate;

pub fn scout_url(provider: &str, teammates: &[Teammate], region: Option<&str>) -> Option<String> {
    if teammates.is_empty() {
        return None;
    }
    let region = region.filter(|slug| !slug.is_empty()).unwrap_or("na");

    let url = match provider {
        "ugg" => format!(
            "https://u.gg/multisearch?region={region}1&summoners={}",
            encode(&joined(teammates, '-'))
        ),
        "deeplol" => format!(
            "https://deeplol.gg/multi/{region}/{}",
            encode(&joined(teammates, '#'))
        ),
        "tracker" => format!(
            "https://tracker.gg/lol/multisearch/{region}/{}",
            encode(&joined(teammates, '#'))
        ),
        // default and "opgg"
        _ => format!(
            "https://www.op.gg/multisearch/{region}?summoners={}",
            encode(&joined(teammates, '#'))
        ),
    };
    Some(url)
}

// joins teammates as "name<sep>tag,name<sep>tag,...". the separator between
// name and tag differs by site (op.gg uses '#', u.gg uses '-').
fn joined(teammates: &[Teammate], sep: char) -> String {
    teammates
        .iter()
        .map(|teammate| format!("{}{sep}{}", teammate.game_name, teammate.tag_line))
        .collect::<Vec<_>>()
        .join(",")
}

// percent-encodes the whole summoners string, so '#', ',', and spaces survive.
fn encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len() * 2);
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Teammate;

    fn team() -> Vec<Teammate> {
        ["Faker#KR1", "Hide on bush#NA1"]
            .iter()
            .map(|id| {
                let (name, tag) = id.split_once('#').unwrap();
                Teammate {
                    cell_id: 0,
                    puuid: String::new(),
                    game_name: name.to_string(),
                    tag_line: tag.to_string(),
                    summoner_level: 0,
                    profile_icon_id: 0,
                    assigned_position: String::new(),
                    champion_id: 0,
                    champion_pick_intent: 0,
                    opgg_url: String::new(),
                }
            })
            .collect()
    }

    #[test]
    fn empty_team_has_no_url() {
        assert!(scout_url("opgg", &[], Some("na")).is_none());
    }

    #[test]
    fn opgg_uses_hash_and_encodes_the_whole_list() {
        assert_eq!(
            scout_url("opgg", &team(), Some("na")).unwrap(),
            "https://www.op.gg/multisearch/na?summoners=Faker%23KR1%2CHide%20on%20bush%23NA1"
        );
    }

    #[test]
    fn ugg_uses_dash_and_a_platform_region() {
        let url = scout_url("ugg", &team(), Some("na")).unwrap();
        assert!(url.starts_with("https://u.gg/multisearch?region=na1&summoners="));
        assert!(url.contains("Faker-KR1%2CHide%20on%20bush-NA1"));
    }

    #[test]
    fn unknown_provider_falls_back_to_opgg() {
        assert!(scout_url("nonsense", &team(), Some("euw"))
            .unwrap()
            .starts_with("https://www.op.gg/multisearch/euw"));
    }
}
