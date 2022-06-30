use crate::app_ctx::AppCtx;
use crate::queue_item::{CarQueue, FaceQueue};
use fy_base::api::upload_api::{NotifyCarQueueItem, NotifyFaceQueueItem};
use fy_base::util::service::Service;
use std::sync::Arc;
use bytes::Bytes;
use log::error;
use s3::Bucket;

use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;
use tracing::{debug, info};
use fy_base::util::minio;
use fy_base::util::minio::new_bucket;
use crate::error::AppError;

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

    async fn process_face(&self, mut item: NotifyFaceQueueItem) {

        // 保存图片,特征值
        // 更改图片名为 minio url
        // 删除 图片buf
        // 传入到后续队列

        debug!("MinioService, process_face: {}", item.uuid);
        let saved = self.save_facetrack_to_minio(&mut item).await;
        if let Err(e) = saved {
            error!("error, MinioService, save_facetrack_to_minio:{}, err: {:?}", item.uuid,e);
        } else {
            debug!("MinioService, save_facetrack_to_minio, ok, {}",item.uuid);
        }

        debug!("MinioService, face put to next queue, {}",item.uuid);
        self.face_out_queue.push(item);
    }

    async fn process_car(&self, mut item: NotifyCarQueueItem) {
        debug!("MinioService, process_car: {}", item.uuid);
        let saved = self.save_cartrack_to_minio(&mut item).await;
        if let Err(e) = saved {
            error!("error, MinioService, save_cartrack_to_minio:{}, err: {:?}",item.uuid, e);
        } else {
            debug!("MinioService, save_cartrack_to_minio, ok, {}",item.uuid);
        }

        debug!("MinioService, car put to next queue, {}",item.uuid);
        self.car_out_queue.push(item);
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

//--------------------------------------------

impl MinioService {
    async fn save_minio_from_bytes(
        bucket: &Bucket,
        path: &str,
        content: &mut Bytes,
        clean: bool,
    ) -> Result<(), AppError> {
        let rst = minio::save_to_minio(bucket, path, content).await;

        if clean && !content.is_empty() {
            *content = Bytes::new();
        }

        let rst = match rst {
            Ok(v) => v,
            Err(e) => {
                return Err(e.into());
            }
        };

        if rst.1 != 200 {
            return Err(AppError::new(&format!("minio put_object return: {},{}", rst.1, rst.0)));
        }
        Ok(())
    }

    // 保存只minio，修改 item的 file name
    async fn save_facetrack_to_minio(&self, item: &mut NotifyFaceQueueItem) -> Result<(), AppError> {
        let uuid = item.uuid.as_str();
        let ts = item.ts;

        // 先将 image_file置空，后赋值为minio path，然后清掉 Bytes

        // 保存背景图
        item.notify.background.image_file = "".into();
        let path = minio::get_facetrack_relate_bg_path(uuid, ts);
        let _saved = Self::save_minio_from_bytes(
            &self.facetrack_bucket,
            &path,
            &mut item.notify.background.image_buf,
            true,
        ).await?;
        item.notify.background.image_file = path;

        for (face_id, face) in item.notify.faces.iter_mut().enumerate() {

            // 小图
            face.aligned_file = "".into();
            let path = minio::get_facetrack_relate_small_path(uuid, ts, face_id as u8 + 1);
            let _saved = Self::save_minio_from_bytes(
                &self.facetrack_bucket,
                &path,
                &mut face.aligned_buf,
                true,
            ).await?;
            face.aligned_file = path;

            // 大图
            face.display_file = "".into();
            let path = minio::get_facetrack_relate_large_path(uuid, ts, face_id as u8 + 1);
            let _saved = Self::save_minio_from_bytes(
                &self.facetrack_bucket,
                &path,
                &mut face.display_buf,
                true,
            ).await?;
            face.display_file = path;

            // 特征值
            if face.feature_file.is_some() && face.feature_buf.is_some() {
                let feature_file = face.feature_file.as_mut().unwrap();
                let feature_buf = face.feature_buf.as_mut().unwrap();

                *feature_file = "".into();
                let path = minio::get_facetrack_relate_fea_path(uuid, ts, face_id as u8 + 1);
                let _saved = Self::save_minio_from_bytes(
                    &self.facetrack_bucket,
                    &path,
                    feature_buf,
                    false,
                ).await?;
                *feature_file = path;
            }
        }


        Ok(())
    }

    async fn save_cartrack_to_minio(&self, item: &mut NotifyCarQueueItem) -> Result<(), AppError> {
        let uuid = item.uuid.as_str();
        let ts = item.ts;

        // 先将 image_file置空，后赋值为minio path，然后清掉 Bytes

        // 保存背景图
        item.notify.background.image_file = "".into();
        let path = minio::get_cartrack_relate_bg_path(uuid, ts);
        let _saved = Self::save_minio_from_bytes(
            &self.cartrack_bucket,
            &path,
            &mut item.notify.background.image_buf,
            true,
        ).await?;
        item.notify.background.image_file = path;

        // 车辆图
        for (car_id, car) in item.notify.vehicles.iter_mut().enumerate() {
            car.image_file = "".into();
            let path = minio::get_cartrack_relate_car_path(uuid, ts, car_id as u8 + 1);
            let _saved = Self::save_minio_from_bytes(
                &self.cartrack_bucket,
                &path,
                &mut car.img_buf,
                true,
            ).await?;
            car.image_file = path;
        }

        // 车牌图
        if item.notify.has_plate_info() {
            if let Some(ref mut plate_info) =  item.notify.plate_info {
                // 车牌图
                plate_info.image_file = Some("".into());
                let path = minio::get_cartrack_relate_plate_path(uuid, ts);
                let _saved = Self::save_minio_from_bytes(
                    &self.cartrack_bucket,
                    &path,
                    &mut plate_info.img_buf,
                    true,
                ).await?;
                plate_info.image_file = Some(path);

                // 车牌二值图
                plate_info.binary_file = Some("".into());
                let path = minio::get_cartrack_relate_binary_path(uuid, ts);
                let _saved = Self::save_minio_from_bytes(
                    &self.cartrack_bucket,
                    &path,
                    &mut plate_info.binary_buf,
                    true,
                ).await?;
                plate_info.binary_file = Some(path);

            }
        }

        Ok(())
    }

}