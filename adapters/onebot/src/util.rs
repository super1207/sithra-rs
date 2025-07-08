use serde::{Deserialize, Deserializer};

pub(crate) fn de_str_from_num<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let num: i64 = Deserialize::deserialize(deserializer)?;
    Ok(num.to_string())
}
