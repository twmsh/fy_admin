use crate::dao::base_model::BaseBoxLog;
use crate::dao::Dao;
use crate::service::rabbitmq::model::BoxLogMessage;
use lapin::message::Delivery;
use lapin::options::BasicAckOptions;
use tracing::error;

pub async fn process_boxlog_message(dao: Dao, delivery: Delivery) -> Result<(), lapin::Error> {

    // save to db
    let message = match serde_json::from_reader::<_, BoxLogMessage>(delivery.data.as_slice()) {
        Ok(v) => v,
        Err(e) => {
            error!(
                "error, RabbitmqService, process_boxlog_message, err: {:?}",
                e
            );
            return Ok(());
        }
    };

    let obj: BaseBoxLog = message.into();
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

    Ok(())
}
