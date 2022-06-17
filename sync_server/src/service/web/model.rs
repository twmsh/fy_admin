use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::prelude::*;
use fy_base::util::time_format::long_ts_format;
use std::collections::HashMap;

use crate::dao::base_model::{BaseCamera, BaseCameraDel, BaseDb, BaseDbDel, BaseFeaDel};
use crate::error::AppError;
use serde::{Deserialize, Serialize};

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
    pub quality: f32,
    //base64
    pub id: String, // face num
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PersonInfo {
    pub uuid: String,
    pub db_id: String,

    //base64 是否需要?
    #[serde(skip_serializing_if = "Option::is_none")]
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

impl Person {
    pub fn add_face(&mut self, face: PersonInfoFace) {
        match self.detail {
            None => {
                let info = PersonInfo {
                    uuid: self.uuid.clone(),
                    db_id: self.db_id.clone(),
                    aggregate: None,
                    faces: vec![face],
                };
                self.detail = Some(info);
            }
            Some(ref mut v) => {
                v.faces.push(face);
            }
        }
    }
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseData<T> {
    pub status: i32,
    pub message: Option<String>,

    #[serde(with = "long_ts_format")]
    pub ts: DateTime<Local>,

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


//----------------------------------
#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct BaseFeaMapRow {
    pub id: i64,
    pub db_uuid: String,
    pub uuid: String,
    pub face_id: String,
    pub feature: String,
    pub quality: f32,
    pub modify_time: DateTime<Local>,
}

//----------------------------------
impl From<AppError> for ResponseData<()> {
    fn from(e: AppError) -> Self {
        ResponseData {
            status: RES_STATUS_ERROR,
            message: Some(e.msg),
            ts: Local::now(),
            data: None,
        }
    }
}

//-------------------------------
impl<T> IntoResponse for ResponseData<T>
where
    T: Serialize,
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
        ts: Local::now(),
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
    fn from(obj: BaseCameraDel) -> Camera {
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
// 需要合并多个fea到一个person中
pub fn get_personinfo_from_map(rows: Vec<BaseFeaMapRow>) -> Vec<Person> {
    let mut map: HashMap<String, Person> = HashMap::new();

    for v in rows.iter() {
        let face = PersonInfoFace {
            fea: v.feature.clone(),
            quality: v.quality,
            id: v.face_id.to_string(),
        };
        let person = map.get_mut(&v.uuid);
        match person {
            None => {
                // 不存在，new一个
                let mut person_new = Person {
                    id: v.id.to_string(),
                    uuid: v.uuid.clone(),
                    db_id: v.db_uuid.clone(),
                    op: SYNC_OP_MODIFY,
                    last_update: v.modify_time,
                    detail: None,
                };
                person_new.add_face(face);
                map.insert(v.uuid.clone(), person_new);
            }
            Some(vv) => {
                // 已经存在
                vv.add_face(face);
            }
        }
    }

    map.into_values().collect()
}
