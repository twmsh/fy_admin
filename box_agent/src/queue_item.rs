use chrono::prelude::*;

use deadqueue::unlimited::Queue;
use fy_base::api::bm_api::{CarNotifyParams, FaceNotifyParams};
use fy_base::util::time_format::long_ts_format;
use serde::{Deserialize, Serialize};

//-----------------------
pub type FaceQueue = Queue<NotifyFaceQueueItem>;
pub type CarQueue = Queue<NotifyCarQueueItem>;

// ------------------- queue structs (face) -------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MatchPerson {
    pub db_id: String,
    pub uuid: String,
    pub score: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotifyFaceQueueItem {
    pub uuid: String,
    pub notify: FaceNotifyParams,

    #[serde(with = "long_ts_format")]
    pub ts: DateTime<Local>,

    pub matches: Option<Vec<MatchPerson>>,
}

// ------------------- queue structs (car) -------------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotifyCarQueueItem {
    pub uuid: String,
    pub notify: CarNotifyParams,

    #[serde(with = "long_ts_format")]
    pub ts: DateTime<Local>,
}

// --------------------------------------
#[derive(Debug)]
pub enum QI {
    FT(Box<NotifyFaceQueueItem>),
    CT(Box<NotifyCarQueueItem>),
}

impl QI {
    pub fn get_id(&self) -> String {
        match self {
            QI::FT(v) => v.uuid.clone(),
            QI::CT(v) => v.uuid.clone(),
        }
    }
}
