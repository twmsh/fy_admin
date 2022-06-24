use crate::error::AppError;
use crate::model::queue_item::RabbitmqItem;
use crate::model::{StatusCamera, StatusDb, StatusPayload};
use chrono::Local;
use std::sync::Arc;
use std::time::Duration;

use fy_base::api::bm_api::{AnalysisApi, ApiFeatureQuality, CreateSourceReqConfig, RecognitionApi};
use fy_base::sync::rabbitmq_type::{BOXLOGMESSAGE_LEVEL_INFO, BOXLOGMESSAGE_TYPE_STATUS};
use log::debug;

use fy_base::api::sync_api::{Camera, Db, Person, SYNC_OP_DEL};
use tracing::{error, info};

use crate::app_ctx::AppCtx;
use fy_base::util::utils;

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
        Some(v) => v.clone(),
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
        Some(v) => v.clone(),
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

//-----------------------------------------------------------------------------
pub async fn do_sync_camera(ctx: Arc<AppCtx>) -> Result<bool, AppError> {
    let max_loop = 100;
    let mut loop_count = 0;

    let url = ctx.cfg.sync.server.camera_sync.as_str();
    let hw_id = ctx.hw_id.as_str();
    let api = &ctx.sync_api;

    loop {
        let sync_log = ctx.get_sync_log();
        let last_ts = sync_log.camera.last_ts;
        let last_ts = last_ts.format(utils::DATETIME_FMT_LONG).to_string();

        let res = api.fetch_camera_updated(url, &last_ts, hw_id).await?;
        if res.status != 0 {
            return Err(AppError::new(&format!(
                "fetch_camera_updated, return status: {}",
                res.status
            )));
        }

        // 检查退出信号
        if ctx.is_exit() {
            return Ok(true);
        }

        if res.is_empty() {
            // 没有更新数据，退出
            return Ok(false);
        }

        let res_data = res.data.unwrap();
        let exited = do_sync_camera_batch(ctx.clone(), res_data).await?;
        if exited {
            return Ok(true);
        }

        loop_count += 1;
        // 循环次数过多，退出
        if loop_count >= max_loop {
            break;
        }

        // 休眠一会
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    Ok(false)
}

pub async fn do_sync_camera_batch(ctx: Arc<AppCtx>, list: Vec<Camera>) -> Result<bool, AppError> {
    for camera in list.iter() {

        debug!("process sync_camera, {}, {}",camera.uuid,camera.op);

        if camera.op == SYNC_OP_DEL {
            // 删除
            let deled = ctx.ana_api.delete_source(camera.uuid.clone()).await?;
            debug!(
                "WorkerService, delete camera:{}, return: {:?}",
                camera.uuid, deled
            );
        } else {
            //新增或修改
            let (grab_url, config) = match camera.detail {
                None => {
                    return Err(AppError::new(&format!(
                        "camera:{} detail is none",
                        camera.uuid
                    )));
                }
                Some(ref v) => (v.url.clone(), v.config.clone()),
            };

            let mut camera_config =
                serde_json::from_reader::<_, CreateSourceReqConfig>(config.as_bytes())?;

            // 先删除
            let deled = ctx.ana_api.delete_source(camera.uuid.clone()).await?;
            debug!(
                "WorkerService, for modify, delete camera:{}, return: {:?}",
                camera.uuid, deled
            );

            // 再新增
            // 如果 配置文件中有 摄像头推送地址，则用该地址，覆盖
            let upload_url = match ctx.cfg.sync.camera_upload {
                None => camera_config.upload_url.clone(),
                Some(ref v) => v.clone(),
            };

            match camera.c_type {
                1 => {
                    camera_config.enable_face = true;
                }
                2 => {
                    camera_config.enable_vehicle = true;
                }
                3 => {
                    camera_config.enable_face = true;
                    camera_config.enable_vehicle = true;
                }
                _ => {
                    return Err(AppError::new(&format!(
                        "unkown camera type:{}",
                        camera.c_type
                    )));
                }
            }

            let res = ctx
                .ana_api
                .create_source(
                    Some(camera.uuid.clone()),
                    grab_url,
                    upload_url,
                    camera_config,
                )
                .await?;

            if res.code != 0 {
                return Err(AppError::new(&format!(
                    "create_source:{}, return code:{}, msg:{}",
                    camera.uuid, res.code, res.msg
                )));
            }
        }

        // 更新 last_update_ts
        ctx.update_synclog_for_camera(&camera.id, camera.last_update);

        // 检查退出
        if ctx.is_exit() {
            return Ok(true);
        }
    }
    Ok(false)
}

//-----------------------------------------------------------------------------
pub async fn do_sync_db(ctx: Arc<AppCtx>) -> Result<bool, AppError> {
    let max_loop = 100;
    let mut loop_count = 0;

    let url = ctx.cfg.sync.server.db_sync.as_str();
    let hw_id = ctx.hw_id.as_str();
    let api = &ctx.sync_api;

    loop {
        let sync_log = ctx.get_sync_log();
        let last_ts = sync_log.db.last_ts;
        let last_ts = last_ts.format(utils::DATETIME_FMT_LONG).to_string();

        let res = api.fetch_db_updated(url, &last_ts, hw_id).await?;
        if res.status != 0 {
            return Err(AppError::new(&format!(
                "fetch_db_updated, return status: {}",
                res.status
            )));
        }

        // 检查退出信号
        if ctx.is_exit() {
            return Ok(true);
        }

        if res.is_empty() {
            // 没有更新数据，退出
            return Ok(false);
        }

        let res_data = res.data.unwrap();
        let exited = do_sync_db_batch(ctx.clone(), res_data).await?;
        if exited {
            return Ok(true);
        }

        loop_count += 1;
        // 循环次数过多，退出
        if loop_count >= max_loop {
            break;
        }

        // 休眠一会
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    Ok(false)
}

// db的处理，只支持删除和新增，
// 修改不支持
pub async fn do_sync_db_batch(ctx: Arc<AppCtx>, list: Vec<Db>) -> Result<bool, AppError> {
    for db in list.iter() {

        debug!("process sync_db, {}, {}",db.uuid,db.op);

        if db.op == SYNC_OP_DEL {
            // 删除
            let deled = ctx.recg_api.delete_db(db.uuid.clone()).await?;
            info!("WorkerService, delete db:{}, return: {:?}", db.uuid, deled);
        } else {
            //新增

            // 先检查 db存在与否？ 无论capacity是否相同
            let db_old = ctx.recg_api.get_db_info(db.uuid.clone()).await?;
            if db_old.code == 0 {
                // 已经存在，忽略之
                info!("WorkerService, create db:{}, db is exsit, skip ", db.uuid);
            } else {
                // 不存在，新建
                let res = ctx
                    .recg_api
                    .create_db(Some(db.uuid.clone()), db.capacity as i64)
                    .await?;

                if res.code != 0 {
                    return Err(AppError::new(&format!(
                        "create_db:{}, return code:{}, msg:{}",
                        db.uuid, res.code, res.msg
                    )));
                }
            }
        }

        // 更新 last_update_ts
        ctx.update_synclog_for_db(&db.id, db.last_update);

        // 检查退出
        if ctx.is_exit() {
            return Ok(true);
        }
    }
    Ok(false)
}

//-----------------------------------------------------------------------------
pub async fn do_sync_person(ctx: Arc<AppCtx>) -> Result<bool, AppError> {
    let max_loop = 100;
    let mut loop_count = 0;

    let url = ctx.cfg.sync.server.person_sync.as_str();
    let hw_id = ctx.hw_id.as_str();
    let api = &ctx.sync_api;

    loop {
        let sync_log = ctx.get_sync_log();
        let last_ts = sync_log.person.last_ts;
        let last_ts = last_ts.format(utils::DATETIME_FMT_LONG).to_string();

        let res = api.fetch_person_updated(url, &last_ts, hw_id).await?;
        if res.status != 0 {
            return Err(AppError::new(&format!(
                "fetch_person_updated, return status: {}",
                res.status
            )));
        }

        // 检查退出信号
        if ctx.is_exit() {
            return Ok(true);
        }

        if res.is_empty() {
            // 没有更新数据，退出
            return Ok(false);
        }

        let res_data = res.data.unwrap();
        let exited = do_sync_person_batch(ctx.clone(), res_data).await?;
        if exited {
            return Ok(true);
        }

        loop_count += 1;
        // 循环次数过多，退出
        if loop_count >= max_loop {
            break;
        }

        // 休眠一会
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    Ok(false)
}

pub async fn do_sync_person_batch(ctx: Arc<AppCtx>, list: Vec<Person>) -> Result<bool, AppError> {
    for person in list.iter() {

        debug!("process sync_person, {}, {}",person.uuid,person.op);

        if person.op == SYNC_OP_DEL {
            // 删除
            let deled = ctx
                .recg_api
                .delete_person(person.db_id.clone(), person.uuid.clone())
                .await?;
            debug!(
                "WorkerService, delete person:{}, return: {:?}",
                person.uuid, deled
            );
        } else {
            //新增或修改

            // 先删除再加
            if person.detail.is_none() {
                return Err(AppError::new(&format!(
                    "person:{} detail is none",
                    person.uuid
                )));
            }

            let _deled = ctx
                .recg_api
                .delete_person(person.db_id.clone(), person.uuid.clone())
                .await?;
            // debug!(
            //     "WorkerService, for modify, delete person:{}, return: {:?}",
            //     person.uuid, deled
            // );

            let detail = person.detail.as_ref().unwrap();
            let fea_list: Vec<_> = detail
                .faces
                .iter()
                .map(|x| ApiFeatureQuality {
                    feature: x.fea.clone(),
                    quality: x.quality as f64,
                })
                .collect();

            let res = ctx
                .recg_api
                .create_persons(
                    person.db_id.clone(),
                    vec![person.uuid.clone()],
                    vec![fea_list],
                )
                .await?;

            if res.code != 0 {
                return Err(AppError::new(&format!(
                    "create_persons:{}, return code:{}, msg:{}",
                    person.uuid, res.code, res.msg
                )));
            }
        }

        // 更新 last_update_ts
        ctx.update_synclog_for_person(&person.id, person.last_update);

        // 检查退出
        if ctx.is_exit() {
            return Ok(true);
        }
    }
    Ok(false)
}
