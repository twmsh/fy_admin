use crate::error::AppResult;
use fy_base::util::mysql_util;
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
pub struct AppCfgClean {
    pub ttl_day: u64,       // 保留多少天的box_log
    pub interval_hour: u64,  // 多少秒执行一次
}


#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgHttp {
    pub addr: String,
    pub max_conn: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgRabbitMq {
    pub url: String,
    pub queue: String,
    pub exchange: String,
    pub route_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfg {
    pub version: AppCfgVersion,
    pub log: AppCfgLog,
    pub db: AppCfgDb,
    pub clean: AppCfgClean,
    pub sync_batch: u32,
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

        Ok(())
    }
}

//----------------------------------------------
