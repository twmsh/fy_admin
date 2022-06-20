use crate::error::AppError;
use crate::model::queue_item::RabbitmqItem;
use crate::model::{StatusCamera, StatusDb, StatusPayload};
use chrono::Local;
use fy_base::api::bm_api::{AnalysisApi, RecognitionApi};
use fy_base::sync::rabbitmq_type::{BOXLOGMESSAGE_LEVEL_INFO, BOXLOGMESSAGE_TYPE_STATUS};
use log::debug;
use tracing::{error, info};

#[cfg(unix)]
pub fn reboot_box() {
    use std::process::Command;
    let status = Command::new("sh").arg("-c").arg("sudo reboot").status();
    info!("WorkerService, reboot_box: {:?}", status);
}

#[cfg(windows)]
pub fn reboot_box() {
    info!("WorkerService, reboot_box, non_op");
}

//-----------------------------
pub async fn delete_all_cameras(api_client: &AnalysisApi) -> Result<u64, AppError> {
    //
    let res = api_client.get_sources().await?;
    if res.code != 0 {
        return Err(AppError::new(&format!("get_sources, code:{}", res.code)));
    }

    let uuids = match res.sources {
        None => vec![],
        Some(v) => v.iter().map(|x| x.id.clone()).collect(),
    };

    // 删除camera
    for uuid in uuids.iter() {
        let res = api_client.delete_source(uuid.clone()).await?;
        if res.code != 0 {
            return Err(AppError::new(&format!(
                "delete_source({}), code:{}",
                uuid, res.code
            )));
        }
        debug!("WorkerService, deleted camera: {}", uuid);
    }

    Ok(uuids.len() as u64)
}

pub async fn delete_all_dbs(api_client: &RecognitionApi) -> Result<u64, AppError> {
    //
    let res = api_client.get_dbs().await?;
    if res.code != 0 {
        return Err(AppError::new(&format!("get_dbs, code:{}", res.code)));
    }

    let uuids = match res.dbs {
        None => vec![],
        Some(v) => v.iter().map(|x| x.clone()).collect(),
    };

    // 删除
    for uuid in uuids.iter() {
        let res = api_client.delete_db(uuid.clone()).await?;
        if res.code != 0 {
            return Err(AppError::new(&format!(
                "delete_db({}), code:{}",
                uuid, res.code
            )));
        }
    }

    Ok(uuids.len() as u64)
}

pub async fn get_status_payload(
    ref_id: u64,
    ana_api: &AnalysisApi,
    rec_api: &RecognitionApi,
) -> Result<StatusPayload, AppError> {
    // 获取所有摄像头
    let res = ana_api.get_sources().await?;
    if res.code != 0 {
        return Err(AppError::new(&format!("get_sources, code:{}", res.code)));
    }

    let uuids = match res.sources {
        None => vec![],
        Some(v) => v.iter().map(|x| x.id.clone()).collect(),
    };

    // camera详情
    let mut camera_list = vec![];
    for uuid in uuids.iter() {
        let res = ana_api.get_source_info(uuid.clone()).await?;
        if res.code != 0 {
            return Err(AppError::new(&format!(
                "get_source_info({}), code:{}",
                uuid, res.code
            )));
        }
        debug!("WorkerService, get_source_info: {}", uuid);

        if let Some(ref v) = res.config {
            let mut c_type = 0;
            if v.enable_face {
                c_type += 1;
            }
            if v.enable_vehicle {
                c_type += 2;
            }
            let camera = StatusCamera {
                uuid: uuid.clone(),
                url: res
                    .url
                    .as_ref()
                    .map_or_else(|| "".to_string(), |x| x.clone()),
                c_type,
            };
            camera_list.push(camera);
        }
    }

    // dbs
    let mut db_list = vec![];
    let res = rec_api.get_dbs().await?;
    if res.code != 0 {
        return Err(AppError::new(&format!("get_dbs, code:{}", res.code)));
    }

    let uuids = match res.dbs {
        None => vec![],
        Some(v) => v.iter().map(|x| x.clone()).collect(),
    };

    for uuid in uuids.iter() {
        let res = rec_api.get_db_info(uuid.clone()).await?;
        if res.code != 0 {
            return Err(AppError::new(&format!(
                "get_db_info({}), code:{}",
                uuid, res.code
            )));
        }
        debug!("WorkerService, get_db_info: {}", uuid);

        let mut status_db = StatusDb {
            uuid: uuid.clone(),
            capacity: 0,
            used: 0,
        };
        if let Some(v) = res.volume {
            status_db.capacity = v as u64;
        }
        if let Some(v) = res.usage {
            status_db.used = v as u64;
        }
        db_list.push(status_db);
    }

    Ok(StatusPayload {
        ref_id,
        cameras: camera_list,
        dbs: db_list,
    })
}

//-----------------------------
pub fn build_rabbitmqitem_from_status(
    hw_id: &str,
    ips: &str,
    status_payload: &StatusPayload,
) -> RabbitmqItem {
    let payload = match serde_json::to_string(status_payload) {
        Ok(v) => v,
        Err(e) => {
            error!("error, build_rabbitmqitem_from_status, err: {:?}", e);
            "".to_string()
        }
    };

    RabbitmqItem {
        hwid: hw_id.to_string(),
        ips: ips.to_string(),
        c_type: BOXLOGMESSAGE_TYPE_STATUS.to_string(),
        level: BOXLOGMESSAGE_LEVEL_INFO,
        payload,
        ts: Local::now(),
    }
}
