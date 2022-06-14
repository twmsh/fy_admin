use std::ops::Deref;
use std::sync::Arc;

use log::info;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;

pub trait Service {
    fn run(self, exit_rx: Receiver<i64>) -> JoinHandle<()>;
}

pub trait SignalProduce {
    fn clone_receiver(&self) -> Receiver<i64>;
}

impl<T: SignalProduce> SignalProduce for Arc<T> {
    fn clone_receiver(&self) -> Receiver<i64> {
        self.deref().clone_receiver()
    }
}

pub struct ServiceRepo<T: SignalProduce> {
    pub signaler: T,
    pub handles: Vec<JoinHandle<()>>,
}

impl<T: SignalProduce> ServiceRepo<T> {
    pub fn new(signaler: T) -> Self {
        Self {
            signaler,
            handles: Vec::new(),
        }
    }

    pub fn start_service<S: Service>(&mut self, service: S) {
        let exit_rx = self.signaler.clone_receiver();
        self.handles.push(service.run(exit_rx));
    }

    pub async fn join(self) {
        for h in self.handles {
            let _ = h.await;
            info!("service handles joined.");
        }
    }
}
