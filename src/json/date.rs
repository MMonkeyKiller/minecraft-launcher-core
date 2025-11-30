use std::str::FromStr;

use chrono::{ DateTime, FixedOffset, NaiveDateTime };
use serde::{ Serializer, Deserializer, Serialize, Deserialize };

#[derive(Debug, Clone, PartialEq)]
pub struct Date {
  date: DateTime<FixedOffset>,
}

impl Date {
  pub fn inner(&self) -> &DateTime<FixedOffset> {
    &self.date
  }
}

impl From<Date> for DateTime<FixedOffset> {
  fn from(val: Date) -> Self {
    val.date
  }
}

impl From<DateTime<FixedOffset>> for Date {
  fn from(date: DateTime<FixedOffset>) -> Self {
    Date { date }
  }
}

impl Serialize for Date {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    serializer.serialize_str(&self.date.to_rfc3339())
  }
}

impl<'de> Deserialize<'de> for Date {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
    let date_str = String::deserialize(deserializer)?;
    if let Ok(date) = DateTime::parse_from_rfc3339(&date_str) {
      Ok(Date { date })
    } else {
      let date = NaiveDateTime::from_str(&date_str)
        .map(|dt| dt.and_utc().into())
        .map_err(serde::de::Error::custom)?;
      Ok(Date { date })
    }
  }
}
