use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

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
pub struct AppCfgHttp {
    pub addr: String,
    pub max_conn: u64,
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


        Ok(())
    }
}

//----------------------------------------------
