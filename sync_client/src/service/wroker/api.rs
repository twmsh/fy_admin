
use std::time::Duration;
use http::StatusCode;
use reqwest::{Client, Error as Reqwest_Error};
use reqwest::header;
use bytes::Buf;

use serde::{de::DeserializeOwned};
use serde_json::{Error as Serde_Error};

use super::{ResponseData, Camera};

use fy_base::util::utils;
use crate::bm_sync::Person;

//-----------------------
#[derive(Debug)]
pub enum ApiError {
    NetErr(String),
    JsonErr(String),
}

pub type ApiResult<T> = std::result::Result<T, ApiError>;

//------------------------ ApiError ------------------------
impl ApiError {
    pub fn net_err(s: &str) -> Self {
        ApiError::NetErr(s.to_string())
    }
    pub fn json_err(s: &str) -> Self {
        ApiError::JsonErr(s.to_string())
    }
}

impl From<Serde_Error> for ApiError {
    fn from(e: Serde_Error) -> Self {
        ApiError::json_err(&format!("{:?}", e))
    }
}

impl From<Reqwest_Error> for ApiError {
    fn from(e: Reqwest_Error) -> Self {
        ApiError::net_err(&format!("{:?}", e))
    }
}
//---------------------

pub struct Api {
    pub client: Client,
}

impl Default for Api {
    fn default() -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, header::HeaderValue::from_static("sync_client-api/1.0"));
        headers.insert(header::ACCEPT_ENCODING, header::HeaderValue::from_static("gzip, deflate"));
        headers.insert(header::ALLOW, header::HeaderValue::from_static("*/*"));
        headers.insert(header::CONNECTION, header::HeaderValue::from_static("keep-alive"));

        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(0)
            .default_headers(headers)
            .danger_accept_invalid_certs(true)
            .build().unwrap();

        Self {
            client,
        }
    }
}


impl Api {

    pub async fn fetch_db_updated(&self, url: &str, last_update_ts: &str,device_id:&str) -> ApiResult<ResponseData<Person>> {
        let dst_url = utils::add_url_query(url,"last_update",last_update_ts);
        let dst_url =  utils::add_url_query(&dst_url,"hw_id",device_id);
        do_get(&self.client, &dst_url).await
    }

    pub async fn fetch_camera_updated(&self, url: &str, last_update_ts: &str,device_id:&str) -> ApiResult<ResponseData<Camera>> {
        let dst_url = utils::add_url_query(url,"last_update",last_update_ts);
        let dst_url =  utils::add_url_query(&dst_url,"hw_id",device_id);
        do_get(&self.client, &dst_url).await
    }

}

async fn do_get<T: DeserializeOwned>(client: &Client, url: &str) -> ApiResult<T> {

    println!("--> url: {:?}",url);

    let response = client.get(url)
        .send().await?;

    if response.status() != StatusCode::OK {
        return Err(ApiError::net_err(&format!("http status:{}", response.status())));
    }

    let body = response.bytes().await?;

    // debug
    {
        let body_str = std::str::from_utf8(&body);
        match body_str {
            Ok(v) => {
                println!("--> body: {}",v);
            }
            Err(e) => {
                println!("--> from_utf8, err: {:?}",e);
            }
        }

    }


    let res = serde_json::from_reader(body.reader())?;

    Ok(res)
}