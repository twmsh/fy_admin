use std::sync::Arc;
use chrono::Local;
use fy_base::util::service::Service;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};
use fy_base::api::upload_api::{NotifyCarQueueItem, NotifyFaceQueueItem};

use crate::app_ctx::AppCtx;
use crate::dao::base_model::Facetrack;
use crate::error::AppError;
use crate::queue_item::{CarQueue, FaceQueue};

pub struct MysqlService {
    pub ctx: Arc<AppCtx>,
    pub face_in_queue: Arc<FaceQueue>,
    pub car_in_queue: Arc<CarQueue>,

    pub face_trackdb_queue: Arc<FaceQueue>,
    pub rabbitmq_face_queue: Arc<FaceQueue>,
    pub rabbitmq_car_queue: Arc<CarQueue>,

}

impl MysqlService {
    pub fn new(
        ctx: Arc<AppCtx>,
        face_in_queue: Arc<FaceQueue>,
        car_in_queue: Arc<CarQueue>,
        face_trackdb_queue: Arc<FaceQueue>,
        rabbitmq_face_queue: Arc<FaceQueue>,
        rabbitmq_car_queue: Arc<CarQueue>,
    ) -> Self {
        MysqlService {
            ctx,
            face_in_queue,
            car_in_queue,
            face_trackdb_queue,
            rabbitmq_face_queue,
            rabbitmq_car_queue,
        }
    }

    async fn process_face(&self, item: NotifyFaceQueueItem) {
        // 保存到数据库中，如果要倒查，放入到路人库队列中
        // 放入rabbitmq相关队列中.

        debug!("MysqlService, process_face: {}", item.uuid);

        if let Err(e) = self.save_facetrack_to_mysql(&item).await {
            error!("error, MysqlService, save_facetrack_to_mysql, err: {:?}",e);
        }


    }

    async fn process_car(&self,  item: NotifyCarQueueItem) {
        debug!("MysqlService, process_car: {}", item.uuid);
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
                    info!("MysqlService recv exit");
                    break;
                }
            }
        }

        info!("MysqlService exit");
    }
}

impl Service for MysqlService {
    fn run(self, exit_rx: Receiver<i64>) -> JoinHandle<()> {
        tokio::spawn(self.do_run(exit_rx))
    }
}


impl MysqlService {
    async fn save_facetrack_to_mysql(&self, item: & NotifyFaceQueueItem) -> Result<(), AppError> {

        let facetrack = Self::from_queueitem_to_facetrack(item);

        let id = self.ctx.dao.save_facetrack(&facetrack).await?;

        debug!("MysqlService, save facetrack ok, {}, {}", id, facetrack.uuid);

        Ok(())
    }

    fn from_queueitem_to_facetrack(item:& NotifyFaceQueueItem)-> Facetrack {

        let now = Local::now();

        let mut gender = 0; // 0 不确定; 1 男性; 2 ⼥性
        let mut age = 0;
        let mut glasses = 0; // 0 不戴眼镜; 1 墨镜; 2 普通眼镜

        if let Some(ref props) = item.notify.props {
            gender = props.gender as i16;
            age = props.age as i16;
            glasses = props.glasses as i16;
        }

        let img_list:Vec<String> = item.notify.faces.iter().enumerate()
            .map(|(id,face)|{
                format!("{}:{}",id+1,face.quality)
            }).collect();
        let img_ids = img_list.join(",");

        // feature_file字段不为空
        let mut feature_list =vec![];
        for (id,face) in item.notify.faces.iter().enumerate() {
             if let Some(ref v) = face.feature_file {
                if !v.is_empty() {
                    feature_list.push(format!("{}:{}",id+1,face.quality));
                }
             }
        }
        let feature_ids = feature_list.join(",");

         Facetrack{
            id: 0,
            uuid: item.uuid.clone(),
            camera_uuid: item.notify.source.clone(),
            img_ids,
            feature_ids,
            gender,
            age,
            glasses,
            most_persons: None,
            capture_time: item.ts,
            create_time: now
        }
    }

}