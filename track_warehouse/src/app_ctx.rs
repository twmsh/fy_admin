use tokio::sync::watch::Receiver;

use crate::app_cfg::AppCfg;

use crate::dao::Dao;
use fy_base::{api::bm_api::RecognitionApi, util::service::SignalProduce};

//---------------------

pub struct AppCtx {
    pub cfg: AppCfg,
    pub exit_rx: Receiver<i64>,

    pub search_recg_api: RecognitionApi,
    pub trackdb_recg_api: RecognitionApi,

    pub dao: Dao,
}

impl AppCtx {
    pub fn new(cfg: AppCfg, exit_rx: Receiver<i64>, dao: Dao) -> Self {
        let search_recg_api = RecognitionApi::new(&cfg.search.recg_url);
        let trackdb_recg_api = RecognitionApi::new(&cfg.track_db.recg_url);

        Self {
            cfg,
            exit_rx,

            search_recg_api,
            trackdb_recg_api,
            dao,
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
