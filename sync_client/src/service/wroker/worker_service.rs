use std::sync::Arc;
use deadqueue::unlimited::Queue;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;
use tracing::{debug, info};
use fy_base::util::service::Service;
use crate::app_ctx::AppCtx;
use crate::model::queue_item::{RabbitmqItem, TaskItem};

pub struct WorkerService {
    pub ctx: Arc<AppCtx>,
    pub task_queue: Arc<Queue<TaskItem>>,
    pub rabbitmq_queue: Arc<Queue<RabbitmqItem>>,
}

impl WorkerService {
    pub fn new(
        ctx: Arc<AppCtx>,
        task_queue: Arc<Queue<TaskItem>>,
        rabbitmq_queue: Arc<Queue<RabbitmqItem>>,
    ) -> Self {
        Self {
            ctx,
            task_queue,
            rabbitmq_queue,
        }
    }

    async fn process_task(&self, item: TaskItem) {
        debug!("WorkerService, process_task, {:?}",item);
    }


    pub async fn do_run(self, mut exit_rx: Receiver<i64>) {
        loop {
            tokio::select! {
                item = self.task_queue.pop() => {
                    self.process_task(item).await;
                }
                _ = exit_rx.changed() => {
                    info!("WorkerService, recv signal, will exit");
                    break;
                }
            }
        }
        info!("WorkerService exit.");
    }
}


impl Service for WorkerService {
    fn run(self, exit_rx: Receiver<i64>) -> JoinHandle<()> {
        let this = self;
        tokio::spawn(this.do_run(exit_rx))
    }
}