use crate::util::time_format::long_ts_format;
use chrono::prelude::*;
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

pub const BOXLOGMESSAGE_TYPE_STATUS: &str = "status";
pub const BOXLOGMESSAGE_TYPE_LOG: &str = "log";

// log级别 ： debug(0), info(1), warn(2), error(3)
pub const BOXLOGMESSAGE_LEVEL_DEBUG: i16 = 0;
pub const BOXLOGMESSAGE_LEVEL_INFO: i16 = 1;
pub const BOXLOGMESSAGE_LEVEL_WARN: i16 = 2;
pub const BOXLOGMESSAGE_LEVEL_ERROR: i16 = 3;
