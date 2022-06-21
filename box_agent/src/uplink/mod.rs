pub mod uplink_api;
pub mod upload;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseData {
    pub status: i32,
    pub message: Option<String>,
}
