use std::sync::Arc;
use chrono::FixedOffset;

use sqlx::{MySql, Pool};

pub mod base_model;

pub struct Dao {
    pub pool: Arc<Pool<MySql>>,
    pub tz: FixedOffset,
}