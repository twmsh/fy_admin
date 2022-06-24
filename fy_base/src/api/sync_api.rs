use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use std::time::Duration;

use bytes::Buf;
use chrono::{DateTime, Local};
use reqwest::header;
use reqwest::{Client, Error as Reqwest_Error};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Error as Serde_Error;
use tracing::{debug, error, info};

use crate::util::time_format::long_ts_format;
use crate::util::utils;

pub const SYNC_OP_MODIFY: i8 = 1;
pub const SYNC_OP_DEL: i8 = 2;

pub const RES_STATUS_OK: i32 = 0;
pub const RES_STATUS_ERROR: i32 = 500;
pub const RES_STATUS_INVALID_PARA: i32 = 1;
pub const RES_STATUS_BIZ_ERR: i32 = 2;

#[derive(Serialize, Deserialize, Debug)]
pub struct PersonInfoFace {
    //base64
    pub fea: String,
    pub quality: f32,

    pub id: String, // face num
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PersonInfo {
    pub uuid: String,
    pub db_id: String,

    //base64 是否需要?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregate: Option<String>,
    pub faces: Vec<PersonInfoFace>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Person {
    pub id: String,
    pub uuid: String,
    pub db_id: String,

    // 1：增加或修改 2：删除
    pub op: i8,

    #[serde(with = "long_ts_format")]
    pub last_update: DateTime<Local>,

    pub detail: Option<PersonInfo>,
}

impl Person {
    pub fn add_face(&mut self, face: PersonInfoFace) {
        match self.detail {
            None => {
                let info = PersonInfo {
                    uuid: self.uuid.clone(),
                    db_id: self.db_id.clone(),
                    aggregate: None,
                    faces: vec![face],
                };
                self.detail = Some(info);
            }
            Some(ref mut v) => {
                v.faces.push(face);
            }
        }
    }
}

//----------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct Camera {
    pub id: String,
    pub uuid: String,

    // 1：增加或修改 2：删除
    pub op: i8,

    #[serde(with = "long_ts_format")]
    pub last_update: DateTime<Local>,

    // 采集类型 1：人脸 2：车辆 3：人脸+车辆
    #[serde(rename = "type")]
    pub c_type: i64,

    pub detail: Option<CameraInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CameraInfo {
    pub uuid: String,

    // 采集地址
    pub url: String,

    // 摄像头配置
    pub config: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Db {
    pub id: String,
    pub uuid: String,

    // 1：增加或修改 2：删除
    pub op: i8,

    #[serde(with = "long_ts_format")]
    pub last_update: DateTime<Local>,

    pub capacity: i32,
}

//----------------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseData<T> {
    pub status: i32,
    pub message: Option<String>,

    #[serde(with = "long_ts_format")]
    pub ts: DateTime<Local>,

    #[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<T>>,
}

impl<T> ResponseData<T> {
    pub fn is_empty(&self) -> bool {
        match self.data {
            None => true,
            Some(ref v) => v.is_empty(),
        }
    }
}

impl<T> IntoResponse for ResponseData<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

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
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("sync_client-api/1.0"),
        );
        headers.insert(
            header::ACCEPT_ENCODING,
            header::HeaderValue::from_static("gzip, deflate"),
        );
        headers.insert(header::ALLOW, header::HeaderValue::from_static("*/*"));
        headers.insert(
            header::CONNECTION,
            header::HeaderValue::from_static("keep-alive"),
        );

        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(0)
            .default_headers(headers)
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        Self { client }
    }
}

impl Api {
    pub async fn fetch_db_updated(
        &self,
        url: &str,
        last_update_ts: &str,
        hw_id: &str,
    ) -> ApiResult<ResponseData<Db>> {
        let dst_url = utils::add_url_query(url, "last_update", last_update_ts);
        let dst_url = utils::add_url_query(&dst_url, "hw_id", hw_id);
        do_get(&self.client, &dst_url).await
    }

    pub async fn fetch_person_updated(
        &self,
        url: &str,
        last_update_ts: &str,
        hw_id: &str,
    ) -> ApiResult<ResponseData<Person>> {
        let dst_url = utils::add_url_query(url, "last_update", last_update_ts);
        let dst_url = utils::add_url_query(&dst_url, "hw_id", hw_id);
        do_get(&self.client, &dst_url).await
    }

    pub async fn fetch_camera_updated(
        &self,
        url: &str,
        last_update_ts: &str,
        hw_id: &str,
    ) -> ApiResult<ResponseData<Camera>> {
        let dst_url = utils::add_url_query(url, "last_update", last_update_ts);
        let dst_url = utils::add_url_query(&dst_url, "hw_id", hw_id);
        do_get(&self.client, &dst_url).await
    }
}

async fn do_get<T: DeserializeOwned>(client: &Client, url: &str) -> ApiResult<T> {
    info!("url: {:?}", url);

    let response = client.get(url).send().await?;

    if response.status() != StatusCode::OK {
        return Err(ApiError::net_err(&format!(
            "http status:{}",
            response.status()
        )));
    }

    let body = response.bytes().await?;

    // debug
    let debug_it = false;
    if debug_it {
        let body_str = std::str::from_utf8(&body);
        match body_str {
            Ok(v) => {
                debug!("http body: {}", v);
            }
            Err(e) => {
                error!("http body from_utf8, err: {:?}", e);
            }
        }
    }

    let res = serde_json::from_reader(body.reader())?;

    Ok(res)
}
