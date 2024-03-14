use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer};

pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;

    match s {
        Some(s) => match NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S") {
            Ok(result) => Ok(Some(result)),
            Err(_) => Ok(None),
        },
        None => Ok(None),
    }
}
