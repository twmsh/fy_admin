use crate::app_ctx::AppCtx;
use crate::queue_item::{CarQueue, FaceQueue};
use fy_base::api::upload_api::{NotifyCarQueueItem, NotifyFaceQueueItem};
use fy_base::util::service::Service;
use std::sync::Arc;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;
use tracing::{debug, info};

pub struct MinioService {
    pub ctx: Arc<AppCtx>,
    pub face_queue: Arc<FaceQueue>,
    pub car_queue: Arc<CarQueue>,
}

impl MinioService {
    pub fn new(ctx: Arc<AppCtx>, face_queue: Arc<FaceQueue>, car_queue: Arc<CarQueue>) -> Self {
        Self {
            ctx,
            face_queue,
            car_queue,
        }
    }

    async fn process_face(&self, item: NotifyFaceQueueItem) {
        debug!("MinioService, process_face: {}", item.uuid);
    }

    async fn process_car(&self, item: NotifyCarQueueItem) {
        debug!("MinioService, process_car: {}", item.uuid);
    }

    pub async fn do_run(self, mut exit_rx: Receiver<i64>) {
        loop {
            tokio::select! {
                face = self.face_queue.pop() => {
                    self.process_face(face).await;
                }
                car = self.car_queue.pop() => {
                    self.process_car(car).await;
                }
                _ = exit_rx.changed() => {
                    info!("MinioService recv exit");
                    break;
                }
            }
        }

        info!("MinioService exit");
    }
}

impl Service for MinioService {
    fn run(self, exit_rx: Receiver<i64>) -> JoinHandle<()> {
        tokio::spawn(self.do_run(exit_rx))
    }
}
