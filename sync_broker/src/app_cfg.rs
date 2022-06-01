use crate::error::{AppError, AppResult};
use chrono::{DateTime, Local, TimeZone};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

use box_base::util::time_format::long_ts_format;

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgVersion {
    pub product: String,
    pub ver: String,
    pub api_ver: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgLog {
    pub app: String,
    pub web: String,
    pub level: String,
    pub lib_level: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgDb {
    pub url: String,
    pub tz: String,
    pub max_conn: u64,
    pub min_conn: u64,
    pub idle: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgRabbitMq {
    pub url: String,
    pub queue: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfg {
    pub version: AppCfgVersion,
    pub log: AppCfgLog,
    pub db: AppCfgDb,

    pub rabbitmq: AppCfgRabbitMq,
}

impl AppCfg {
    pub fn load<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        let file = fs::File::open(path)?;
        let cfg = serde_json::from_reader(file)?;
        Ok(cfg)
    }

    pub fn validate(&self) -> AppResult<()> {
        if let Err(e) = self.track.local_address.parse::<SocketAddr>() {
            return Err(AppError::from_debug(e));
        };

        Ok(())
    }
}

//----------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppSyncLogDb {
    #[serde(with = "long_ts_format")]
    pub last_ts: DateTime<Local>,
    pub last_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppSyncLogCamera {
    #[serde(with = "long_ts_format")]
    pub last_ts: DateTime<Local>,
    pub last_id: String,
}

//----------------------
impl Default for AppSyncLogDb {
    fn default() -> Self {
        Self {
            last_ts: Local.timestamp(0, 0),
            last_id: "".into(),
        }
    }
}

impl Default for AppSyncLogCamera {
    fn default() -> Self {
        Self {
            last_ts: Local.timestamp(0, 0),
            last_id: "".into(),
        }
    }
}

//----------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppSyncLog {
    pub db: AppSyncLogDb,
    pub camera: AppSyncLogCamera,
    pub uplink_url: String,

    pub db_id: String,
    pub hw_id: String,
    pub device_id: String,
}

impl Default for AppSyncLog {
    fn default() -> Self {
        Self {
            db: Default::default(),
            camera: Default::default(),
            uplink_url: "".to_string(),

            db_id: "".to_string(),
            hw_id: "".to_string(),
            device_id: "".to_string(),
        }
    }
}

impl AppSyncLog {
    pub fn load<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        let file = fs::File::open(path)?;
        let cfg = serde_json::from_reader(file)?;
        Ok(cfg)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> AppResult<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

//----------------------------------------------
// mqtt 消息中发送过来的同步相关参数配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncConfigParams {
    pub sync_person_url: String,
    pub sync_camera_url: String,
    pub db_id: String,
    pub uplink_url: String,
    pub device_id: String,
}

impl Default for SyncConfigParams {
    fn default() -> Self {
        Self {
            sync_person_url: "".to_string(),
            sync_camera_url: "".to_string(),
            db_id: "".to_string(),
            uplink_url: "".to_string(),
            device_id: "".to_string(),
        }
    }
}
