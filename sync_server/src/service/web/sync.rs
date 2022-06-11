use crate::service::web::model::{
    build_fail_response_data, get_personinfo_from_map, Camera, Db, Person, ResponseData,
    RES_STATUS_BIZ_ERR, RES_STATUS_INVALID_PARA,
};
use crate::service::web::WebState;
use crate::util::utils;
use crate::util::utils::DATETIME_FMT_LONG;
use axum::extract::Query;

use axum::Extension;
use chrono::{Local, NaiveDateTime};
use serde::Deserialize;

use std::sync::Arc;
use tracing::{debug, error};

fn build_success_response<T>(list: Vec<T>) -> ResponseData<T> {
    ResponseData {
        status: 0,
        message: Some("success".to_string()),
        ts: Local::now(),
        data: Some(list),
    }
}

fn build_invalid_paras_response(msg: &str) -> ResponseData<()> {
    build_fail_response_data(RES_STATUS_INVALID_PARA, msg)
}

fn build_device_notfound_response(hw_id: &str) -> ResponseData<()> {
    build_fail_response_data(RES_STATUS_BIZ_ERR, &format!("box:{} not found", hw_id))
}

//--------------------------------------------
fn check_para_exist(para: &Option<String>) -> bool {
    match para {
        None => false,
        Some(ref v) => !v.is_empty(),
    }
}

fn check_para_lastupdate(para: &Option<String>) -> bool {
    match para {
        None => false,
        Some(ref v) => {
            if v.is_empty() {
                return false;
            }

            if NaiveDateTime::parse_from_str(v.as_str(), DATETIME_FMT_LONG).is_err() {
                return false;
            }
            true
        }
    }
}

//----------------------------- db sync  --------------------------------------
#[derive(Debug, Deserialize)]
pub struct DbUpdateParas {
    device_id: Option<String>,
    last_update: Option<String>,
}

fn check_dbupdate_paras(paras: &DbUpdateParas) -> Result<(), ResponseData<()>> {
    if !check_para_exist(&paras.device_id) {
        return Err(build_invalid_paras_response("invalid device_id"));
    }
    if !check_para_lastupdate(&paras.last_update) {
        return Err(build_invalid_paras_response("invalid last_update"));
    }

    Ok(())
}

pub async fn get_db_update(
    Extension(state): Extension<Arc<WebState>>,
    Query(paras): Query<DbUpdateParas>,
) -> Result<ResponseData<Db>, ResponseData<()>> {
    debug!("get_db_update, paras: {:?}", paras);

    // 检查参数
    let _ = check_dbupdate_paras(&paras)?;

    let device_id = paras.device_id.unwrap();
    let last_update =
        utils::parse_localtime_str(paras.last_update.unwrap().as_str(), DATETIME_FMT_LONG).unwrap();

    // 检查 box，是否需要同步 db
    let base_box = match state.ctx.dao.find_box(device_id.clone()).await {
        Ok(v) => v,
        Err(e) => {
            error!("error, find_box({}), err: {:?}", device_id, e);
            return Err(e.into());
        }
    };
    match base_box {
        None => {
            // 不在硬件表中
            return Err(build_device_notfound_response(device_id.as_str()));
        }
        Some(ref v) => {
            // 不需要同步
            if v.has_db == 0 || v.sync_flag == 0 {
                return Ok(build_success_response(vec![]));
            }
        }
    };

    debug!("last_update: {}", last_update);

    let limit = state.ctx.cfg.sync_batch;
    let db_update = match state.ctx.dao.get_db_list(last_update, limit).await {
        Ok(v) => v,
        Err(e) => {
            error!("error, get_db_list({}), err: {:?}", device_id, e);
            return Err(e.into());
        }
    };
    let db_del_update = match state.ctx.dao.get_dbdel_list(last_update, limit).await {
        Ok(v) => v,
        Err(e) => {
            error!("error, get_dbdel_list({}), err: {:?}", device_id, e);
            return Err(e.into());
        }
    };

    // 合并 db和 db_del的记录
    let mut list: Vec<Db> = vec![];
    for v in db_update {
        list.push(v.into());
    }
    for v in db_del_update {
        list.push(v.into());
    }

    // 根据 last_update排序
    list.sort_by(|a, b| a.last_update.cmp(&b.last_update));

    // 取前N条记录
    list.truncate(limit as usize);

    // 返回值
    Ok(build_success_response(list))
}

//----------------------------- person sync  --------------------------------------
#[derive(Debug, Deserialize)]
pub struct PersonUpdateParas {
    device_id: Option<String>,
    last_update: Option<String>,
}

fn check_personupdate_paras(paras: &PersonUpdateParas) -> Result<(), ResponseData<()>> {
    if !check_para_exist(&paras.device_id) {
        return Err(build_invalid_paras_response("invalid device_id"));
    }
    if !check_para_lastupdate(&paras.last_update) {
        return Err(build_invalid_paras_response("invalid last_update"));
    }

    Ok(())
}

