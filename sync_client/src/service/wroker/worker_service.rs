use crate::app_ctx::AppCtx;
use crate::error::AppError;
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
    build_rabbitmqitem_from_status, delete_all_cameras, delete_all_dbs, do_sync_camera, do_sync_db,
    do_sync_person, get_status_payload, reboot_box,
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

    async fn process_task(&self, item: TaskItem, exit_rx: Receiver<i64>) {
        debug!("WorkerService, process_task, {:?}", item);

        match item.t_type {
            TaskItemType::SyncTimer => {
                self.process_task_sync(item, exit_rx).await;
            }
            TaskItemType::HeartBeat => {
                self.process_task_status(item).await;
            }
            TaskItemType::ServerCmd => {
                if item.sub_type == 0 {
                    // sync
                    self.process_task_sync(item, exit_rx).await;
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

    async fn process_task_sync(&self, _item: TaskItem, exit_rx: Receiver<i64>) {
        // -> camera -> db -> person
        // ?????????camera???camera????????????,person?????????????????????????????????
        // camera??????????????????????????????????????????????????? db???????????????person

        let exited = match self.sync_camera(&exit_rx).await {
            Ok(v) => v,
            Err(e) => {
                error!("error, Worker_service, sync_camera, err: {}", e);
                false
            }
        };
        self.ctx.save_sync_log();
        if exited {
            // ?????? sync_log
            return;
        }

        // ?????? sync db
        let exited = match self.sync_db(&exit_rx).await {
            Ok(v) => v,
            Err(e) => {
                error!("error, Worker_service, sync_db, err: {}", e);
                false
            }
        };
        self.ctx.save_sync_log();
        if exited {
            // ?????? sync_log
            return;
        }

        // ?????? sync person
        let _exited = match self.sync_person(&exit_rx).await {
            Ok(v) => v,
            Err(e) => {
                error!("error, Worker_service, sync_person, err: {}", e);
                false
            }
        };
        self.ctx.save_sync_log();
    }

    // bool ??????????????????????????????
    async fn sync_camera(&self, _exit_rx: &Receiver<i64>) -> Result<bool, AppError> {
        do_sync_camera(self.ctx.clone()).await
    }

    async fn sync_db(&self, _exit_rx: &Receiver<i64>) -> Result<bool, AppError> {
        do_sync_db(self.ctx.clone()).await
    }

    async fn sync_person(&self, _exit_rx: &Receiver<i64>) -> Result<bool, AppError> {
        do_sync_person(self.ctx.clone()).await
    }

    async fn process_task_status(&self, item: TaskItem) {
        // ?????????????????????????????????db??????????????????????????????rabbitmq_queue???s

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
            // ????????????camera
            let deleted = match delete_all_cameras(&self.ctx.ana_api).await {
                Ok(v) => v,
                Err(e) => {
                    error!("error, WorkerService, delete_all_cameras, err: {:?}", e);
                    0
                }
            };
            info!("WorkerService, delete_all_cameras, deleted: {}", deleted);

            //
            self.ctx.reset_synclog_for_camera();
        }

        if reset_payload.db {
            // ????????????db
            let deleted = match delete_all_dbs(&self.ctx.recg_api).await {
                Ok(v) => v,
                Err(e) => {
                    error!("error, WorkerService, delete_all_dbs, err: {:?}", e);
                    0
                }
            };
            info!("WorkerService, delete_all_dbs, deleted: {}", deleted);

            //
            self.ctx.reset_synclog_for_db();
        }

        // ?????? sync_log
        self.ctx.save_sync_log();
    }

    async fn process_task_reboot(&self, _item: TaskItem) {
        // ??????
        debug!("WorkerService, process_task_reboot");
        reboot_box();
    }

    pub async fn do_run(self, mut exit_rx: Receiver<i64>) {
        loop {
            tokio::select! {
                item = self.task_queue.pop() => {
                    self.process_task(item,exit_rx.clone()).await;
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
