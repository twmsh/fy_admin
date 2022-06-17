pub mod queue_item;

use chrono::{DateTime, Local};

use fy_base::util::time_format::long_ts_format;
use serde::{Deserialize, Serialize};

//--------------------------------------
// 从rabbitmq读取的消息
#[derive(Debug, Serialize, Deserialize)]
pub struct RabbitmqInMessage {
    pub id: u64, // 流水号

    #[serde(rename = "type")]
    pub m_type: String, // 消息类型

    pub payload: String, // 具体内容

    #[serde(with = "long_ts_format")]
    pub ts: DateTime<Local>, // 时间戳
}
