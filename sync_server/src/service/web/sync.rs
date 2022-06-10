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

    Ok(ResponseData {
        status: 0,
        message: Some("success".to_string()),
        data: Some(vec![]),
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

    Ok(ResponseData {
        status: 0,
        message: Some("success".to_string()),
        data: Some(vec![]),
    })
}
