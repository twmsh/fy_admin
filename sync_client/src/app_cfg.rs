use crate::error::AppResult;
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
pub struct AppCfgServer {
    pub db_sync: String,
    pub person_sync: String,
    pub camera_sync: String,
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
    pub server: AppCfgServer,
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