pub async fn get_person_update(
    Extension(state): Extension<Arc<WebState>>,
    Query(paras): Query<PersonUpdateParas>,
) -> Result<ResponseData<Person>, ResponseData<()>> {
    debug!("get_person_update, paras: {:?}", paras);

    // 检查参数
    let _ = check_personupdate_paras(&paras)?;

    let device_id = paras.device_id.unwrap();
    let last_update =
        utils::parse_localtime_str(paras.last_update.unwrap().as_str(), DATETIME_FMT_LONG).unwrap();

    debug!("last_update: {}", last_update);

    // 检查 box，是否需要同步 person
    let base_box = match state.ctx.dao.find_box(device_id.clone()).await {
        Ok(v) => v,
        Err(e) => {
            error!("error, find_box({}), err: {:?}", device_id, e);
            return Err(e.into());
        }
    };
    match base_box {
        None => {
            // 不在硬件表中
            return Err(build_device_notfound_response(device_id.as_str()));
        }
        Some(ref v) => {
            // 不需要同步 person
            if v.has_db == 0 || v.sync_flag == 0 {
                return Ok(build_success_response(vec![]));
            }
        }
    };

    let limit = state.ctx.cfg.sync_batch;
    let list_update = match state.ctx.dao.get_feamap_row_list(last_update, limit).await {
        Ok(v) => v,
        Err(e) => {
            error!("error, get_feamap_row_list({}), err: {:?}", last_update, e);
            return Err(e.into());
        }
    };

    debug!("list_update: {:?}", list_update);

    // 从fea_map 转成 Person
    let mut list_update = get_personinfo_from_map(list_update);
    debug!("list_update: {:?}", list_update);

    let list_del_update = match state.ctx.dao.get_feadel_list(last_update, limit).await {
        Ok(v) => v,
        Err(e) => {
            error!("error, get_feadel_list({}), err: {:?}", last_update, e);
            return Err(e.into());
        }
    };
    debug!("list_del_update: {:?}", list_del_update);

    // 合并 db和 db_del的记录
    let mut list: Vec<Person> = vec![];

    list.append(&mut list_update);

    for v in list_del_update {
        list.push(v.into());
    }

    // 根据 last_update排序
    list.sort_by(|a, b| a.last_update.cmp(&b.last_update));

    // 取前N条记录
    list.truncate(limit as usize);

    Ok(build_success_response(list))
}

//----------------------------- camera sync  --------------------------------------
#[derive(Debug, Deserialize)]
pub struct CameraUpdateParas {
    device_id: Option<String>,
    last_update: Option<String>,
}

fn check_cameraupdate_paras(paras: &CameraUpdateParas) -> Result<(), ResponseData<()>> {
    if !check_para_exist(&paras.device_id) {
        return Err(build_invalid_paras_response("invalid device_id"));
    }
    if !check_para_lastupdate(&paras.last_update) {
        return Err(build_invalid_paras_response("invalid last_update"));
    }

    Ok(())
}

pub async fn get_camera_update(
    Extension(state): Extension<Arc<WebState>>,
    Query(paras): Query<CameraUpdateParas>,
) -> Result<ResponseData<Camera>, ResponseData<()>> {
    debug!("get_camera_update, paras: {:?}", paras);

    // 检查参数
    let _ = check_cameraupdate_paras(&paras)?;

    let device_id = paras.device_id.unwrap();
    let last_update =
        utils::parse_localtime_str(paras.last_update.unwrap().as_str(), DATETIME_FMT_LONG).unwrap();

    debug!("last_update: {}", last_update);

    // 检查 box，是否需要同步 camera
    let base_box = match state.ctx.dao.find_box(device_id.clone()).await {
        Ok(v) => v,
        Err(e) => {
            error!("error, find_box({}), err: {:?}", device_id, e);
            return Err(e.into());
        }
    };
    match base_box {
        None => {
            // 不在硬件表中
            return Err(build_device_notfound_response(device_id.as_str()));
        }
        Some(ref v) => {
            // 不需要同步camera
            if v.has_camera == 0 || v.sync_flag == 0 {
                return Ok(build_success_response(vec![]));
            }
        }
    };

    let limit = state.ctx.cfg.sync_batch;
    let list_update = match state
        .ctx
        .dao
        .get_camera_list(last_update, &device_id, limit)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            error!("error, get_camera_list({}), err: {:?}", device_id, e);
            return Err(e.into());
        }
    };
    let list_del_update = match state
        .ctx
        .dao
        .get_cameradel_list(last_update, &device_id, limit)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            error!("error, get_cameradel_list({}), err: {:?}", device_id, e);
            return Err(e.into());
        }
    };

    // 合并 db和 db_del的记录
    let mut list: Vec<Camera> = vec![];
    for v in list_update {
        list.push(v.into());
    }
    for v in list_del_update {
        list.push(v.into());
    }

    // 根据 last_update排序
    list.sort_by(|a, b| a.last_update.cmp(&b.last_update));

    // 取前N条记录
    list.truncate(limit as usize);

    // 返回值
    Ok(build_success_response(list))
}
