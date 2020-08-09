use chrono::NaiveDateTime;
use serde::de;
use std::fmt;

struct NaiveDateTimeVisitor;

impl<'de> de::Visitor<'de> for NaiveDateTimeVisitor {
    type Value = NaiveDateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string represents chrono::NaiveDateTime")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S.%f") {
            Ok(t) => Ok(t),
            Err(_) => Err(de::Error::invalid_value(de::Unexpected::Str(s), &self)),
        }
    }
}

/// # from_timestamp
/// Deserialize a postgres timestamp to `chrono::NaiveDateTime`
///
/// `usage`
/// ```
/// #[derive(Deserialize, Debug)]
/// pub struct User {
///     pub id: Uuid,
///     pub username: String,
///     #[serde(deserialize_with = "from_timestamp")] // 👈
///     pub created: NaiveDateTime,
/// }
/// ```
pub fn from_timestamp<'de, D>(d: D) -> Result<NaiveDateTime, D::Error>
where
    D: de::Deserializer<'de>,
{
    d.deserialize_str(NaiveDateTimeVisitor)
}
