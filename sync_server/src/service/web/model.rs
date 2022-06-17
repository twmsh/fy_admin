use chrono::prelude::*;

use std::collections::HashMap;

use crate::dao::base_model::{BaseCamera, BaseCameraDel, BaseDb, BaseDbDel, BaseFeaDel};
use crate::error::AppError;
use serde::{Deserialize, Serialize};

use fy_base::sync::response_type::{
    Camera, CameraInfo, Db, Person, PersonInfoFace, ResponseData, RES_STATUS_ERROR, SYNC_OP_DEL,
    SYNC_OP_MODIFY,
};

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
