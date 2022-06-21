use deadqueue::unlimited::Queue;
use tracing::{debug, error, info};
use std::sync::Arc;

use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle as TkJoinHandle;

use crate::app_ctx::AppCtx;
use crate::queue_item::QI;
use crate::uplink::uplink_api::UplinkApi;

use fy_base::util::service::Service;

pub struct UplinkService {
    ctx: Arc<AppCtx>,
    queue: Arc<Queue<QI>>,
    api: UplinkApi,
}

impl UplinkService {
    pub fn new(ctx: Arc<AppCtx>, queue: Arc<Queue<QI>>) -> Self {
        UplinkService {
            ctx,
            queue,
            api: Default::default(),
        }
    }
    async fn process_item(&mut self, item: QI) {
        let upload_url = self.ctx.cfg.up_link.server.as_str();
        let uuid = item.get_id();

        match item {
            QI::FT(ft) => {
                let rst = self.api.upload_face(upload_url, *ft).await;
                match rst {
                    Ok(_) => {
                        debug!("UplinkService, upload ft: {}, ok", uuid);
                    }
                    Err(e) => {
                        error!("error, UplinkService, upload: {} fail, err:{:?}", uuid, e);
                    }
                }
            }
            QI::CT(ct) => {
                let rst = self.api.upload_car(upload_url, *ct).await;
                match rst {
                    Ok(_) => {
                        debug!("UplinkService, upload car: {}, ok", uuid);
                    }
                    Err(e) => {
                        error!("error, UplinkService, upload: {} fail, err:{:?}", uuid, e);
                    }
                }
            }
        }
    }


}

impl Service for UplinkService {
    fn run(self, rx: Receiver<i64>) -> TkJoinHandle<()> {
        let mut svc = self;
        let mut exit_rx = rx;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = exit_rx.changed() => {
                        info!("UplinkService recv exit");
                        break;
                    }
                    items = svc.queue.pop() => {
                        svc.process_item(items).await;
                    }
                }
            }
            info!("UplinkService exit");
        })
    }
}
