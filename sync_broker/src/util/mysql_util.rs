use std::error::Error as StdError;
use std::fmt::{Display, Formatter};
use std::time::Duration;

use chrono::LocalResult;
use chrono::prelude::*;
use sqlx::{Executor, MySql, Pool};
use sqlx::error::DatabaseError;
use sqlx::mysql::MySqlPoolOptions;

// 从驱动层(sqlx)读取的时间，被认为是UtC时间，实际不是
pub fn fix_read_dt(dt: &mut DateTime<Local>, db_offset: &FixedOffset) {
    let origin_dt = dt.with_timezone(&Utc);

    let db_local = db_offset
        .from_local_datetime(&origin_dt.naive_local())
        .unwrap();

    let dst_dt = db_local.with_timezone(&Local);
    *dt = dst_dt;
}

pub fn fix_read_dt_option(dt: &mut Option<DateTime<Local>>, db_offset: &FixedOffset) {
    if let Some(v) = dt {
        fix_read_dt(v, db_offset);
    }
}

// 驱动层(sqlx)会将DateTime<Local>类型，转成utc时间。对NaiveDateTime类型不做变动
pub fn fix_write_dt(dt: &DateTime<Local>, db_offset: &FixedOffset) -> NaiveDateTime {
    let db_local = dt.with_timezone(db_offset);
    db_local.naive_local()
}

pub fn fix_write_dt_option(
    dt: &Option<DateTime<Local>>,
    db_offset: &FixedOffset,
) -> Option<NaiveDateTime> {
    dt.map(|ref f| fix_write_dt(f, db_offset))
}

// 解析字符串得到时区 例如："+08:00"
pub fn parse_timezone(tz: &str) -> Result<FixedOffset, String> {
    let list: Vec<char> = tz.chars().collect();
    // println!("list: {:?}", list);
    if list.len() < 6 {
        return Err(format!("{} is invalid timezone", tz));
    }
    let east = match list[0] {
        '+' => true,
        '-' => false,
        _ => {
            return Err(format!("{} is invalid timezone", tz));
        }
    };

    let ts = match NaiveTime::parse_from_str(&tz[1..], "%H:%M") {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("{} is invalid timezone, {:?}", tz, e));
        }
    };

    let seconds = ts.hour() * 60 * 60 + ts.minute() * 60 + ts.second();
    if east {
        Ok(FixedOffset::east(seconds as i32))
    } else {
        Ok(FixedOffset::west(seconds as i32))
    }
}

//---------------------------------------------------
pub async fn init_mysql_pool(
    db_url: &str,
    tz: &str,
    max_conn: u32,
    min_conn: u32,
) -> Result<Pool<MySql>, sqlx::Error> {
    let tz_str = tz.to_string();

    let pool = MySqlPoolOptions::new()
        .max_connections(max_conn)
        .min_connections(min_conn)
        .idle_timeout(Duration::from_secs(60 * 10))
        .after_connect(move |conn| {
            let tz_str = tz_str.clone();
            Box::pin(async move {
                let sql = format!("set time_zone = '{}'", tz_str);
                conn.execute(sql.as_str()).await?;
                Ok(())
            })
        })
        .connect(db_url)
        .await?;
    Ok(pool)
}

//------------------------------------------------------
pub fn parse_local_time_str(ts: &str, fmt: &str) -> Result<DateTime<Local>, MySqxErr> {
    let nt = match NaiveDateTime::parse_from_str(ts, fmt) {
        Ok(v) => v,
        Err(e) => {
            return Err(MySqxErr(e.to_string()));
        }
    };

    let dt = match (Local).from_local_datetime(&nt) {
        LocalResult::None => {
            return Err(MySqxErr(format!("invalid {}", ts)));
        }
        LocalResult::Single(v) => v,
        LocalResult::Ambiguous(_, _) => {
            return Err(MySqxErr(format!("invalid {}", ts)));
        }
    };

    Ok(dt)
}

#[derive(Debug)]
pub struct MySqxErr(pub String);

impl Display for MySqxErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl StdError for MySqxErr {}

impl DatabaseError for MySqxErr {
    fn message(&self) -> &str {
        self.0.as_str()
    }

    fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self
    }

    fn as_error_mut(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
        self
    }

    fn into_error(self: Box<Self>) -> Box<dyn StdError + Send + Sync + 'static> {
        Box::new(self)
    }
}
