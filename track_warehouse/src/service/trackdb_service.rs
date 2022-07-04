use fy_base::api::bm_api::RecognitionApi;
use fy_base::api::upload_api::NotifyFaceQueueItem;
use fy_base::util::service::Service;
use std::sync::Arc;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;
use tracing::{debug, info};

use crate::app_ctx::AppCtx;
use crate::dao::base_model::Facetrack;

use crate::queue_item::FaceQueue;

pub struct TrackDbService {
    ctx: Arc<AppCtx>,
    face_queue: Arc<FaceQueue>,
    api: RecognitionApi,
}

impl TrackDbService {
    pub fn new(ctx: Arc<AppCtx>, face_queue: Arc<FaceQueue>) -> Self {
        let api = RecognitionApi::new(ctx.cfg.track_db.recg_url.as_str());

        TrackDbService {
            ctx,
            face_queue,
            api,
        }
    }

    async fn process_face(&self, item: NotifyFaceQueueItem) {
        debug!("TrackDbService, process_face: {}", item.uuid);
    }

    pub async fn do_run(self, mut exit_rx: Receiver<i64>) {
        loop {
            tokio::select! {
                face = self.face_queue.pop() => {
                    self.process_face(face).await;
                }
                _ = exit_rx.changed() => {
                    info!("MysqlService recv exit");
                    break;
                }
            }
        }

        info!("TrackDbService exit");
    }
}

impl Service for TrackDbService {
    fn run(self, exit_rx: Receiver<i64>) -> JoinHandle<()> {
        tokio::spawn(self.do_run(exit_rx))
    }
}
