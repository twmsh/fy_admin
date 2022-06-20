use crate::app_ctx::AppCtx;
use crate::model::queue_item::{RabbitmqItem, TaskItem, TaskItemType};
use deadqueue::unlimited::Queue;
use fy_base::util::ip::get_local_ips;
use fy_base::util::service::Service;
use std::sync::Arc;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

use crate::model::ResetPayload;
use crate::service::wroker::work::{
    build_rabbitmqitem_from_status, delete_all_cameras, delete_all_dbs, get_status_payload,
    reboot_box,
};

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
        debug!("WorkerService, process_task, {:?}", item);

        match item.t_type {
            TaskItemType::SyncTimer => {
                self.process_task_sync(item).await;
            }
            TaskItemType::HeartBeat => {
                self.process_task_status(item).await;
            }
            TaskItemType::ServerCmd => {
                if item.sub_type == 0 {
                    // sync
                    self.process_task_sync(item).await;
                } else if item.sub_type == 1 {
                    // reset
                    self.process_task_reset(item).await;
                } else if item.sub_type == 2 {
                    // reboot
                    self.process_task_reboot(item).await;
                } else {
                    error!("error, Worker_service, unknown sub_type: {}", item.sub_type);
                }
            }
        }
    }

    async fn process_task_sync(&self, _item: TaskItem) {
        // -> camera -> db -> person
        // 先处理camera，camera数量较少,person数据量最多，最后处理。
        // camera处理完，无论处理成功与否，继续处理 db，最后处理person



    }

    async fn process_task_status(&self, item: TaskItem) {
        // 获取小盒子上，摄像头和db的数量情况，然后放到rabbitmq_queue中s

        let status_payload =
            match get_status_payload(item.id, &self.ctx.ana_api, &self.ctx.recg_api).await {
                Ok(v) => v,
                Err(e) => {
                    error!("error, WorkerService, process_task_status, {:?}", e);
                    return;
                }
            };

        let ips = get_local_ips().join(",");

        let rabbitmq_item = build_rabbitmqitem_from_status(&self.ctx.hw_id, &ips, &status_payload);
        debug!("WorkerService, push to rabbitmq_queue, {:?}", rabbitmq_item);

        self.rabbitmq_queue.push(rabbitmq_item);
    }

    async fn process_task_reset(&self, item: TaskItem) {
        //
        let reset_payload: ResetPayload = match serde_json::from_reader(item.payload.as_bytes()) {
            Ok(v) => v,
            Err(e) => {
                error!("error, WorkerService, process_task_reset, {:?}", e);
                return;
            }
        };

        if reset_payload.camera {
            // 清除所有camera
            let deleted = match delete_all_cameras(&self.ctx.ana_api).await {
                Ok(v) => v,
                Err(e) => {
                    error!("error, WorkerService, delete_all_cameras, err: {:?}", e);
                    0
                }
            };
            info!("WorkerService, delete_all_cameras, deleted: {}", deleted);
        }

        if reset_payload.db {
            // 清除所有db
            let deleted = match delete_all_dbs(&self.ctx.recg_api).await {
                Ok(v) => v,
                Err(e) => {
                    error!("error, WorkerService, delete_all_dbs, err: {:?}", e);
                    0
                }
            };
            info!("WorkerService, delete_all_dbs, deleted: {}", deleted);
        }
    }

    async fn process_task_reboot(&self, _item: TaskItem) {
        // 重启
        debug!("WorkerService, process_task_reboot");
        reboot_box();
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
