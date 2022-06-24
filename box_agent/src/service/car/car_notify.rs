use std::sync::Arc;
use std::time::Duration;

use chrono::prelude::*;
use dashmap::{DashMap, DashSet};
use deadqueue::unlimited::Queue;
use log::{debug, error, info, warn};

use tokio::sync::mpsc::{self, Receiver as TkReceiver, Sender as TkSender};
use tokio::sync::watch::Receiver;
use tokio::sync::Mutex;
use tokio::task::JoinHandle as TkJoinHandle;
use tokio::time::error::Error as TkTimeError;
use tokio_util::time::delay_queue::{DelayQueue, Expired};

use fy_base::api::bm_api::CarNotifyParams;
use fy_base::util::delay_queue::DelayQueueChan;
use fy_base::util::service::Service;

use crate::app_ctx::AppCtx;

use crate::error::AppResult;
use crate::queue_item::CarQueue;
use fy_base::api::upload_api::{NotifyCarQueueItem, QI};

use super::{SerialPool, SpHolder};

// ------------------- structs -------------------
pub struct Track {
    uuid: String,
    ts: DateTime<Local>,

    ready_flag: bool,

    notify: CarNotifyParams,
}

pub enum TrackEvent {
    New,
    APPEND(Box<Track>),
    DELAY(String),
}

pub struct CarHandler {
    sender: TkSender<String>,
}

pub struct CarNotifyService {
    ctx: Arc<AppCtx>,
    queue: Arc<Queue<NotifyCarQueueItem>>,

    spool: SerialPool,
    ready_tx: TkSender<(String, Duration)>,
    ready_rx: TkReceiver<Result<Expired<String>, TkTimeError>>,

    clean_tx: TkSender<(String, Duration)>,
    clean_rx: TkReceiver<Result<Expired<String>, TkTimeError>>,

    track_map: DashMap<String, Arc<SpHolder>>,
    track_id_map: DashSet<String>,

    next_rx: TkReceiver<String>,

    out: Arc<Queue<QI>>,
}

// ------------------- impls -------------------
impl CarNotifyService {
    pub fn new(ctx: Arc<AppCtx>, queue: Arc<CarQueue>, out: Arc<Queue<QI>>) -> Self {
        let (next_tx, next_rx) = mpsc::channel(100);

        let handler = CarHandler { sender: next_tx };

        let spool = SerialPool::new(handler);

        let dq_ready = DelayQueue::new();
        let (ready_tx, ready_rx) = dq_ready.channel();

        let dq_clean = DelayQueue::new();
        let (clean_tx, clean_rx) = dq_clean.channel();

        let track_map = DashMap::new();
        let track_id_map = DashSet::new();

        CarNotifyService {
            ctx,
            queue,
            spool,
            ready_tx,
            ready_rx,
            clean_tx,
            clean_rx,
            track_map,
            track_id_map,
            next_rx,
            out,
        }
    }

    // 检查车牌号的每一个bit的conf
    fn check_plate_conf(&self, item: &NotifyCarQueueItem) -> bool {
        let conf = self.ctx.cfg.track.car.conf;
        if item.notify.plate_info.is_none() {
            return false;
        }
        let plate_info = item.notify.plate_info.as_ref().unwrap();
        if plate_info.bits.is_none() {
            return false;
        }

        let plate_bits = plate_info.bits.as_ref().unwrap();
        for bit in plate_bits {
            if let Some(bit_value) = bit.get(0) {
                if bit_value.conf < conf {
                    return false;
                }
            }
        }

        true
    }

    // 有车牌且conf大于某个值 + 车身图大于count
    fn check_recv_mode(&self, item: &NotifyCarQueueItem) -> bool {
        let count = self.ctx.cfg.track.car.count as usize;

        let check_plate = self.check_plate_conf(item);
        let img_count = item.notify.vehicles.len();

        check_plate && (img_count >= count)
    }

    async fn process_item(&mut self, item: NotifyCarQueueItem) {
        debug!("CarNotifyProcSvc, process_item:{:?}", item.uuid);

        let uuid = item.uuid.clone();
        let ready = self.check_recv_mode(&item);
        let track = Track {
            ready_flag: ready,
            uuid: item.uuid,
            ts: item.ts,

            notify: item.notify,
        };

        // 如果在track_id_map中有，但是在 track_map中没有，说明该track的数据已经传递到下一步，这个track的后续数据忽略
        if self.track_id_map.contains(&uuid) && !self.track_map.contains_key(&uuid) {
            warn!("warn, CarNotifyProcSvc, {}, has put to next, skip it", uuid);
            return;
        }

        if let Some(holder) = self.track_map.get(uuid.as_str()) {
            // 已经在 spmap中, dispath it
            self.spool
                .dispatch(holder.clone(), TrackEvent::APPEND(Box::new(track)))
                .await;
            debug!("already in spmap, {}", uuid);
        } else {
            // 新的track, 不在spmap中, dispath it
            debug!("create to spmap, {}", uuid);
            let holder = Arc::new(SpHolder::new(track));
            self.track_map.insert(uuid.clone(), holder.clone());
            self.track_id_map.insert(uuid.clone());
            self.spool.dispatch(holder.clone(), TrackEvent::New).await;

            // 放入清除/延时队列中
            let clean_timeout = self.ctx.cfg.track.car.clear_delay;
            let _ = self
                .clean_tx
                .send((uuid.clone(), Duration::from_millis(clean_timeout)))
                .await;
            debug!("put to clean_queue, {}", uuid);

            // 如果not ready, 加入 timeout队列
            if !ready {
                let ready_timeout = self.ctx.cfg.track.car.ready_delay;
                let _ = self
                    .ready_tx
                    .send((uuid.clone(), Duration::from_millis(ready_timeout)))
                    .await;
                debug!("put to timeout_queue, {}", uuid);
            }
        }
    }

