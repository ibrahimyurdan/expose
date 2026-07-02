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
            "https://u.gg/lol/multisearch?region={}&summoners={}",
            ugg_region(region),
            encode(&joined(teammates, '-'))
        ),
        // default and "opgg". only op.gg and u.gg expose a shareable
        // multi-search url; deeplol and tracker.gg do not (tracker has no lol
        // multi-search at all, deeplol keeps the extra players in app state and
        // never in the url), so they are not offered. anything unknown — including
        // a stale "deeplol"/"tracker" still in the store — falls back to op.gg.
        _ => format!(
            "https://www.op.gg/multisearch/{region}?summoners={}",
            encode(&joined(teammates, '#'))
        ),
    };
    Some(url)
}

// u.gg addresses regions by riot platform id, which only sometimes equals the
// op.gg web slug plus a trailing "1". map explicitly so kr, oce, eune, ru and
// the rest are not silently malformed (the old code just appended "1").
fn ugg_region(slug: &str) -> &'static str {
    match slug {
        "na" => "na1",
        "euw" => "euw1",
        "eune" => "eun1",
        "kr" => "kr",
        "jp" => "jp1",
        "oce" => "oc1",
        "br" => "br1",
        "lan" => "la1",
        "las" => "la2",
        "ru" => "ru",
        "tr" => "tr1",
        "sg" | "sea" => "sg2",
        "ph" => "ph2",
        "th" => "th2",
        "tw" => "tw2",
        "vn" => "vn2",
        _ => "na1",
    }
}

// the single-summoner profile link used by each teammate row, on the chosen
// scout site. region is the op.gg web slug (for example "na").
pub fn profile_url(
    provider: &str,
    region: Option<&str>,
    game_name: &str,
    tag_line: &str,
) -> String {
    let slug = region.filter(|s| !s.is_empty()).unwrap_or("na");
    match provider {
        "ugg" => format!(
            "https://u.gg/lol/profile/{}/{}-{}/overview",
            ugg_region(slug),
            encode(game_name),
            encode(tag_line)
        ),
        // default and "opgg".
        _ => format!(
            "https://www.op.gg/summoners/{slug}/{}-{}",
            encode(game_name),
            encode(tag_line)
        ),
    }
}

// the display label for the chosen scout site, shown on the row button.
pub fn provider_label(provider: &str) -> &'static str {
    match provider {
        "ugg" => "U.GG",
        _ => "OP.GG",
    }
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
                    scout_url: String::new(),
                    scout_label: String::new(),
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
        assert!(url.starts_with("https://u.gg/lol/multisearch?region=na1&summoners="));
        assert!(url.contains("Faker-KR1%2CHide%20on%20bush-NA1"));
    }

    #[test]
    fn ugg_maps_platform_regions_instead_of_appending_one() {
        // these are the cases the old "{region}1" concatenation got wrong.
        let kr = scout_url("ugg", &team(), Some("kr")).unwrap();
        assert!(
            kr.starts_with("https://u.gg/lol/multisearch?region=kr&"),
            "{kr}"
        );
        let oce = scout_url("ugg", &team(), Some("oce")).unwrap();
        assert!(
            oce.starts_with("https://u.gg/lol/multisearch?region=oc1&"),
            "{oce}"
        );
        let eune = scout_url("ugg", &team(), Some("eune")).unwrap();
        assert!(
            eune.starts_with("https://u.gg/lol/multisearch?region=eun1&"),
            "{eune}"
        );
    }

    #[test]
    fn unknown_provider_falls_back_to_opgg() {
        assert!(scout_url("nonsense", &team(), Some("euw"))
            .unwrap()
            .starts_with("https://www.op.gg/multisearch/euw"));
    }

    #[test]
    fn opgg_profile_url_is_region_correct_and_encoded() {
        assert_eq!(
            profile_url("opgg", Some("euw"), "Faker", "KR1"),
            "https://www.op.gg/summoners/euw/Faker-KR1"
        );
        assert_eq!(
            profile_url("opgg", Some("na"), "Hide on bush", "NA 1"),
            "https://www.op.gg/summoners/na/Hide%20on%20bush-NA%201"
        );
        assert!(
            profile_url("opgg", None, "Name", "NA1").starts_with("https://www.op.gg/summoners/na/")
        );
    }

    #[test]
    fn ugg_profile_url_uses_platform_region_and_overview() {
        assert_eq!(
            profile_url("ugg", Some("na"), "Faker", "NA1"),
            "https://u.gg/lol/profile/na1/Faker-NA1/overview"
        );
        assert_eq!(
            profile_url("ugg", Some("kr"), "Hide on bush", "KR1"),
            "https://u.gg/lol/profile/kr/Hide%20on%20bush-KR1/overview"
        );
    }

    #[test]
    fn provider_label_maps_known_and_unknown() {
        assert_eq!(provider_label("opgg"), "OP.GG");
        assert_eq!(provider_label("ugg"), "U.GG");
        assert_eq!(provider_label("nonsense"), "OP.GG");
    }
}
