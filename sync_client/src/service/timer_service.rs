use crate::app_ctx::AppCtx;
use crate::model::queue_item::{TaskItem, TaskItemType};
use chrono::Local;
use deadqueue::unlimited::Queue;
use fy_base::util::service::Service;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;
use tracing::info;

pub struct TimerService {
    pub ctx: Arc<AppCtx>,
    pub queue: Arc<Queue<TaskItem>>,
}

impl TimerService {
    pub fn new(ctx: Arc<AppCtx>, queue: Arc<Queue<TaskItem>>) -> Self {
        Self { ctx, queue }
    }

    fn push_hb_task(&self) {
        let now = Local::now();
        self.queue.push(TaskItem {
            id: now.timestamp_millis() as u64,
            t_type: TaskItemType::HeartBeat,
            ts: now,
            sub_type: 0,
            payload: "".to_string(),
        });
        info!("TimerService, push_hb_task");
    }

    fn push_sync_task(&self) {
        let now = Local::now();
        self.queue.push(TaskItem {
            id: now.timestamp_millis() as u64,
            t_type: TaskItemType::SyncTimer,
            ts: now,
            sub_type: 0,
            payload: "".to_string(),
        });
        info!("TimerService, push_sync_task");
    }

    pub async fn do_run(self, mut exit_rx: Receiver<i64>) {
        let hb_interval = self.ctx.cfg.sync.heartbeat; // 分钟
        let sync_interval = self.ctx.cfg.sync.sync_ttl; // 分钟
        let mut hb_timer = tokio::time::interval(Duration::from_secs(hb_interval * 60));
        let mut sync_timer = tokio::time::interval(Duration::from_secs(sync_interval * 60));

        loop {
            tokio::select! {
                _ = hb_timer.tick() => {
                    info!("TimerService, hb timer, tick ...");
                    self.push_hb_task();

                }
                _ = sync_timer.tick() => {
                    info!("TimerService, sync timer, tick ...");
                    self.push_sync_task();
                }
                _ = exit_rx.changed() => {
                    info!("TimerService, recv signal, will exit");
                    break;
                }
            }
        }

        info!("TimerService exit.");
    }
}

impl Service for TimerService {
    fn run(self, exit_rx: Receiver<i64>) -> JoinHandle<()> {
        let this = self;
        tokio::spawn(this.do_run(exit_rx))
    }
}
