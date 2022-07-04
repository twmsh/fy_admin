use chrono::Local;
use fy_base::api::upload_api::{NotifyCarQueueItem, NotifyFaceQueueItem};
use fy_base::util::service::Service;
use std::sync::Arc;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

use crate::app_ctx::AppCtx;
use crate::dao::base_model::{Cartrack, Facetrack};
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
            error!("error, MysqlService, save_facetrack_to_mysql, err: {:?}", e);
        }

        self.rabbitmq_face_queue.push(item.clone());
        self.face_trackdb_queue.push(item);
    }

    async fn process_car(&self, item: NotifyCarQueueItem) {
        debug!("MysqlService, process_car: {}", item.uuid);
        if let Err(e) = self.save_cartrack_to_mysql(&item).await {
            error!("error, MysqlService, save_cartrack_to_mysql, err: {:?}", e);
        }
        self.rabbitmq_car_queue.push(item);
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
    async fn save_facetrack_to_mysql(&self, item: &NotifyFaceQueueItem) -> Result<(), AppError> {
        let facetrack = Self::from_queueitem_to_facetrack(item);

        let id = self.ctx.dao.save_facetrack(&facetrack).await?;

        debug!(
            "MysqlService, save facetrack ok, {}, {}",
            id, facetrack.uuid
        );

        Ok(())
    }

    async fn save_cartrack_to_mysql(&self, item: &NotifyCarQueueItem) -> Result<(), AppError> {
        let cartrack = Self::from_queueitem_to_cartrack(item);

        let id = self.ctx.dao.save_cartrack(&cartrack).await?;

        debug!("MysqlService, save cartrack ok, {}, {}", id, cartrack.uuid);

        Ok(())
    }

    fn from_queueitem_to_facetrack(item: &NotifyFaceQueueItem) -> Facetrack {
        let now = Local::now();

        let mut gender = 0; // 0 不确定; 1 男性; 2 ⼥性
        let mut age = 0;
        let mut glasses = 0; // 0 不戴眼镜; 1 墨镜; 2 普通眼镜

        if let Some(ref props) = item.notify.props {
            gender = props.gender as i16;
            age = props.age as i16;
            glasses = props.glasses as i16;
        }

        let img_list: Vec<String> = item
            .notify
            .faces
            .iter()
            .enumerate()
            .map(|(id, face)| format!("{}:{}", id + 1, face.quality))
            .collect();
        let img_ids = img_list.join(",");

        // feature_file字段不为空
        let mut feature_list = vec![];
        for (id, face) in item.notify.faces.iter().enumerate() {
            if let Some(ref v) = face.feature_file {
                if !v.is_empty() {
                    feature_list.push(format!("{}:{}", id + 1, face.quality));
                }
            }
        }
        let feature_ids = feature_list.join(",");

        Facetrack {
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
            create_time: now,
        }
    }

    fn from_queueitem_to_cartrack(item: &NotifyCarQueueItem) -> Cartrack {
        let now = Local::now();

        let img_list: Vec<String> = item
            .notify
            .vehicles
            .iter()
            .enumerate()
            .map(|(id, _face)| format!("{}:{}", id + 1, 1.0))
            .collect();
        let img_ids = img_list.join(",");

        let mut plate_judged = 0;
        if item.notify.has_plate_info() {
            plate_judged = 1;
        };

        let mut vehicle_judged = 0;
        if item.notify.has_props_info() {
            vehicle_judged = 1;
        }

        let (plate_content, plate_type) = item.notify.get_plate_tuple();
        let (
            move_direct,
            car_direct,
            car_color,
            car_brand,
            car_top_series,
            car_series,
            car_top_type,
            car_mid_type,
        ) = item.notify.get_props_tuple();

        let plate_confidence = item.notify.get_plate_confidence().map(|x| x as f32);

        Cartrack {
            id: 0,
            uuid: item.uuid.clone(),
            camera_uuid: item.notify.source.clone(),
            img_ids,
            plate_judged,
            vehicle_judged,
            move_direct: move_direct as i16,
            car_direct,
            plate_content,
            plate_confidence,
            plate_type,
            car_color,
            car_brand,
            car_top_series,
            car_series,
            car_top_type,
            car_mid_type,
            capture_time: item.ts,
            create_time: now,
        }
    }
}
