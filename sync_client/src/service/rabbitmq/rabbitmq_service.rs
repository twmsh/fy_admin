use deadqueue::unlimited::Queue;
use std::sync::Arc;
use std::time::Duration;

use crate::app_ctx::AppCtx;
use crate::model::queue_item::{RabbitmqItem, TaskItem};
use fy_base::util::rabbitmq::init_conn_props;
use fy_base::util::service::Service;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;

use crate::model::RabbitmqInMessage;
use lapin::message::Delivery;
use lapin::options::{
    BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, ConfirmSelectOptions,
};
use lapin::{
    options::{ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, Consumer, ExchangeKind,
};
use tokio::time::sleep;
use tokio_stream::StreamExt;
use tracing::{debug, error, info};

pub struct RabbitmqService {
    pub ctx: Arc<AppCtx>,
    pub task_queue: Arc<Queue<TaskItem>>,
    pub rabbitmq_queue: Arc<Queue<RabbitmqItem>>,
    wait: u64, // retry interval, second
}

impl RabbitmqService {
    pub fn new(
        ctx: Arc<AppCtx>,
        task_queue: Arc<Queue<TaskItem>>,
        rabbitmq_queue: Arc<Queue<RabbitmqItem>>,
    ) -> Self {
        Self {
            ctx,
            task_queue,
            rabbitmq_queue,
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

    async fn init_rabbitmq_log_queue(
        &self,
        channel: &Channel,
        exchange_name: &str,
        queue_name: &str,
        route_key: &str,
    ) -> Result<lapin::Queue, lapin::Error> {
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
        let queue = channel
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

        Ok(queue)
    }

    async fn init_rabbitmq_cmd_queue(
        &self,
        channel: &Channel,
        exchange_name: &str,
        queue_name: &str,
        route_key: &str,
    ) -> Result<lapin::Queue, lapin::Error> {
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
        let queue = channel
            .queue_declare(
                queue_name,
                QueueDeclareOptions {
                    passive: false,
                    durable: false,
                    exclusive: false,  // 排他
                    auto_delete: true, // 自动删除
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

        Ok(queue)
    }

    async fn init_rabbitmq_exchange_queue(
        &self,
        conn: &Connection,
    ) -> Result<(Channel, lapin::Queue), lapin::Error> {
        let channel = conn.create_channel().await?;

        // 设置发送方确认
        let _ = channel
            .confirm_select(ConfirmSelectOptions::default())
            .await?;

        let _log_queue = self
            .init_rabbitmq_log_queue(
                &channel,
                self.ctx.cfg.rabbitmq.log.exchange.as_str(),
                self.ctx.cfg.rabbitmq.log.queue.as_str(),
                self.ctx.cfg.rabbitmq.log.route_key.as_str(),
            )
            .await?;

        let cmd_queue_name = format!("box_cmd_{}", self.ctx.hw_id);

        let cmd_queue = self
            .init_rabbitmq_cmd_queue(
                &channel,
                self.ctx.cfg.rabbitmq.cmd.exchange.as_str(),
                cmd_queue_name.as_str(),
                self.ctx.cfg.rabbitmq.cmd.route_key.as_str(),
            )
            .await?;

        Ok((channel, cmd_queue))
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
        let consumer_tag = format!("sync_client-{}", self.ctx.hw_id);

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
            let (rst, queue) = self
                .check_rabbitmq_error("init_rabbitmq_exchange_queue", queue, exit_rx.clone())
                .await;
            let (channel, cmd_queue) = match rst {
                1 => {
                    break;
                }
                2 => {
                    continue;
                }
                _ => queue.unwrap(),
            };

            // init consumer
            let cmd_consumer = channel
                .basic_consume(
                    cmd_queue.name().as_str(),
                    consumer_tag.as_str(),
                    BasicConsumeOptions::default(),
                    FieldTable::default(),
                )
                .await;

            let (rst, cmd_consumer) = self
                .check_rabbitmq_error("basic_consume", cmd_consumer, exit_rx.clone())
                .await;
            let mut cmd_consumer = match rst {
                1 => {
                    break;
                }
                2 => {
                    continue;
                }
                _ => cmd_consumer.unwrap(),
            };

            debug!("RabbitmqService, rabbitmq inited");

            // loop message
            let loop_rst = self
                .loop_message(&mut cmd_consumer, exit_rx.clone(), &channel)
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
        consumer: &mut Consumer,
        mut exit_rx: Receiver<i64>,
        channel: &Channel,
    ) -> Result<(), lapin::Error> {
        loop {
            tokio::select! {
                delivery = consumer.next() => {
                    match delivery {
                        Some(Ok(v)) => {
                            // 处理消息
                            let _ = self.process_in_rabbitmsg(v).await?;

                        }
                        Some(Err(e)) => {
                            // consume出错
                            return Err(e);
                        }
                        None => {
                            let io_err = std::io::Error::new(std::io::ErrorKind::Other, "consumer.next()");
                            let lapin_err = lapin::Error::IOError(Arc::new(io_err));
                            return Err(lapin_err);
                        }
                    }
                }
                _ = exit_rx.changed() => {
                    info!("RabbitmqService, recv signal, will exit");
                    break;
                }
                item = self.rabbitmq_queue.pop() => {
                    let _ = self.process_out_rabbitmsg(channel,item).await?;
                }

            }
        }

        Ok(())
    }

    async fn process_in_rabbitmsg(&mut self, delivery: Delivery) -> Result<(), lapin::Error> {
        // ack it
        let _ = delivery.ack(BasicAckOptions::default()).await?;

        let msg: RabbitmqInMessage = match serde_json::from_reader(delivery.data.as_slice()) {
            Ok(v) => v,
            Err(e) => {
                error!("error, Rabbitmq_service, process_in_rabbitmsg, {:?}", e);
                return Ok(());
            }
        };

        // 检查是否满足hw_id的限定
        if let Some(ref hw_id) = msg.hw_id {
            if !hw_id.is_empty() && hw_id.eq_ignore_ascii_case(self.ctx.hw_id.as_str()) {
                // msg中存在hw_id限定，与本box不符合，该消息丢弃
                debug!("Rabbitmq_service, process_in_rabbitmsg, skip, {:?}", hw_id);
                return Ok(());
            }
        }

        let task_item: TaskItem = match msg.try_into() {
            Ok(v) => v,
            Err(e) => {
                error!("error, Rabbitmq_service, process_in_rabbitmsg, {:?}", e);
                return Ok(());
            }
        };

        debug!(
            "Rabbitmq_service, push to task_queue, {:?}, {}",
            task_item.t_type, task_item.sub_type
        );
        self.task_queue.push(task_item);

        Ok(())
    }

    async fn process_out_rabbitmsg(
        &mut self,
        channel: &Channel,
        item: RabbitmqItem,
    ) -> Result<(), lapin::Error> {
        let exchange = self.ctx.cfg.rabbitmq.log.exchange.as_str();
        let route_key = self.ctx.cfg.rabbitmq.log.route_key.as_str();
        let payload = serde_json::to_string(&item);
        if let Err(e) = payload {
            error!(
                "error, rabbitmq_service, serde_json::to_string, err: {:?}",
                e
            );
            return Ok(());
        }
        let payload = payload.unwrap();

        let expire = self.ctx.cfg.rabbitmq.log.expire * 60 * 1000; // 分钟 * 60* 1000

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
        debug!("RabbitmqService, publish_confirm, {:?}", publish_confirm);

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
