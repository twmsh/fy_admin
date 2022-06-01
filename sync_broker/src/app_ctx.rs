
use std::sync::Mutex;
use std::sync::Arc;
use chrono::{DateTime, Local};
use crate::app_cfg::{AppCfg, AppSyncLog, SyncConfigParams};

use tokio::sync::watch::Receiver;

use box_base::{
    service::SignalProduce,
    api::bm_api::{RecognitionApi, AnalysisApi}};


//---------------------

pub struct AppCtx {
    pub cfg: AppCfg,
    pub exit_rx: Receiver<i64>,

    pub ana_api: AnalysisApi,
    pub recg_api: RecognitionApi,
    pub hw_id: String,

    sync_log: Arc<Mutex<AppSyncLog>>,
    sync_params: Arc<Mutex<SyncConfigParams>>,

}


impl AppCtx {
    pub fn new(app_cfg: AppCfg, exit_rx: Receiver<i64>, hw_id: String,
               sync_log: AppSyncLog, sync_params: SyncConfigParams,
    ) -> Self {
        let ana_api = AnalysisApi::new(&app_cfg.api.grab_url);
        let recg_api = RecognitionApi::new(&app_cfg.api.recg_url);

        Self {
            cfg: app_cfg,
            exit_rx,
            ana_api,
            recg_api,
            hw_id,
            sync_log: Arc::new(Mutex::new(sync_log)),
            sync_params: Arc::new(Mutex::new(sync_params)),
        }
    }

    //------------------
    pub fn get_sync_log(&self) -> AppSyncLog {
        let guard = self.sync_log.lock().unwrap();
        guard.clone()
    }

    pub fn get_sync_params(&self) -> SyncConfigParams {
        let guard = self.sync_params.lock().unwrap();
        guard.clone()
    }

    pub fn update_sync_log(&self, sync_log: AppSyncLog) {
        let mut guard = self.sync_log.lock().unwrap();
        *guard = sync_log
    }

    pub fn update_sync_params(&self, config: SyncConfigParams) {
        let mut guard = self.sync_params.lock().unwrap();
        *guard = config
    }

    pub fn update_synclog_for_person(&self, last_id: String, last_ts: DateTime<Local>) {
        let mut guard = self.sync_log.lock().unwrap();
        guard.db.last_id = last_id;
        guard.db.last_ts = last_ts;
    }

    pub fn update_synclog_for_camera(&self, last_id: String, last_ts: DateTime<Local>) {
        let mut guard = self.sync_log.lock().unwrap();
        guard.camera.last_id = last_id;
        guard.camera.last_ts = last_ts;
    }

    //------------------
    pub fn is_exit(&self) -> bool {
        let value = self.exit_rx.borrow();
        *value == 100
    }
}

impl SignalProduce for AppCtx {
    fn clone_receiver(&self) -> Receiver<i64> {
        self.exit_rx.clone()
    }
}