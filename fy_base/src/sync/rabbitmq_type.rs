
use chrono::prelude::*;
use crate::util::time_format::long_ts_format;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BoxLogMessage {
    pub hwid: String,
    pub ips: String,

    #[serde(rename = "type")]
    pub c_type: String,
    pub level: i16,
    pub payload: String,

    #[serde(with = "long_ts_format")]
    pub ts: DateTime<Local>,
}

