use crate::util::service::Service;
use tokio::sync::watch;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;

use log::{debug, info};

pub struct SignalService {
    pub exit_tx: watch::Sender<i64>,
}

impl SignalService {
    pub fn new(exit_tx: watch::Sender<i64>) -> Self {
        Self { exit_tx }
    }
}

impl Service for SignalService {
    fn run(self, _exit_rx: Receiver<i64>) -> JoinHandle<()> {
        let service = self;
        tokio::spawn(async move {
            shutdown_signal().await;
            let _ = service.exit_tx.send(100);
            info!("SignalService exit.");
        })
    }
}

#[cfg(unix)]
pub async fn shutdown_signal() {
    use std::io;
    use tokio::signal::unix::SignalKind;

    async fn terminate() -> io::Result<()> {
        let signal = tokio::signal::unix::signal(SignalKind::terminate());
        if let Err(e) = signal {
            debug!("error, signal, {:?}", e);
            return Err(e);
        }

        let mut signal = signal.unwrap();

        signal.recv().await;
        Ok(())
    }

    tokio::select! {
        t = terminate() => {
            debug!("recv unix terminate signal, {:?}",t);
        },
        _ = tokio::signal::ctrl_c() => {
            debug!("recv unix ctrl_c signal.")
        },
    }
    debug!("shutdown_signal, unix, end.");
}

#[cfg(windows)]
pub async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("faild to install CTRL+C handler");
    debug!("shutdown_signal, windows, end.");
}
