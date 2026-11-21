// the league and riot clients change their payloads between patches, and a
// field can arrive as null or as a number outside the range its docs give. a
// field read through `lenient` falls back to its default instead, so one odd
// value costs that field and never the whole payload (a champ select session
// that failed to parse would blank the lobby for the whole champ select).

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

pub fn lenient<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + DeserializeOwned,
{
    let value = Value::deserialize(deserializer)?;
    Ok(T::deserialize(value).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default, Deserialize, PartialEq)]
    #[serde(default, rename_all = "camelCase")]
    struct Sample {
        #[serde(deserialize_with = "lenient")]
        name: String,
        #[serde(deserialize_with = "lenient")]
        spell_id: i64,
        #[serde(deserialize_with = "lenient")]
        autofilled: bool,
        #[serde(deserialize_with = "lenient")]
        team: Vec<i64>,
    }

    #[test]
    fn odd_values_fall_back_to_their_defaults() {
        let sample: Sample = serde_json::from_str(
            r#"{"name": null, "spellId": 18446744073709551615, "autofilled": "yes", "team": null}"#,
        )
        .expect("an odd field should not fail the payload");
        assert_eq!(sample, Sample::default());

        let sample: Sample = serde_json::from_str(
            r#"{"name": "Gromp", "spellId": 4, "autofilled": true, "team": [1, 2]}"#,
        )
        .expect("a normal payload parses");
        assert_eq!(
            sample,
            Sample {
                name: "Gromp".to_string(),
                spell_id: 4,
                autofilled: true,
                team: vec![1, 2],
            }
        );
        // a missing field takes the default too.
        let sample: Sample = serde_json::from_str("{}").expect("an empty object parses");
        assert_eq!(sample, Sample::default());
    }
}
