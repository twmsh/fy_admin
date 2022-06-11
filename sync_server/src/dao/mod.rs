use std::ops::Deref;
use std::sync::Arc;
use chrono::{DateTime, FixedOffset, Local};

use sqlx::{MySql, Pool};
use crate::dao::base_model::{BaseBox, BaseCamera, BaseCameraDel, BaseDb, BaseDbDel};
use crate::error::AppError;
use crate::util::mysql_util;

pub mod base_model;

pub struct Dao {
    pub pool: Arc<Pool<MySql>>,
    pub tz: FixedOffset,
}


//--------------------------------
// 从 A 表取 100条，从A_del表取100条，然后按照modify_time排序，取前100条

impl Dao {
    pub async fn find_box(&self, hw_id: String) -> Result<Option<BaseBox>, AppError> {
        let sql = "select * from base_box where hw_id = ?";
        let mut obj = sqlx::query_as::<_, BaseBox>(sql)
            .bind(hw_id)
            .fetch_optional(self.pool.deref()).await?;

        match obj {
            None => {}
            Some(ref mut v) => {
                mysql_util::fix_read_dt(&mut v.create_time, &self.tz);
                mysql_util::fix_read_dt(&mut v.modify_time, &self.tz);
            }
        }

        Ok(obj)
    }


    pub async fn get_db_list(&self, last_update: DateTime<Local>, limit: u32)
                             -> Result<Vec<BaseDb>, AppError> {
        let sql = "select * from base_db where modify_time > ? order by modify_time asc limit ?";
        let last_update = mysql_util::fix_write_dt(&last_update, &self.tz);

        let mut list = sqlx::query_as::<_, BaseDb>(sql)
            .bind(last_update)
            .bind(limit)
            .fetch_all(self.pool.deref()).await?;

        for v in list.iter_mut() {
            mysql_util::fix_read_dt(&mut v.modify_time, &self.tz);
        }

        Ok(list)
    }

    pub async fn get_dbdel_list(&self, last_update: DateTime<Local>, limit: u32)
                                -> Result<Vec<BaseDbDel>, AppError> {
        let sql = "select * from base_db_del where modify_time > ? order by modify_time asc limit ?";
        let last_update = mysql_util::fix_write_dt(&last_update, &self.tz);

        let mut list = sqlx::query_as::<_, BaseDbDel>(sql)
            .bind(last_update)
            .bind(limit)
            .fetch_all(self.pool.deref()).await?;

        for v in list.iter_mut() {
            mysql_util::fix_read_dt(&mut v.modify_time, &self.tz);
        }

        Ok(list)
    }

    //------------------------------------------------
    pub async fn get_camera_list(&self, last_update: DateTime<Local>, limit: u32)
                                 -> Result<Vec<BaseCamera>, AppError> {
        let sql = "select * from base_camera where modify_time > ? order by modify_time asc limit ?";
        let last_update = mysql_util::fix_write_dt(&last_update, &self.tz);

        let mut list = sqlx::query_as::<_, BaseCamera>(sql)
            .bind(last_update)
            .bind(limit)
            .fetch_all(self.pool.deref()).await?;

        for v in list.iter_mut() {
            mysql_util::fix_read_dt(&mut v.modify_time, &self.tz);
        }

        Ok(list)
    }

    pub async fn get_cameradel_list(&self, last_update: DateTime<Local>, limit: u32)
                                    -> Result<Vec<BaseCameraDel>, AppError> {
        let sql = "select * from base_camera_del where modify_time > ? order by modify_time asc limit ?";
        let last_update = mysql_util::fix_write_dt(&last_update, &self.tz);

        let mut list = sqlx::query_as::<_, BaseCameraDel>(sql)
            .bind(last_update)
            .bind(limit)
            .fetch_all(self.pool.deref()).await?;

        for v in list.iter_mut() {
            mysql_util::fix_read_dt(&mut v.modify_time, &self.tz);
        }

        Ok(list)
    }
}

