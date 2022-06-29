use chrono::{  FixedOffset};

use std::sync::Arc;

use crate::dao::base_model::{ Facetrack};
use crate::error::AppError;

use sqlx::{MySql, Pool};

pub mod base_model;

#[derive(Clone)]
pub struct Dao {
    pub pool: Arc<Pool<MySql>>,
    pub tz: FixedOffset,
}

//--------------------------------

impl Dao {

    pub async fn save_facetrack(
        &self,
        facetrack: &Facetrack
    ) -> Result<u64, AppError> {
        let new_id = facetrack.insert(&self.pool,&self.tz).await?;

        Ok(new_id)
    }


}
