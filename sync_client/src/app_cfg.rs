use crate::error::AppResult;
use chrono::{DateTime, Local, TimeZone};
use fy_base::util::time_format::long_ts_format;
use serde::{Deserialize, Serialize};
use std::fs;
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
    pub grab_url: String,
    pub recg_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgSyncServer {
    pub db_sync: String,
    pub person_sync: String,
    pub camera_sync: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgSync {
    pub sync_log: String,
    pub server: AppCfgSyncServer,
    pub heartbeat: u64, // 心跳间隔
    pub sync_ttl: u64,  // 多久触发同步
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgRabbitMqItem {
    pub queue: String,
    pub exchange: String,
    pub route_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgRabbitMq {
    pub url: String,
    pub log: AppCfgRabbitMqItem,
    pub cmd: AppCfgRabbitMqItem,
}

//----------------------------
#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfg {
    pub version: AppCfgVersion,
    pub log: AppCfgLog,
    pub api: AppCfgApi,
    pub sync: AppCfgSync,
    pub rabbitmq: AppCfgRabbitMq,
    pub hw_id: Option<String>,
}

impl AppCfg {
    pub fn load<P: AsRef<Path>>(path: P) -> AppResult<Self> {
        let file = fs::File::open(path)?;
        let cfg = serde_json::from_reader(file)?;
        Ok(cfg)
    }

    pub fn validate(&self) -> AppResult<()> {
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
pub struct AppSyncLogPerson {
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

impl Default for AppSyncLogPerson {
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppSyncLog {
    pub db: AppSyncLogDb,
    pub person: AppSyncLogPerson,
    pub camera: AppSyncLogCamera,
    pub hw_id: String,
}

impl Default for AppSyncLog {
    fn default() -> Self {
        Self {
            db: Default::default(),
            person: Default::default(),
            camera: Default::default(),
            hw_id: "".to_string(),
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
