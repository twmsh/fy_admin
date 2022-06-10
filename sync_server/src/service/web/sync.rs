use crate::service::web::model::{build_fail_response_data, Db, ResponseData, RES_STATUS_INVALID_PARA, PersonInfo, Camera};
use crate::service::web::WebState;
use crate::util::utils::DATETIME_FMT_LONG;
use axum::extract::Query;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;
use crate::util::utils;


fn build_invalid_paras_response(msg: &str) -> ResponseData<()> {
    build_fail_response_data(RES_STATUS_INVALID_PARA, msg)
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
    let last_update = utils::parse_localtime_str(
        paras.last_update.unwrap().as_str(), DATETIME_FMT_LONG).unwrap();

    debug!("last_update: {}",last_update);

    let limit = state.ctx.cfg.sync_batch;
    let db_update = state.ctx.dao.get_db_list(last_update,limit).await?;
    let db_del_update = state.ctx.dao.get_dbdel_list(last_update,limit).await?;

    // 合并 db和 db_del的记录
    let mut list:Vec<Db> =vec![];
    for v in db_update {
        list.push(v.into());
    }
    for v in db_del_update {
        list.push(v.into());
    }

    // 根据 last_update排序
    list.sort_by(|a,b|{
        a.last_update.cmp(&b.last_update)
    });

    // 取前N条记录
    list.truncate(limit as usize);

    // 返回值
    Ok(ResponseData {
        status: 0,
        message: Some("success".to_string()),
        data: Some(list),
    })
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
) -> Result<ResponseData<PersonInfo>, ResponseData<()>> {
    debug!("get_person_update, paras: {:?}", paras);

    // 检查参数
    let _ = check_personupdate_paras(&paras)?;

    let device_id = paras.device_id.unwrap();
    let last_update = utils::parse_localtime_str(
        paras.last_update.unwrap().as_str(), DATETIME_FMT_LONG).unwrap();

    debug!("last_update: {}",last_update);

    Ok(ResponseData {
        status: 0,
        message: Some("success".to_string()),
        data: Some(vec![]),
    })
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
    let last_update = utils::parse_localtime_str(
        paras.last_update.unwrap().as_str(), DATETIME_FMT_LONG).unwrap();

    debug!("last_update: {}",last_update);

    let limit = state.ctx.cfg.sync_batch;
    let list_update = state.ctx.dao.get_camera_list(last_update,limit).await?;
    let list_del_update = state.ctx.dao.get_cameradel_list(last_update,limit).await?;

    // 合并 db和 db_del的记录
    let mut list:Vec<Camera> =vec![];
    for v in list_update {
        list.push(v.into());
    }
    for v in list_del_update {
        list.push(v.into());
    }

    // 根据 last_update排序
    list.sort_by(|a,b|{
        a.last_update.cmp(&b.last_update)
    });

    // 取前N条记录
    list.truncate(limit as usize);

    // 返回值
    Ok(ResponseData {
        status: 0,
        message: Some("success".to_string()),
        data: Some(list),
    })
}
