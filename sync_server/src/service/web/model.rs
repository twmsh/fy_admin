use axum::Json;
use axum::response::{IntoResponse, Response};
use crate::util::time_format::long_ts_format;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

//----------------------------------

pub const SYNC_OP_MODIFY: i8 = 1;
pub const SYNC_OP_DEL: i8 = 2;

#[derive(Serialize, Deserialize, Debug)]
pub struct PersonInfoFace {
    //base64
    pub fea: String,
    pub quality: f64, //base64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PersonInfo {
    pub uuid: String,
    pub db_id: String,

    //base64 是否需要
    pub aggregate: Option<String>,
    pub faces: Vec<PersonInfoFace>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Person {
    pub id: String,
    pub uuid: String,
    pub db_id: String,

    // 1：增加或修改 2：删除
    pub op: i8,

    #[serde(with = "long_ts_format")]
    pub last_update: DateTime<Local>,

    pub detail: Option<PersonInfo>,
}

//----------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct Camera {
    pub id: String,
    pub uuid: String,

    // 1：增加或修改 2：删除
    pub op: i8,

    #[serde(with = "long_ts_format")]
    pub last_update: DateTime<Local>,

    // 采集类型 1：人脸 2：车辆 3：人脸+车辆
    #[serde(rename = "type")]
    pub c_type: i64,

    pub detail: Option<CameraInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CameraInfo {
    pub uuid: String,

    // 采集地址
    pub url: String,

    // 摄像头配置
    pub config: String,
}

//----------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseData<T> {
    pub status: i32,
    pub message: Option<String>,

    #[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
    pub data: Option<Vec<T>>,
}

impl<T> ResponseData<T> {
    pub fn is_empty(&self) -> bool {
        match self.data {
            None => true,
            Some(ref v) => v.is_empty(),
        }
    }
}

//-------------------------------
impl<T> IntoResponse for ResponseData<T>
    where T: Serialize,
{
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

//-------------------------
pub const RES_STATUS_OK: i32 = 0;
pub const RES_STATUS_ERROR: i32 = 500;
pub const RES_STATUS_INVALID_PARA: i32 = 1;


impl IntoResponse for AppError
{
    fn into_response(self) -> Response {
        ResponseData::<String> {
            status: RES_STATUS_ERROR,
            message: Some(self.msg),
            data: None,
        }.into_response()
    }
}

//-----------------
pub fn build_fail_response_data( status: i32, message: &str)-> ResponseData<()> {
    ResponseData {
        status,
        message: Some(message.to_string()),
        data: None,
    }
}
