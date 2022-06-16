use chrono::{DateTime, Local};
use serde::{Serialize,Deserialize};
use fy_base::util::time_format::long_ts_format;

#[derive(Debug,Serialize,Deserialize)]
pub struct TaskItem {
    pub id: u64,                // timestamp 流水号

    #[serde(rename="type")]
    pub t_type: TaskItemType,         // 任务类型

    #[serde(with = "long_ts_format")]
    pub ts: DateTime<Local>,    // 时间戳
    pub payload: String,        // 具体内容
}

#[derive(Debug,Serialize,Deserialize)]
pub enum TaskItemType {
    SyncTimer,  // 定时器触发，去同步
    HeartBeat,  // 心跳信息
    ServerCmd,  // 服务端的命令
}

//--------------------------------------
// 从rabbitmq读取的消息
pub struct RabbitmqInMessage {
    pub id: u64,                // 流水号

    #[serde(rename="type")]
    pub m_type: String,         // 消息类型

    pub payload: String,        // 具体内容

    #[serde(with = "long_ts_format")]
    pub ts: DateTime<Local>,    // 时间戳
}

// 发送到 rabbitmq的消息，等同于sync_server中的BoxLogMessage
pub struct RabbitmqItem {
    pub hwid: String,
    pub ips: String,

    #[serde(rename = "type")]
    pub c_type: String,
    pub level: i16,
    pub payload: String,

    #[serde(with = "long_ts_format")]
    pub ts: DateTime<Local>,
}