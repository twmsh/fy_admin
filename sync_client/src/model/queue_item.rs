use chrono::{DateTime, Local};
use fy_base::sync::rabbitmq_type::BoxLogMessage;
use fy_base::util::time_format::long_ts_format;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskItem {
    pub id: u64, // timestamp 流水号

    #[serde(rename = "type")]
    pub t_type: TaskItemType, // 任务类型

    #[serde(with = "long_ts_format")]
    pub ts: DateTime<Local>, // 时间戳

    pub sub_type: u32,   // 子命令类型
    pub payload: String, // 具体内容
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TaskItemType {
    SyncTimer, // 定时器触发，去同步
    HeartBeat, // 心跳信息
    ServerCmd, // 服务端的命令
}

pub type RabbitmqItem = BoxLogMessage;
