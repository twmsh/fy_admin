use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use crate::dao::base_model::BaseBoxLog;
use fy_base::util::time_format::long_ts_format;

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

impl From<BoxLogMessage> for BaseBoxLog {
    fn from(msg: BoxLogMessage) -> Self {
        Self {
            id: 0,
            box_hwid: msg.hwid,
            box_ips: msg.ips,
            log_type: msg.c_type,
            log_level: msg.level,
            log_payload: msg.payload,
            create_time: Local::now(),
        }
    }
}