use deadqueue::unlimited::Queue;
use log::{debug, error, info};
use moka::future::Cache;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle as TkJoinHandle;

use fy_base::api::bm_api::{ApiFeatureQuality, RecognitionApi, SearchResPerson};
use fy_base::util::utils;

use crate::app_ctx::AppCtx;
use crate::error::AppError;
use crate::queue_item::FaceQueue;
use fy_base::api::upload_api::{MatchPerson, NotifyFaceQueueItem, QI};
use fy_base::util::service::Service;

pub struct FaceSearchService {
    num: i64,
    ctx: Arc<AppCtx>,
    queue: Arc<FaceQueue>,

    skip_search: bool,

    out: Arc<Queue<QI>>,
    api: RecognitionApi,

    last_cache_ts: Instant,
    dbs_cache: Cache<String, Vec<String>>,
}

impl FaceSearchService {
    pub fn new(num: i64, ctx: Arc<AppCtx>, queue: Arc<FaceQueue>, out: Arc<Queue<QI>>) -> Self {
        let api = RecognitionApi::new(ctx.cfg.api.recg_url.as_str());
        let skip_search = ctx.cfg.track.face.skip_search;

        let last_cache_ts = Instant::now();
        let dbs_cache = Cache::new(10);

        FaceSearchService {
            num,
            ctx,
            api,
            last_cache_ts,
            queue,
            skip_search,

            out,
            dbs_cache,
        }
    }

    async fn pop_batch(&mut self) -> Vec<NotifyFaceQueueItem> {
        let max = 4;
        utils::pop_queue_batch(&self.queue, max).await
    }

    fn fill_with_matchinfo(
        &self,
        items: &mut Vec<NotifyFaceQueueItem>,
        persons: &Option<Vec<Vec<SearchResPerson>>>,
    ) {
        let persons = match persons.as_ref() {
            Some(v) => v,
            None => {
                error!(
                    "error, FaceSearchWorker[{}], search result persons is none",
                    self.num
                );
                return;
            }
        };

        if persons.len() != items.len() {
            error!(
                "error, FaceSearchWorker[{}], items len:{} not equal result persons:{}",
                self.num,
                persons.len(),
                items.len()
            );
            return;
        }

        let count = items.len();
        for i in 0..count {
            let item = items.get_mut(i).unwrap();

            self.fill_one_person(item, persons.get(i));
        }
    }

    fn fill_one_person(
        &self,
        item: &mut NotifyFaceQueueItem,
        match_persons: Option<&Vec<SearchResPerson>>,
    ) {
        if match_persons.map_or(false, |x| x.is_empty()) {
            debug!(
                "FaceSearchWorker[{}], search facetrack:{}, has no match",
                self.num, item.uuid
            );
            return;
        }
        let match_persons = match_persons.unwrap();

        let matches = match_persons
            .iter()
            .map(|x| {
                debug!(
                    "FaceSearchWorker[{}], facetrack:{}, matched:{}, score:{}",
                    self.num, item.uuid, x.id, x.score
                );
                MatchPerson {
                    db_id: x.db.clone(),
                    uuid: x.id.clone(),
                    score: x.score,
                }
            })
            .collect();

        item.matches = Some(matches);
    }

    async fn get_dbs_from_api(&self) -> Result<Vec<String>, AppError> {
        let res = self.api.get_dbs().await?;
        if res.code != 0 {
            return Err(AppError::new(&format!(
                "get_dbs return code:{}, msg:{}",
                res.code, res.msg
            )));
        }

        let ids = match res.dbs {
            None => vec![],
            Some(v) => v,
        };
        debug!("FaceSearchService, get_dbs, len: {}", ids.len());
        Ok(ids)
    }

    async fn get_dbs(&mut self) -> Vec<String> {
        // 有数据，且没有过期的，返回
        // 查询，然后放入cache中

        let key = "dbs".to_string();
        let dbs = self.dbs_cache.get(&key);
        if let Some(v) = dbs {
            if self
                .last_cache_ts
                .elapsed()
                .lt(&Duration::from_secs(self.ctx.cfg.track.face.cache_ttl*60))
            {
                // 在ttl周期内，直接返回
                return v;
            }
        }

        let dbs = self.get_dbs_for_real().await;
        let now = Instant::now();
        self.last_cache_ts = now;
        self.dbs_cache.insert(key, dbs.clone()).await;
        dbs
    }

    async fn get_dbs_for_real(&self) -> Vec<String> {
        match self.get_dbs_from_api().await {
            Ok(v) => v,
            Err(e) => {
                error!("FaceSearchWorker, get_dbs_from_api, err: {:?}", e);
                vec![]
            }
        }
    }

    /// api 比对搜索，(有特征值, 并且dbs不为空)
    /// 无论处理成功或失败，都提交到mpsc中
    async fn process_batch(&mut self, mut items: Vec<NotifyFaceQueueItem>) {
        let tops = vec![self.ctx.cfg.track.face.search_top as i64];
        let thresholds = vec![self.ctx.cfg.track.face.search_threshold as i64];
        let dbs = if self.skip_search {
            vec![]
        } else {
            self.get_dbs().await
        };

        let mut persons = Vec::new();

        debug!(
            "FaceSearchWorker[{}], process_batch: {}",
            self.num,
            items.len()
        );

        items.iter_mut().for_each(|x| {
            let mut feas = Vec::new();
            x.notify.faces.iter_mut().for_each(|f| {
                if let Some(ref feature) = f.feature_buf {
                    let fea = base64::encode(&feature);
                    feas.push(ApiFeatureQuality {
                        feature: fea,
                        quality: f.quality,
                    });
                }
            });

            if !feas.is_empty() {
                // 不为空，才加入到 最后的比对中
                persons.push(feas);
            }
        });

        if self.skip_search || dbs.is_empty() || persons.len() != items.len() {
            // dbs 为空，或者 items中存在没有特征值的item，则不进行比对
            debug!("FaceSearchWorker[{}], skip search", self.num);
        } else {
            let ts_start = Instant::now();
            let search_res = self.api.search(dbs, tops, thresholds, persons).await;
            let ts_use = ts_start.elapsed();
            debug!(
                "FaceSearchWorker[{}], search api use: {} ms, batch size:{}",
                self.num,
                ts_use.as_millis(),
                items.len()
            );

            debug!(
                "FaceSearchWorker[{}], search_res: {:?}",
                self.num, search_res
            );

            match search_res {
                Ok(res) => {
                    if res.code == 0 {
                        // 填充 比对结果
                        self.fill_with_matchinfo(&mut items, &res.persons);
                    } else {
                        error!(
                            "error, FaceSearchWorker[{}], search, code:{}, msg:{}",
                            self.num, res.code, res.msg
                        );
                    }
                }
                Err(e) => {
                    error!("error, FaceSearchWorker[{}], search, {:?}", self.num, e);
                }
            }
        }

        // 放入 queue 中
        for v in items {
            self.out.push(QI::FT(Box::new(v)));
        }
    }
}

impl Service for FaceSearchService {
    fn run(self, rx: Receiver<i64>) -> TkJoinHandle<()> {
        let mut svc = self;
        let mut exit_rx = rx;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = exit_rx.changed() => {
                        info!("FaceSearchWorker[{}] recv exit",svc.num);
                        break;
                    }
                    items = svc.pop_batch() => {
                        svc.process_batch(items).await;
                    }
                }
            }
            info!("FaceSearchWorker[{}] exit", svc.num);
        })
    }
}
