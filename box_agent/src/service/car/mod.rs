pub mod car_notify;

//-----------------------------

use std::sync::Arc;
use tokio::sync::Mutex;

use super::car::car_notify::{CarHandler, Track, TrackEvent};

//-----------------------------

type SpHandler = CarHandler;
type SpEvent = TrackEvent;
type SpData = Track;

//-----------------------------

pub struct HolderState {
    pub running: bool,
    pub events: Vec<SpEvent>,
}

pub struct SpHolder {
    pub data: Arc<Mutex<SpData>>,
    pub state: Mutex<HolderState>,
}

pub struct SerialPool {
    pub handler: Arc<SpHandler>,
}

//------------------------------
impl SpHolder {
    pub fn new(data: SpData) -> SpHolder {
        SpHolder {
            data: Arc::new(Mutex::new(data)),
            state: Mutex::new(HolderState {
                running: false,
                events: Vec::new(),
            }),
        }
    }
}

impl SerialPool {
    pub fn new(h: SpHandler) -> SerialPool {
        SerialPool {
            handler: Arc::new(h),
        }
    }

    /// 将holder中的events处理完
    async fn drain_event(handler: Arc<SpHandler>, holder: Arc<SpHolder>) {
        // println!("enter drain_event .");
        loop {
            let events_copy;
            {
                let mut guard = holder.state.lock().await;
                let size = guard.events.len();
                // println!("events.len: {}", size);
                if size == 0 {
                    guard.running = false;
                    break;
                }
                events_copy = guard.events.drain(..).collect();
            }

            let data = holder.data.clone();
            handler.process(data, events_copy).await;
        }
    }

    /// 分发/处理 事件
    /// dispatch 密集调用时候，导致 drain_event 没机会运行,
    /// dispatch调用方可以 tokio::task::yield_now().await
    pub async fn dispatch(&self, holder: Arc<SpHolder>, event: SpEvent) {
        let mut guard = holder.state.lock().await;
        guard.events.push(event);

        // if not running
        if !guard.running {
            let ha = self.handler.clone();
            let ho = holder.clone();

            // println!("will spawn ...");

            tokio::spawn(async move {
                // println!("will drain_event .");
                Self::drain_event(ha, ho).await;
            });
            guard.running = true;
        }
    }
}
