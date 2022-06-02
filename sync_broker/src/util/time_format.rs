
pub mod long_ts_format {
    use chrono::prelude::*;
    use serde::{self, Deserialize, Deserializer, Serializer};

    use crate::util::utils;

    pub fn serialize<S>(
        date: &DateTime<Local>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let s = format!("{}", date.format(utils::DATETIME_FMT_LONG));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Local>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        utils::parse_localtime_str(&s, utils::DATETIME_FMT_LONG).map_err(serde::de::Error::custom)
    }
}


pub mod short_ts_format {
    use chrono::prelude::*;
    use serde::{self, Deserialize, Deserializer, Serializer};

    use crate::util::utils;

    pub fn serialize<S>(
        date: &DateTime<Local>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let s = format!("{}", date.format(utils::DATETIME_FMT_SHORT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Local>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        utils::parse_localtime_str(&s, utils::DATETIME_FMT_SHORT).map_err(serde::de::Error::custom)
    }
}