    async fn process_ready_timeout(&self, uuid: String) {
        debug!("process ready_timeout: {}", uuid);
        if let Some(holder) = self.track_map.get(&uuid) {
            // 已经在map中
            let holder = holder.value().clone();
            self.spool.dispatch(holder, TrackEvent::DELAY(uuid)).await;
        } else if !self.track_id_map.contains(&uuid) {
            error!("error, uuid not in map, {}", uuid);
        }
    }

    async fn process_clean_timeout(&self, uuid: String) {
        debug!("process clean_timeout: {}", uuid);
        let _ = self.track_id_map.remove(&uuid);
        debug!("remove uuid: {}", uuid);
    }

    // 从track_map中删除，放入下一个队列中
    async fn process_put_next(&self, uuid: String) {
        debug!("process put_next: {}", uuid);
        let track = self.track_map.remove(&uuid);
        if let Some((_, track)) = track {
            let (ts, notify) = {
                let guard = track.data.lock().await;
                (guard.ts, guard.notify.clone())
            };

            let item = NotifyCarQueueItem {
                uuid: uuid.clone(),
                notify,
                ts,
            };

            self.out.push(QI::CT(Box::new(item)));
        }
    }
}

// ------------------- impl Service -------------------
impl Service for CarNotifyService {
    fn run(self, rx: Receiver<i64>) -> TkJoinHandle<()> {
        let mut svc = self;
        let mut exit_rx = rx;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = exit_rx.changed() => {

                            info!("CarNotifyProcSvc recv exit");
                            break;

                    }
                    item = svc.queue.pop() => {
                        svc.process_item(item).await;
                    }
                    Some(Ok(expired)) = svc.ready_rx.recv() => {
                        svc.process_ready_timeout(expired.into_inner()).await;
                    }
                    Some(Ok(expired)) = svc.clean_rx.recv() => {
                        svc.process_clean_timeout(expired.into_inner()).await;
                    }
                    Some(track_id) = svc.next_rx.recv() => {
                        svc.process_put_next(track_id).await;
                    }
                }
            }
            info!("CarNotifyProcSvc exit.");
        })
    }
}

// ------------------- impl Handler -------------------
impl CarHandler {
    /// 查询 source
    /// 生成 FtQI 放入后续队列中
    async fn put_to_next(&self, track: &Track) -> AppResult<()> {
        let _ = self.sender.send(track.uuid.clone()).await;
        Ok(())
    }

    pub async fn process(&self, holder_data: Arc<Mutex<Track>>, events: Vec<TrackEvent>) {
        let mut newed = false;
        let mut appended = false;
        let mut delayed = false;
        // let ready_old;

        let mut data = holder_data.lock().await;
        let ready_old = data.ready_flag;

        // debug!("sp process: {}, events len: {}, ready_old:{}", data.uuid, events.len(), ready_old);
        for event in events.into_iter() {
            match event {
                TrackEvent::New => {
                    newed = true;
                }
                TrackEvent::APPEND(mut track) => {
                    appended = true;
                    // 替换背景图，增加图片，车牌图，属性
                    data.notify.background = track.notify.background;
                    data.notify.vehicles.append(&mut track.notify.vehicles);
                    if track.notify.plate_info.is_some() {
                        data.notify.plate_info = track.notify.plate_info;
                    }
                    if track.notify.props.is_some() {
                        data.notify.props = track.notify.props;
                    }
                }
                TrackEvent::DELAY(_) => {
                    delayed = true;
                }
            }
        }

        // 新建
        if newed {
            debug!("CarHandler, {} is newed", data.uuid);
        }

        // 更新
        if !newed && appended {
            debug!("CarHandler, {} is appended", data.uuid);
        }

        if appended || delayed {
            data.ready_flag = true;
        }
        let ready_new = data.ready_flag;

        if (newed && ready_old) || (!ready_old && ready_new) {
            // 交给后续队列处理
            if let Err(e) = self.put_to_next(&data).await {
                error!("error, CarHandler put_to_next, {}, {:?}", data.uuid, e);
            } else {
                debug!("CarHandler put_to_next ok, {}", data.uuid);
            }
        }
    }
}
