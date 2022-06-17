use chrono::Local;
use fy_base::sync::rabbitmq_type::BoxLogMessage;
use crate::dao::base_model::BaseBoxLog;

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
