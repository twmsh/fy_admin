use std::cmp::Ordering;
use std::sync::Arc;
use std::time::Duration;

use chrono::prelude::*;
use dashmap::{DashMap, DashSet};
use deadqueue::unlimited::Queue;
use tracing::{debug, error, info, warn};

use tokio::sync::mpsc::{self, Receiver as TkReceiver, Sender as TkSender};
use tokio::sync::watch::Receiver;
use tokio::sync::Mutex;
use tokio::task::JoinHandle as TkJoinHandle;
use tokio::time::error::Error as TkTimeError;
use tokio_util::time::delay_queue::{DelayQueue, Expired};

use fy_base::api::bm_api::FaceNotifyParams;
use fy_base::util::delay_queue::DelayQueueChan;
use fy_base::util::service::Service;

use crate::app_ctx::AppCtx;

use super::{SerialPool, SpHolder};
use crate::error::AppResult;
use crate::queue_item::FaceQueue;
use fy_base::api::upload_api::NotifyFaceQueueItem;

// ------------------- structs -------------------
pub struct Track {
    uuid: String,
    ts: DateTime<Local>,

    ready_flag: bool,

    notify: FaceNotifyParams,
}

pub enum TrackEvent {
    New,
    APPEND(Box<Track>),
    DELAY(String),
}

pub struct FaceHandler {
    sender: TkSender<String>,
}

pub struct FaceNotifyService {
    ctx: Arc<AppCtx>,
    queue: Arc<Queue<NotifyFaceQueueItem>>,

    spool: SerialPool,
    ready_tx: TkSender<(String, Duration)>,
    ready_rx: TkReceiver<Result<Expired<String>, TkTimeError>>,

    clean_tx: TkSender<(String, Duration)>,
    clean_rx: TkReceiver<Result<Expired<String>, TkTimeError>>,

    track_map: DashMap<String, Arc<SpHolder>>,
    track_id_map: DashSet<String>,

    next_rx: TkReceiver<String>,

    out: Arc<Queue<NotifyFaceQueueItem>>,
}

// ------------------- impls -------------------
impl FaceNotifyService {
    pub fn new(ctx: Arc<AppCtx>, queue: Arc<FaceQueue>, out: Arc<FaceQueue>) -> Self {
        let (next_tx, next_rx) = mpsc::channel(100);

        let handler = FaceHandler { sender: next_tx };

        let spool = SerialPool::new(handler);

        let dq_ready = DelayQueue::new();
        let (ready_tx, ready_rx) = dq_ready.channel();

        let dq_clean = DelayQueue::new();
        let (clean_tx, clean_rx) = dq_clean.channel();

        let track_map = DashMap::new();
        let track_id_map = DashSet::new();

        FaceNotifyService {
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

    fn check_recv_mode(&self, item: &NotifyFaceQueueItem) -> bool {
        let count = self.ctx.cfg.track.face.count as usize;
        let quality = self.ctx.cfg.track.face.quality;

        // 图片数量和质量满足要求,含有特征值
        let cc = item.notify.faces.iter().fold(0_usize, |acc, x| {
            if x.quality > quality && x.feature_buf.is_some() {
                acc + 1
            } else {
                acc
            }
        });

        cc >= count
    }

    async fn process_item(&mut self, item: NotifyFaceQueueItem) {
        debug!("FaceNotifyProcSvc, process_item:{:?}", item.uuid);

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
            warn!(
                "warn, FaceNotifyProcSvc, {}, has put to next, skip it",
                uuid
            );
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
            let clean_timeout = self.ctx.cfg.track.face.clear_delay;
            let _ = self
                .clean_tx
                .send((uuid.clone(), Duration::from_millis(clean_timeout)))
                .await;
            debug!("put to clean_queue, {}", uuid);

            // 如果not ready, 加入 timeout队列
            if !ready {
                let ready_timeout = self.ctx.cfg.track.face.ready_delay;
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

            let mut item = NotifyFaceQueueItem {
                uuid: uuid.clone(),
                notify,
                ts,
                matches: None,
            };

            sort_track_faces(&mut item);

            self.out.push(item);
        }
    }
}

// ------------------- impl Service -------------------
impl Service for FaceNotifyService {
    fn run(self, rx: Receiver<i64>) -> TkJoinHandle<()> {
        let mut svc = self;
        let mut exit_rx = rx;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = exit_rx.changed() => {

                            info!("FaceNotifyProcSvc recv exit");
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
            info!("FaceNotifyProcSvc exit.");
        })
    }
}

// ------------------- impl Handler -------------------
impl FaceHandler {
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
                    // 替换背景图，增加图片
                    data.notify.background = track.notify.background;
                    data.notify.faces.append(&mut track.notify.faces);
                }
                TrackEvent::DELAY(_) => {
                    delayed = true;
                }
            }
        }

        // 新建
        if newed {
            debug!("FaceHandler, {} is newed", data.uuid);
        }

        // 更新
        if !newed && appended {
            debug!("FaceHandler, {} is appended", data.uuid);
        }

        if appended || delayed {
            data.ready_flag = true;
        }
        let ready_new = data.ready_flag;

        if (newed && ready_old) || (!ready_old && ready_new) {
            // 交给后续队列处理
            if let Err(e) = self.put_to_next(&data).await {
                error!("error, FaceHandler put_to_next, {}, {:?}", data.uuid, e);
            } else {
                debug!("FaceHandler put_to_next ok, {}", data.uuid);
            }
        }
    }
}

//-----------------------
// 将facetrack中 face按照抓拍+有特征值的原则排序，
// file name也修改，不重复，没有特征值的face排在后面
fn sort_track_faces(item: &mut NotifyFaceQueueItem) {
    item.notify.faces.sort_by(|a, b| {
        // 先比较 是否有fea,再比较 fram_num
        if a.feature_file.is_some() && b.feature_file.is_none() {
            return Ordering::Less;
        }

        if a.feature_file.is_none() && b.feature_file.is_some() {
            return Ordering::Greater;
        }

        a.frame_num.cmp(&b.frame_num)
    });

    // 修改 aligned_file , display_file, feature_file
    for (id, face) in item.notify.faces.iter_mut().enumerate() {
        face.aligned_file = format!("align_{}.bmp", id + 1);
        face.display_file = format!("display_{}.bmp", id + 1);
        if let Some(ref mut v) = face.feature_file {
            *v = format!("feature_{}.data", id + 1);
        }
    }
}
