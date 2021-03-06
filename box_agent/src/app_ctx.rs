use tokio::sync::watch::Receiver;

use crate::app_cfg::AppCfg;

use fy_base::{
    api::bm_api::{AnalysisApi, RecognitionApi},
    util::service::SignalProduce,
};

//---------------------

pub struct AppCtx {
    pub cfg: AppCfg,
    pub exit_rx: Receiver<i64>,

    pub ana_api: AnalysisApi,
    pub recg_api: RecognitionApi,
}

impl AppCtx {
    pub fn new(cfg: AppCfg, exit_rx: Receiver<i64>) -> Self {
        let ana_api = AnalysisApi::new(&cfg.api.grab_url);
        let recg_api = RecognitionApi::new(&cfg.api.recg_url);

        Self {
            cfg,
            exit_rx,
            ana_api,
            recg_api,
        }
    }

    //------------------

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
