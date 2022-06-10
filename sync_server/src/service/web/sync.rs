use std::sync::Arc;
use serde::{Deserialize, Serialize};
use axum::Extension;
use axum::extract::Query;
use axum::response::{IntoResponse, Response};
use chrono::NaiveDateTime;
use tracing::debug;
use crate::service::web::model::{build_fail_response_data, RES_STATUS_INVALID_PARA, ResponseData};
use crate::service::web::WebState;
use crate::util::utils::DATETIME_FMT_LONG;

#[derive(Debug, Deserialize)]
pub struct DbUpdateParas {
    device_id: Option<String>,
    last_update: Option<String>,
}


fn build_invalid_paras_response(msg: &str) -> ResponseData<()> {
    build_fail_response_data(RES_STATUS_INVALID_PARA, msg)
}

fn check_para_exist(para: &Option<String>) -> bool {
    match para {
        None => false,
        Some(ref v) => {
            !v.is_empty()
        }
    }
}

fn check_para_lastupdate(para: &Option<String>) -> bool {
    match para {
        None => false,
        Some(ref v) => {
            if v.is_empty() {
                return false;
            }

            //
            if NaiveDateTime::parse_from_str(v.as_str(), DATETIME_FMT_LONG).is_err() {
                return false;
            }
            true
        }
    }
}


fn check_dbupdate_paras(paras: &DbUpdateParas) -> Result<(), ResponseData<()>> {

    if !check_para_exist(&paras.device_id) {
        return Err(build_invalid_paras_response("invalid device_id"));
    }
    if !check_para_lastupdate(&paras.last_update) {
        return Err(build_invalid_paras_response("invalid device_id"));
    }

    Ok(())
}

pub async fn get_db_update(Extension(state): Extension<Arc<WebState>>, Query(paras): Query<DbUpdateParas>) -> Response {
    debug!("paras: {:?}",paras);
    let _ = check_dbupdate_paras(&paras)?;

    ResponseData::<String> {
        status: 0,
        message: Some("success".to_string()),
        data: Some(vec!["haha".to_string()])
    }.into_response()
}