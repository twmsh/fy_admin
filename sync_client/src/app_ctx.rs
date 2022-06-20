use std::sync::{Arc, Mutex};
use chrono::{DateTime, Local};
use tokio::sync::watch::Receiver;
use tracing::{debug, error};

use crate::app_cfg::{AppCfg, AppSyncLog};
use fy_base::{
    api::bm_api::{AnalysisApi, RecognitionApi},
    util::service::SignalProduce,
};
use fy_base::api::sync_api::Api;

//---------------------

pub struct AppCtx {
    pub cfg: AppCfg,
    pub exit_rx: Receiver<i64>,

    pub sync_log: Arc<Mutex<AppSyncLog>>,

    pub ana_api: AnalysisApi,
    pub recg_api: RecognitionApi,
    pub sync_api: Api,

    pub hw_id: String,
}

impl AppCtx {
    pub fn new(cfg: AppCfg, exit_rx: Receiver<i64>, sync_log: AppSyncLog, hw_id: String) -> Self {
        let ana_api = AnalysisApi::new(&cfg.api.grab_url);
        let recg_api = RecognitionApi::new(&cfg.api.recg_url);

        Self {
            cfg,
            exit_rx,
            ana_api,
            recg_api,
            sync_api: Api::default(),
            sync_log: Arc::new(Mutex::new(sync_log)),
            hw_id,
        }
    }

    //------------------

    //------------------
    pub fn is_exit(&self) -> bool {
        let value = self.exit_rx.borrow();
        *value == 100
    }

    //------------------
    pub fn get_sync_log(&self) -> AppSyncLog {
        let guard = self.sync_log.lock().unwrap();
        guard.clone()
    }

    pub fn update_synclog_for_db(&self, last_id: &str, last_ts: DateTime<Local>) {
        let mut guard = self.sync_log.lock().unwrap();
        guard.db.last_id = last_id.to_string();
        guard.db.last_ts = last_ts;
    }

    pub fn update_synclog_for_person(&self, last_id: &str, last_ts: DateTime<Local>) {
        let mut guard = self.sync_log.lock().unwrap();
        guard.person.last_id = last_id.to_string();
        guard.person.last_ts = last_ts;
    }

    pub fn update_synclog_for_camera(&self, last_id: &str, last_ts: DateTime<Local>) {
        let mut guard = self.sync_log.lock().unwrap();
        guard.camera.last_id = last_id.to_string();
        guard.camera.last_ts = last_ts;
    }


    pub fn save_sync_log(&self) {
        let sync_log = self.get_sync_log();
        let dst_fn = &self.cfg.sync.sync_log;

        let content = serde_json::to_string_pretty(&sync_log);
        match content {
            Ok(v) => {
                match std::fs::write(dst_fn, v) {
                    Ok(_) => {
                        debug!("save_sync_log, ok, {}", dst_fn);
                    }
                    Err(e) => {
                        error!("error, save_sync_log, {}, err: {:?}", dst_fn, e);
                    }
                };
            }
            Err(e) => {
                error!("error, save_sync_log, {}, err: {:?}", dst_fn, e);
            }
        }
    }
}

impl SignalProduce for AppCtx {
    fn clone_receiver(&self) -> Receiver<i64> {
        self.exit_rx.clone()
    }
}
