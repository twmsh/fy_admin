use chrono::LocalResult;
use chrono::prelude::*;
use deadqueue::unlimited::Queue;
use std::path::Path;
use url::form_urlencoded;

pub const DATETIME_FMT_SHORT: &str = "%Y-%m-%d %H:%M:%S";
pub const DATETIME_FMT_LONG: &str = "%Y-%m-%d %H:%M:%S%.3f";


pub fn parse_localtime_str(ts: &str, fmt: &str) -> Result<DateTime<Local>, String> {
    let dt = match NaiveDateTime::parse_from_str(ts, fmt) {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("{}", e));
        }
    };

    let rst = Local.from_local_datetime(&dt);
    if let LocalResult::Single(v) = rst {
        Ok(v)
    } else {
        Err(format!("invalid {:?}", rst))
    }
}

pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

pub fn add_url_query(url: &str, name: &str, value: &str) -> String {
    let query = form_urlencoded::Serializer::new(String::new())
        .append_pair(name, value).finish();

    if !url.contains('?') {
        return format!("{}?{}", url, query);
    }

    if url.ends_with('?') || url.ends_with('&') {
        return format!("{}{}", url, query);
    }

    return format!("{}&{}", url, query);
}

/// 队列中数据时，一次最多取出limit数据, 没有数据时候阻塞，取一条数据返回
pub async fn pop_queue_batch<T>(queue: &Queue<T>, max: usize) -> Vec<T> {
    let mut size = 0_usize;
    let mut list = Vec::new();

    while let Some(v) = queue.try_pop() {
        list.push(v);
        size += 1;
        if size == max {
            break;
        }
    }

    if list.is_empty() {
        let v = queue.pop().await;
        list.push(v);
    }
    list
}