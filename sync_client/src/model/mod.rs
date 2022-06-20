pub mod queue_item;

use chrono::{DateTime, Local};

use crate::model::queue_item::{TaskItem, TaskItemType};
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

    pub hw_id: Option<String>,

    #[serde(with = "long_ts_format")]
    pub ts: DateTime<Local>, // 时间戳
}

// RabbitmqInMessage 转成 sync_notify, reset{db,camera}, reboot,

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPayload {
    //  清除所有db
    pub db: bool,

    // 清除所有摄像头
    pub camera: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogPayload {
    //  对应某个任务(task)的id
    pub ref_id: u64, // 流水号

    // log消息发生的地方
    pub source: String,

    // log 消息内容
    pub msg: String,
}

//---------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct StatusCamera {
    pub uuid: String,

    // 采集地址
    pub url: String,

    // 摄像头类型 1：人脸，2：车辆，3：人脸+车辆
    #[serde(rename = "type")]
    pub c_type: i16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusDb {
    pub uuid: String,

    // 采集地址
    pub capacity: u64,

    pub used: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusPayload {
    //  对应某个任务(task)的id
    pub ref_id: u64,
    // 流水号
    pub cameras: Vec<StatusCamera>,
    pub dbs: Vec<StatusDb>,
}

//-----------------------------------------------------
impl TryFrom<RabbitmqInMessage> for TaskItem {
    type Error = String;

    fn try_from(value: RabbitmqInMessage) -> Result<Self, Self::Error> {
        let main_type = TaskItemType::ServerCmd;
        let sub_type = match value.m_type.as_str() {
            "sync" => 0,
            "reset" => 1,
            "reboot" => 2,
            _ => {
                return Err(format!("unknown message type: {}", value.m_type));
            }
        };

        Ok(TaskItem {
            id: value.id,
            t_type: main_type,
            ts: value.ts,
            sub_type,
            payload: value.payload,
        })
    }
}
