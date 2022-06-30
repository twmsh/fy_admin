
use std::sync::Arc;
use std::time::Duration;

use crate::app_ctx::AppCtx;

use fy_base::util::rabbitmq::init_conn_props;
use fy_base::util::service::Service;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;


use lapin::options::{
    BasicPublishOptions, ConfirmSelectOptions,
};
use lapin::{
    options::{ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
};
use tokio::time::sleep;

use tracing::{debug, error, info};
use fy_base::api::upload_api::{NotifyCarQueueItem, NotifyFaceQueueItem};
use crate::queue_item::{CarQueue, FaceQueue};

pub struct RabbitmqService {
    pub ctx: Arc<AppCtx>,
    pub face_queue: Arc<FaceQueue>,
    pub car_queue: Arc<CarQueue>,
    wait: u64, // retry interval, second
}

impl RabbitmqService {
    pub fn new(
        ctx: Arc<AppCtx>,
        face_queue: Arc<FaceQueue>,
        car_queue: Arc<CarQueue>,
    ) -> Self {
        Self {
            ctx,
            face_queue,
            car_queue,
            wait: 2,
        }
    }

    fn increate_wait(&mut self) -> u64 {
        let max = 60 * 3;
        self.wait *= 2;
        if self.wait > max {
            self.wait = max;
        }
        self.wait
    }

    fn reset_wait(&mut self) -> u64 {
        self.wait = 2;
        self.wait
    }

    async fn init_rabbitmq_connection(
        &self,
        conn_props: ConnectionProperties,
    ) -> Result<Connection, lapin::Error> {
        let url = self.ctx.cfg.rabbitmq.url.as_str();
        let conn = Connection::connect(url, conn_props).await?;
        Ok(conn)
    }

    async fn init_rabbitmq_durable_queue(
        &self,
        channel: &Channel,
        exchange_name: &str,
        queue_name: &str,
        route_key: &str,
    ) -> Result<(), lapin::Error> {
        // 声明 exchange
        let _ = channel
            .exchange_declare(
                exchange_name,
                ExchangeKind::Topic,
                ExchangeDeclareOptions {
                    passive: false,
                    durable: true, // 持久化，rabbitmq重启后，还存在
                    auto_delete: false,
                    internal: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;

        // 声明 queue
        let _queue = channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions {
                    passive: false,
                    durable: true, // 持久化，rabbitmq重启后，还存在
                    exclusive: false,
                    auto_delete: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;

        // 绑定
        let _ = channel
            .queue_bind(
                queue_name,
                exchange_name,
                route_key,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok(())
    }

    async fn init_rabbitmq_exchange_queue(
        &self,
        conn: &Connection,
    ) -> Result<Channel, lapin::Error> {
        let channel = conn.create_channel().await?;

        // 设置发送方确认
        let _ = channel
            .confirm_select(ConfirmSelectOptions::default())
            .await?;

        let _face_queue = self
            .init_rabbitmq_durable_queue(
                &channel,
                self.ctx.cfg.rabbitmq.face.exchange.as_str(),
                self.ctx.cfg.rabbitmq.face.queue.as_str(),
                self.ctx.cfg.rabbitmq.face.route_key.as_str(),
            )
            .await?;

        let _car_queue = self
            .init_rabbitmq_durable_queue(
                &channel,
                self.ctx.cfg.rabbitmq.car.exchange.as_str(),
                self.ctx.cfg.rabbitmq.car.queue.as_str(),
                self.ctx.cfg.rabbitmq.car.route_key.as_str(),
            )
            .await?;

        Ok(channel)
    }

    async fn wait_a_moment(dur: Duration, mut exit_rx: Receiver<i64>) -> bool {
        let wait_a_moment = sleep(dur);
        tokio::select! {
            _ = exit_rx.changed() => {
                info!("RabbitmqService, recv signal, will exit");
                true
            }
            _ = wait_a_moment => {
                false
            }
        }
    }

    async fn check_rabbitmq_error<T>(
        &mut self,
        msg: &str,
        v: Result<T, lapin::Error>,
        exit_rx: Receiver<i64>,
    ) -> (i8, Option<T>) {
        match v {
            Ok(v) => {
                let _ = self.reset_wait();
                (0, Some(v))
            }
            Err(e) => {
                error!("error, RabbitmqService, {}, err: {:?}", msg, e);

                let wait = self.increate_wait();
                if Self::wait_a_moment(Duration::from_secs(wait), exit_rx).await {
                    // 1: 退出循环 break
                    (1, None)
                } else {
                    // 2: 跳过后续，继续 contiue
                    (2, None)
                }
            }
        }
    }

    pub async fn do_run(mut self, conn_props: ConnectionProperties, exit_rx: Receiver<i64>) {

        loop {
            // init connecton
            let conn = self.init_rabbitmq_connection(conn_props.clone()).await;
            let (rst, conn) = self
                .check_rabbitmq_error("init_rabbitmq_connection", conn, exit_rx.clone())
                .await;

            let conn = match rst {
                1 => {
                    break;
                }
                2 => {
                    continue;
                }
                _ => conn.unwrap(),
            };

            // init exchange, queue
            let queue = self.init_rabbitmq_exchange_queue(&conn).await;
            let (rst, channel) = self
                .check_rabbitmq_error("init_rabbitmq_exchange_queue", queue, exit_rx.clone())
                .await;

            let channel = match rst {
                1 => {
                    break;
                }
                2 => {
                    continue;
                }
                _ => channel.unwrap(),
            };

            debug!("RabbitmqService, rabbitmq inited");

            // loop message
            let loop_rst = self
                .loop_message(&channel, exit_rx.clone())
                .await;

            let (rst, _) = self
                .check_rabbitmq_error("loop_message", loop_rst, exit_rx.clone())
                .await;
            let _ = match rst {
                1 => {
                    // wait_a_moment中收到退出信号
                    break;
                }
                2 => {
                    continue;
                }
                _ => {
                    // loop_message里面收到退出信号
                    break;
                }
            };
        }

        info!("RabbitmqService exit.");
    }

    async fn loop_message(
        &mut self,
        channel: &Channel,
        mut exit_rx: Receiver<i64>,
    ) -> Result<(), lapin::Error> {
        loop {
            tokio::select! {
                _ = exit_rx.changed() => {
                    info!("RabbitmqService, recv signal, will exit");
                    break;
                }
                item = self.face_queue.pop() => {
                    let _ = self.process_out_face_rabbitmsg(channel,item).await?;
                }
                item = self.car_queue.pop() => {
                    let _ = self.process_out_car_rabbitmsg(channel,item).await?;
                }
            }
        }

        Ok(())
    }


    async fn process_out_face_rabbitmsg(
        &mut self,
        channel: &Channel,
        item: NotifyFaceQueueItem,
    ) -> Result<(), lapin::Error> {
        let exchange = self.ctx.cfg.rabbitmq.face.exchange.as_str();
        let route_key = self.ctx.cfg.rabbitmq.face.route_key.as_str();
        let payload = serde_json::to_string(&item);
        if let Err(e) = payload {
            error!(
                "error, rabbitmq_service, serde_json::to_string, err: {:?}",
                e
            );
            return Ok(());
        }
        let payload = payload.unwrap();

        let expire = self.ctx.cfg.rabbitmq.face.expire * 60 * 1000; // 分钟 * 60* 1000

        let publish_confirm = channel
            .basic_publish(
                exchange,
                route_key,
                BasicPublishOptions::default(),
                payload.as_bytes(),
                BasicProperties::default().with_expiration(expire.to_string().into()),
            )
            .await?
            .await?;
        debug!("RabbitmqService, publish_confirm, {:?}, {}", publish_confirm,item.uuid);

        Ok(())
    }

    async fn process_out_car_rabbitmsg(
        &mut self,
        channel: &Channel,
        item: NotifyCarQueueItem,
    ) -> Result<(), lapin::Error> {
        let exchange = self.ctx.cfg.rabbitmq.car.exchange.as_str();
        let route_key = self.ctx.cfg.rabbitmq.car.route_key.as_str();
        let payload = serde_json::to_string(&item);
        if let Err(e) = payload {
            error!(
                "error, rabbitmq_service, serde_json::to_string, err: {:?}",
                e
            );
            return Ok(());
        }
        let payload = payload.unwrap();

        let expire = self.ctx.cfg.rabbitmq.car.expire * 60 * 1000; // 分钟 * 60* 1000

        let publish_confirm = channel
            .basic_publish(
                exchange,
                route_key,
                BasicPublishOptions::default(),
                payload.as_bytes(),
                BasicProperties::default().with_expiration(expire.to_string().into()),
            )
            .await?
            .await?;
        debug!("RabbitmqService, publish_confirm, {:?}, {}", publish_confirm,item.uuid);

        Ok(())
    }
}

impl Service for RabbitmqService {
    fn run(self, exit_rx: Receiver<i64>) -> JoinHandle<()> {
        let conn_props = init_conn_props();
        let this = self;
        tokio::spawn(this.do_run(conn_props, exit_rx))
    }
}
