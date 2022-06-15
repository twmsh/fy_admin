use std::sync::Arc;

use crate::app_ctx::AppCtx;
use chrono::Local;
use fy_base::util::service::Service;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;
use tracing::{error, info};

pub struct CleanService {
    pub ctx: Arc<AppCtx>,
}

impl CleanService {
    pub fn new(ctx: Arc<AppCtx>) -> Self {
        Self { ctx }
    }

    async fn do_clean(&self) {
        let ttl = self.ctx.cfg.clean.ttl_day as i64; // ttl days
        let now = Local::now();
        let before = match now.checked_sub_signed(chrono::Duration::days(ttl)) {
            None => {
                error!("error, CleanService, checked_sub_signed, overflow");
                return;
            }
            Some(v) => v,
        };

        match self.ctx.dao.clean_boxlog(before).await {
            Ok(v) => {
                info!("CleanService, clean_boxlog, delete {} rows", v);
            }
            Err(e) => {
                error!("error, CleanService, clean_boxlog, err: {:?}", e);
            }
        }
    }

    async fn do_run(self, mut exit_rx: Receiver<i64>) {
        let interval = self.ctx.cfg.clean.interval_hour; // 小时
        let interval = chrono::Duration::hours(interval as i64).to_std().unwrap();

        let mut sleep_a_while = tokio::time::interval(interval);
        loop {
            tokio::select! {
                _ = sleep_a_while.tick() => {
                    info!("CleanService, do clean ...");
                    self.do_clean().await;
                }
                _ = exit_rx.changed() => {
                    info!("CleanService, recv signal, will exit");
                    break;
                }
            }
        }
        info!("CleanService exit.");
    }
}

impl Service for CleanService {
    fn run(self, exit_rx: Receiver<i64>) -> JoinHandle<()> {
        let this = self;
        tokio::spawn(this.do_run(exit_rx))
    }
}
