use crate::app_ctx::AppCtx;
use crate::queue_item::{CarQueue, FaceQueue};
use fy_base::api::upload_api::{NotifyCarQueueItem, NotifyFaceQueueItem};
use fy_base::util::service::Service;
use std::sync::Arc;
use s3::Bucket;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;
use tracing::{debug, info};
use fy_base::util::minio::new_bucket;

pub struct MinioService {
    pub ctx: Arc<AppCtx>,
    pub face_in_queue: Arc<FaceQueue>,
    pub car_in_queue: Arc<CarQueue>,

    pub face_out_queue: Arc<FaceQueue>,
    pub car_out_queue: Arc<CarQueue>,

    pub facetrack_bucket: Bucket,
    pub cartrack_bucket: Bucket,

}

impl MinioService {
    pub fn new(
        ctx: Arc<AppCtx>,
        face_in_queue: Arc<FaceQueue>,
        car_in_queue: Arc<CarQueue>,
        face_out_queue: Arc<FaceQueue>,
        car_out_queue: Arc<CarQueue>,
    ) -> Self {
        let facetrack_bucket = new_bucket(
            &ctx.cfg.minio.endpoint,
            &ctx.cfg.minio.access_key,
            &ctx.cfg.minio.secret_key,
            &ctx.cfg.minio.facetrack_bucket,
        ).unwrap();

        let cartrack_bucket = new_bucket(
            &ctx.cfg.minio.endpoint,
            &ctx.cfg.minio.access_key,
            &ctx.cfg.minio.secret_key,
            &ctx.cfg.minio.cartrack_bucket,
        ).unwrap();

        Self {
            ctx,
            face_in_queue,
            car_in_queue,
            face_out_queue,
            facetrack_bucket,
            cartrack_bucket,
            car_out_queue,
        }
    }

    async fn process_face(&self, item: NotifyFaceQueueItem) {

        // 保存图片,特征值
        // 更改图片名为 minio url
        // 删除 图片buf
        // 传入到后续队列

        debug!("MinioService, process_face: {}", item.uuid);
    }

    async fn process_car(&self, item: NotifyCarQueueItem) {
        debug!("MinioService, process_car: {}", item.uuid);
    }

    pub async fn do_run(self, mut exit_rx: Receiver<i64>) {
        loop {
            tokio::select! {
                face = self.face_in_queue.pop() => {
                    self.process_face(face).await;
                }
                car = self.car_in_queue.pop() => {
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
