use crate::error::{AppError, AppResult};

use fy_base::util::mysql_util;

use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgVersion {
    pub product: String,
    pub ver: String,
    pub api_ver: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgLog {
    pub app: String,
    pub level: String,
    pub lib_level: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgApi {
    pub need_search: bool,
    // 是否倒查，倒查的话，
    // 需要将track特征值保存到facetrack_db中
    pub need_dc: bool,
    pub facetrack_db: String,
    pub grab_url: String,
    pub recg_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgRabbitMqItem {
    pub queue: String,
    pub exchange: String,
    pub route_key: String,
    pub expire: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgRabbitMq {
    pub url: String,
    pub face: AppCfgRabbitMqItem,
    pub car: AppCfgRabbitMqItem,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgHttp {
    pub addr: String,
    pub max_conn: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgDb {
    pub url: String,
    pub tz: String,
    pub max_conn: u64,
    pub min_conn: u64,
    pub idle: u64,
}

//----------------------------
#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfg {
    pub version: AppCfgVersion,
    pub log: AppCfgLog,
    pub db: AppCfgDb,
    pub api: AppCfgApi,
    pub http: AppCfgHttp,
    pub rabbitmq: AppCfgRabbitMq,
}

impl AppCfg {
    pub fn load<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        let file = fs::File::open(path)?;
        let cfg = serde_json::from_reader(file)?;
        Ok(cfg)
    }

    pub fn validate(&self) -> AppResult<()> {
        let _ = mysql_util::parse_timezone(self.db.tz.as_str())?;

        if let Err(e) = self.http.addr.parse::<SocketAddr>() {
            return Err(AppError::from_debug(e));
        };

        Ok(())
    }
}

//----------------------------------------------
