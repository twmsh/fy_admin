pub mod process_message;
pub mod model;

use std::sync::Arc;

use lapin::message::Delivery;
use lapin::options::{BasicAckOptions, BasicConsumeOptions};
use lapin::{
    options::{ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions},
    types::FieldTable,
    Channel, Connection, ConnectionProperties, Consumer, ExchangeKind, Queue,
};

use tokio::{
    sync::watch::Receiver,
    task::JoinHandle,
    time::{sleep, Duration},
};

use tokio_stream::StreamExt;

use tracing::{error, info};

use crate::error::AppError;
use crate::{
    app_ctx::AppCtx
};
use fy_base::{
    util::{rabbitmq::init_conn_props, service::Service},
};
use crate::service::rabbitmq::process_message::process_boxlog_message;


pub struct RabbitmqService {
    pub ctx: Arc<AppCtx>,
    wait: u64, // retry interval, second
}

impl RabbitmqService {
    pub fn new(ctx: Arc<AppCtx>) -> Self {
        Self { ctx, wait: 2 }
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

    async fn init_rabbitmq_exchange_queue(
        &self,
        conn: &Connection,
    ) -> Result<(Channel, Queue), lapin::Error> {
        let channel = conn.create_channel().await?;

        // 声明 exchange
        let exchange_name = self.ctx.cfg.rabbitmq.exchange.as_str();
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
        let queue_name = self.ctx.cfg.rabbitmq.queue.as_str();
        let queue = channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions {
                    passive: false,
                    durable: false, // 持久化，rabbitmq重启后，还存在
                    exclusive: false,
                    auto_delete: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;

        // 绑定
        let route_key = self.ctx.cfg.rabbitmq.route_key.as_str();
        let _ = channel
            .queue_bind(
                queue_name,
                exchange_name,
                route_key,
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;

        Ok((channel, queue))
    }

    async fn wait_a_moment(dur: Duration, mut exit_rx: Receiver<i64>) -> bool {
        let wait_a_moment = sleep(dur);
        tokio::select! {
            _ = exit_rx.changed() => {
                info!("WebService, recv signal, will exit");
                true
            }
            _ = wait_a_moment => {
                false
            }
        }
    }

    // 除了ack出错外，其他错误需要catch住。
    async fn process_msg(&mut self, delivery: Delivery) -> Result<(), lapin::Error> {
        // println!("delivery: {:?}", delivery);

        let content = String::from_utf8_lossy(&delivery.data).to_string();
        info!("<-- {}", content);

        let _ = delivery.ack(BasicAckOptions::default()).await?;

        let _ = process_boxlog_message(self.ctx.dao.clone(),delivery).await?;

        Ok(())
    }

    async fn loop_message(
        &mut self,
        consumer: &mut Consumer,
        mut exit_rx: Receiver<i64>,
    ) -> Result<(), AppError> {
        loop {
            tokio::select! {
                delivery = consumer.next() => {
                    match delivery {
                        Some(Ok(v)) => {
                            // 处理消息
                            let _ = self.process_msg(v).await?;

                        }
                        Some(Err(e)) => {
                            // consume出错
                            return Err(e.into());
                        }
                        None => {
                            return Err("consume.next() return none".into());
                        }
                    }
                }
                _ = exit_rx.changed() => {
                    info!("WebService, recv signal, will exit");
                    break;
                }
            }
        }
        Ok(())
    }

    // rabbitmq报错要重新开始，要等待一定时长
    // 等待时候，需要关注退出信号
    async fn run(mut self, conn_props: ConnectionProperties, exit_rx: Receiver<i64>) {
        loop {
            // 创建connection
            let conn = self.init_rabbitmq_connection(conn_props.clone()).await;
            if let Err(e) = conn {
                error!(
                    "error, RabbitmqService, init_rabbitmq_connection, err: {:?}",
                    e
                );
                let wait = self.increate_wait();
                let exited = Self::wait_a_moment(Duration::from_secs(wait), exit_rx.clone()).await;
                if exited {
                    break;
                } else {
                    continue;
                }
            } else {
                let _ = self.reset_wait();
            }
            let conn = conn.unwrap();

            // 定义 exchange, queue
            let queue = self.init_rabbitmq_exchange_queue(&conn).await;
            if let Err(e) = queue {
                error!(
                    "error, RabbitmqService, init_rabbitmq_exchange_queue, err: {:?}",
                    e
                );
                let wait = self.increate_wait();
                let exited = Self::wait_a_moment(Duration::from_secs(wait), exit_rx.clone()).await;
                if exited {
                    break;
                } else {
                    continue;
                }
            } else {
                let _ = self.reset_wait();
            }
            let (channel, queue) = queue.unwrap();

            //处理消息
            let consumer_tag = "sync_server";
            let mut consumer = channel
                .basic_consume(
                    queue.name().as_str(),
                    consumer_tag,
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await
                .unwrap();

            if let Err(e) = self.loop_message(&mut consumer, exit_rx.clone()).await {
                error!("error, RabbitmqService, loop_message, err: {:?}", e);
                let wait = self.increate_wait();
                let exited = Self::wait_a_moment(Duration::from_secs(wait), exit_rx.clone()).await;
                if exited {
                    break;
                } else {
                    continue;
                }
            } else {
                // 收到退出信号，退出
                break;
            }
        }

        info!("RabbitmqService exit.");
    }
}

impl Service for RabbitmqService {
    fn run(self, exit_rx: Receiver<i64>) -> JoinHandle<()> {
        let conn_props = init_conn_props();
        let this = self;
        tokio::spawn(this.run(conn_props, exit_rx))
    }
}
