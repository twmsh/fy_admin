use chrono::{DateTime, FixedOffset, Local};
use std::ops::Deref;
use std::sync::Arc;

use crate::dao::base_model::{BaseBox, BaseCamera, BaseCameraDel, BaseDb, BaseDbDel, BaseFeaDel};
use crate::error::AppError;
use crate::service::web::model::BaseFeaMapRow;
use fy_base::util::mysql_util;
use sqlx::{MySql, Pool};

pub mod base_model;

#[derive(Clone)]
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
            .fetch_optional(self.pool.deref())
            .await?;

        match obj {
            None => {}
            Some(ref mut v) => {
                mysql_util::fix_read_dt(&mut v.create_time, &self.tz);
                mysql_util::fix_read_dt(&mut v.modify_time, &self.tz);
            }
        }

        Ok(obj)
    }

    pub async fn get_db_list(
        &self,
        last_update: DateTime<Local>,
        limit: u32,
    ) -> Result<Vec<BaseDb>, AppError> {
        let sql = "select * from base_db where modify_time > ? order by modify_time asc limit ?";
        let last_update = mysql_util::fix_write_dt(&last_update, &self.tz);

        let mut list = sqlx::query_as::<_, BaseDb>(sql)
            .bind(last_update)
            .bind(limit)
            .fetch_all(self.pool.deref())
            .await?;

        for v in list.iter_mut() {
            mysql_util::fix_read_dt(&mut v.modify_time, &self.tz);
            mysql_util::fix_read_dt(&mut v.create_time, &self.tz);
        }

        Ok(list)
    }

    pub async fn get_dbdel_list(
        &self,
        last_update: DateTime<Local>,
        limit: u32,
    ) -> Result<Vec<BaseDbDel>, AppError> {
        let sql =
            "select * from base_db_del where modify_time > ? order by modify_time asc limit ?";
        let last_update = mysql_util::fix_write_dt(&last_update, &self.tz);

        let mut list = sqlx::query_as::<_, BaseDbDel>(sql)
            .bind(last_update)
            .bind(limit)
            .fetch_all(self.pool.deref())
            .await?;

        for v in list.iter_mut() {
            mysql_util::fix_read_dt(&mut v.modify_time, &self.tz);
            mysql_util::fix_read_dt(&mut v.create_time, &self.tz);
        }

        Ok(list)
    }

    //------------------------------------------------
    pub async fn get_camera_list(
        &self,
        last_update: DateTime<Local>,
        box_hwid: &str,
        limit: u32,
    ) -> Result<Vec<BaseCamera>, AppError> {
        let sql = "select * from base_camera where box_hwid = ? and modify_time > ? order by modify_time asc limit ?";
        let last_update = mysql_util::fix_write_dt(&last_update, &self.tz);

        let mut list = sqlx::query_as::<_, BaseCamera>(sql)
            .bind(box_hwid)
            .bind(last_update)
            .bind(limit)
            .fetch_all(self.pool.deref())
            .await?;

        for v in list.iter_mut() {
            mysql_util::fix_read_dt(&mut v.modify_time, &self.tz);
            mysql_util::fix_read_dt(&mut v.create_time, &self.tz);
        }

        Ok(list)
    }

    pub async fn get_cameradel_list(
        &self,
        last_update: DateTime<Local>,
        box_hwid: &str,
        limit: u32,
    ) -> Result<Vec<BaseCameraDel>, AppError> {
        let sql = "select * from base_camera_del where box_hwid = ? and modify_time > ? order by modify_time asc limit ?";
        let last_update = mysql_util::fix_write_dt(&last_update, &self.tz);

        let mut list = sqlx::query_as::<_, BaseCameraDel>(sql)
            .bind(box_hwid)
            .bind(last_update)
            .bind(limit)
            .fetch_all(self.pool.deref())
            .await?;

        for v in list.iter_mut() {
            mysql_util::fix_read_dt(&mut v.create_time, &self.tz);
            mysql_util::fix_read_dt(&mut v.modify_time, &self.tz);
        }

        Ok(list)
    }

    // unused
    /*
    pub async fn get_fea_map(&self, uuid: &str) -> Result<Vec<BaseFeaMap>, AppError> {
        let sql = "select * from base_fea_map where uuid = ? order by face_id asc ";

        let mut list = sqlx::query_as::<_, BaseFeaMap>(sql)
            .bind(uuid)
            .fetch_all(self.pool.deref())
            .await?;

        for v in list.iter_mut() {
            mysql_util::fix_read_dt(&mut v.modify_time, &self.tz);
            mysql_util::fix_read_dt(&mut v.create_time, &self.tz);
        }

        Ok(list)
    }
    */

    pub async fn get_feadel_list(
        &self,
        last_update: DateTime<Local>,
        limit: u32,
    ) -> Result<Vec<BaseFeaDel>, AppError> {
        let sql =
            "select * from base_fea_del where modify_time > ? order by modify_time asc limit ?";
        let last_update = mysql_util::fix_write_dt(&last_update, &self.tz);

        let mut list = sqlx::query_as::<_, BaseFeaDel>(sql)
            .bind(last_update)
            .bind(limit)
            .fetch_all(self.pool.deref())
            .await?;

        for v in list.iter_mut() {
            mysql_util::fix_read_dt(&mut v.modify_time, &self.tz);
            mysql_util::fix_read_dt(&mut v.create_time, &self.tz);
        }

        Ok(list)
    }

    pub async fn get_feamap_row_list(
        &self,
        last_update: DateTime<Local>,
        limit: u32,
    ) -> Result<Vec<BaseFeaMapRow>, AppError> {
        /*
                select a.id, a.uuid,a.modify_time, b.face_id,b.feature from (
            select * from base_fea where modify_time > '2022-06-11 16:53:02.000'
            ORDER BY modify_time asc limit 3
        ) a  LEFT JOIN base_fea_map b on a.uuid = b.uuid where b.id is NOT NULL
        ORDER BY a.modify_time, a.uuid
                 */

        let sql = r#"select a.id,a.db_uuid,a.uuid,b.face_id,b.feature,b.quality,a.modify_time from (
	select * from base_fea where modify_time > ?
	ORDER BY modify_time asc limit ?
    ) a  LEFT JOIN base_fea_map b on a.uuid = b.uuid where b.id is NOT NULL
    ORDER BY a.modify_time, a.uuid
        "#;

        let last_update = mysql_util::fix_write_dt(&last_update, &self.tz);

        let mut list = sqlx::query_as::<_, BaseFeaMapRow>(sql)
            .bind(last_update)
            .bind(limit)
            .fetch_all(self.pool.deref())
            .await?;

        for v in list.iter_mut() {
            mysql_util::fix_read_dt(&mut v.modify_time, &self.tz);
        }

        Ok(list)
    }


    pub async fn update_latest_online(
        &self,
        hw_id: &str,
        ts: DateTime<Local>,
    ) -> Result<bool, AppError> {
        let sql = "update base_box set latest_online = ? where hw_id = ?";

        let ts = mysql_util::fix_write_dt(&ts, &self.tz);
        let rst = sqlx::query(sql)
            .bind(ts)
            .bind(hw_id)
            .execute(self.pool.deref()).await?;

        Ok(rst.rows_affected() == 1)
    }

    pub async fn clean_boxlog(
        &self,
        ts: DateTime<Local>,
    ) -> Result<u64, AppError> {
        let sql = "delete from base_box_log where create_time < ?";

        let ts = mysql_util::fix_write_dt(&ts, &self.tz);
        let rst = sqlx::query(sql)
            .bind(ts)
            .execute(self.pool.deref()).await?;

        Ok(rst.rows_affected())
    }
}
