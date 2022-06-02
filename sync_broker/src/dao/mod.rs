use std::sync::Arc;

use sqlx::{MySql, Pool};

pub struct Dao {
    pub pool: Arc<Pool<MySql>>,
}