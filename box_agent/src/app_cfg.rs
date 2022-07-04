use crate::error::{AppError, AppResult};

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
    pub grab_url: String,
    pub recg_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgTrackFace {
    pub skip_search: bool,
    pub clear_delay: u64,
    pub ready_delay: u64,
    pub count: u64,
    pub quality: f64,

    pub search_top: u64,
    pub search_threshold: u64,
    pub cache_ttl: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgTrackCar {
    pub clear_delay: u64,
    pub ready_delay: u64,
    pub count: u64,
    pub conf: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgTrack {
    pub local_address: String,
    pub face: AppCfgTrackFace,
    pub car: AppCfgTrackCar,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfgUplink {
    pub max_queue: u64,
    pub server: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCfg {
    pub version: AppCfgVersion,
    pub log: AppCfgLog,
    pub api: AppCfgApi,
    pub track: AppCfgTrack,
    pub up_link: AppCfgUplink,
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
