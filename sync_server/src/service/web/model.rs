use axum::Json;
use axum::response::{IntoResponse, Response};
use crate::util::time_format::long_ts_format;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use crate::dao::base_model::{BaseCamera, BaseCameraDel, BaseDb, BaseDbDel, BaseFeaDel};
use crate::error::AppError;

//----------------------------------

pub const SYNC_OP_MODIFY: i8 = 1;
pub const SYNC_OP_DEL: i8 = 2;

pub const RES_STATUS_OK: i32 = 0;
pub const RES_STATUS_ERROR: i32 = 500;
pub const RES_STATUS_INVALID_PARA: i32 = 1;
pub const RES_STATUS_BIZ_ERR: i32 = 2;



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

#[derive(Serialize, Deserialize, Debug)]
pub struct Db {
    pub id: String,
    pub uuid: String,

    // 1：增加或修改 2：删除
    pub op: i8,

    #[serde(with = "long_ts_format")]
    pub last_update: DateTime<Local>,

    pub capacity: i32,

}

//----------------------------------
#[derive(sqlx::FromRow,  Serialize, Deserialize, Debug, Clone)]
pub struct BaseFeaMapRow {
    pub id: i64,
    pub uuid: String,
    pub face_id: String,
    pub fea: String,
    pub modify_time: DateTime<Local>,
}


//----------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseData<T> {
    pub status: i32,
    pub message: Option<String>,

    #[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
    #[serde(skip_serializing_if = "Option::is_none")]
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

impl From<AppError> for ResponseData<()> {
    fn from(e: AppError) -> Self {
        ResponseData {
            status: RES_STATUS_ERROR,
            message: Some(e.msg),
            data: None,
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


// impl IntoResponse for AppError
// {
//     fn into_response(self) -> Response {
//         ResponseData::<String> {
//             status: RES_STATUS_ERROR,
//             message: Some(self.msg),
//             data: None,
//         }.into_response()
//     }
// }

//-----------------
pub fn build_fail_response_data(status: i32, message: &str) -> ResponseData<()> {
    ResponseData {
        status,
        message: Some(message.to_string()),
        data: None,
    }
}

//---------------------------------------------
impl From<BaseDb> for Db {
    fn from(obj: BaseDb) -> Db {
        Db {
            id: obj.id.to_string(),
            uuid: obj.uuid,
            op: SYNC_OP_MODIFY,
            last_update: obj.modify_time,
            capacity: obj.capacity,
        }
    }
}

impl From<BaseDbDel> for Db {
    fn from(obj: BaseDbDel) -> Db {
        Db {
            id: obj.origin_id.to_string(),
            uuid: obj.uuid,
            op: SYNC_OP_DEL,
            last_update: obj.modify_time,
            capacity: obj.capacity,
        }
    }
}


impl From<BaseCamera> for Camera {
    fn from(obj: BaseCamera) -> Camera {
        Camera {
            id: obj.id.to_string(),
            uuid: obj.uuid.clone(),
            op: SYNC_OP_MODIFY,
            last_update: obj.modify_time,
            c_type: obj.c_type as i64,

            detail: Some(CameraInfo {
                uuid: obj.uuid,
                url: obj.url,
                config: obj.config,
            }),
        }
    }
}


impl From<BaseCameraDel> for Camera {
    fn from(obj:BaseCameraDel) -> Camera {
        Camera {
            id: obj.origin_id.to_string(),
            uuid: obj.uuid.clone(),
            op: SYNC_OP_DEL,
            last_update: obj.modify_time,
            c_type: obj.c_type as i64,

            detail: None,
        }
    }
}

//----------------------------


impl From<BaseFeaDel> for Person {
    fn from(obj: BaseFeaDel) -> Person {
        Person {
            id: obj.origin_id.to_string(),
            uuid: obj.uuid,
            db_id: obj.db_uuid,
            op: SYNC_OP_DEL,
            last_update: obj.modify_time,

            detail: None,
        }
    }
}

//----------------------------------
pub fn get_personinfo_from_map(_row: Vec<BaseFeaMapRow>) -> Vec<Person> {
    vec![]
}
