use crate::dao::base_model::BaseBoxLog;
use crate::dao::Dao;
use fy_base::sync::rabbitmq_type::BoxLogMessage;
use lapin::message::Delivery;

use tracing::{error, warn};

pub async fn process_boxlog_message(dao: Dao, delivery: Delivery) {
    // payload 转成 BoxLogMessage
    let message = match serde_json::from_reader::<_, BoxLogMessage>(delivery.data.as_slice()) {
        Ok(v) => v,
        Err(e) => {
            error!(
                "error, RabbitmqService, process_boxlog_message, err: {:?}",
                e
            );
            return;
        }
    };

    // BoxLogMessage 转成 数据库 entity对象
    let obj: BaseBoxLog = message.into();

    // 保存
    match obj.insert(&dao.pool, &dao.tz).await {
        Ok(_v) => {
            // 保存成功
        }
        Err(e) => {
            error!(
                "error, RabbitmqService, process_boxlog_message, err: {:?}",
                e
            );
        }
    };

    match dao
        .update_latest_online(obj.box_hwid.as_str(), obj.create_time)
        .await
    {
        Ok(saved) => {
            if !saved {
                warn!(
                    "warn, RabbitmqService, update_latest_online({}), return: false",
                    obj.box_hwid
                );
            }
        }
        Err(e) => {
            error!(
                "error, RabbitmqService, update_latest_online({}), err: {:?}",
                obj.box_hwid, e
            );
        }
    };
}
