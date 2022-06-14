use tokio::sync::watch::Receiver;

use crate::app_cfg::AppCfg;
use crate::dao::Dao;
use crate::util::service::SignalProduce;

//---------------------

pub struct AppCtx {
    pub cfg: AppCfg,
    pub exit_rx: Receiver<i64>,
    pub dao: Dao,
}

impl AppCtx {
    pub fn new(app_cfg: AppCfg, exit_rx: Receiver<i64>, dao: Dao) -> Self {
        Self {
            cfg: app_cfg,
            exit_rx,
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
