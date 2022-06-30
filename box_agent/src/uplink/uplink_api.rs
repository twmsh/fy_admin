use std::{fmt::Debug, time::Duration};

use axum::http::StatusCode;
use bytes::Buf;
use reqwest::multipart::{Form, Part};
use std::fmt::{Display, Formatter};
use std::ops::Deref;

use reqwest::header;
use reqwest::Client;
use tracing::debug;

use fy_base::api::upload_api::{NotifyCarQueueItem, NotifyFaceQueueItem, ResponseData};

#[derive(Debug)]
pub enum ApiError {
    JsonErr(String),
    IoErr(String),
    NetErr(String),
    HttpErr(u16),
    BizErr(String),
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::JsonErr(v) => {
                write!(f, "JsonErr:{}", v)
            }
            ApiError::IoErr(v) => {
                write!(f, "IoErr:{}", v)
            }
            ApiError::NetErr(v) => {
                write!(f, "NetErr:{}", v)
            }
            ApiError::HttpErr(code) => {
                write!(f, "HttpErr:(http code:{})", code)
            }
            ApiError::BizErr(v) => {
                write!(f, "BizErr:{}", v)
            }
        }
    }
}

impl std::error::Error for ApiError {}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        Self::JsonErr(format!("{:?}", e))
    }
}

impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        Self::IoErr(format!("{:?}", e))
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        Self::NetErr(format!("{:?}", e))
    }
}

pub type ApiResult<T> = std::result::Result<T, ApiError>;

pub struct UplinkApi {
    pub client: Client,
}

impl Default for UplinkApi {
    fn default() -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("uplink-api/1.0"),
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

impl UplinkApi {
    // 上传人脸
    pub async fn upload_face(&self, url: &str, item: NotifyFaceQueueItem) -> ApiResult<()> {
        let json_content = serde_json::to_string(&item)?;
        debug!("--> face json: {}", json_content);

        let mut form = Form::new().text("json", json_content);
        form = form.text("type", "facetrack");

        // 背景图
        let bg_part = fill_part(
            &item.notify.background.image_buf,
            item.notify.background.image_file.clone(),
            "image/jpeg",
        )?;
        form = form.part(item.notify.background.image_file.clone(), bg_part);

        // 人脸图/特征
        for face in item.notify.faces.iter() {
            // 小图
            let part = fill_part(&face.aligned_buf, face.aligned_file.clone(), "image/bmp")?;
            form = form.part(face.aligned_file.clone(), part);

            // 显示图
            let part = fill_part(&face.display_buf, face.display_file.clone(), "image/bmp")?;
            form = form.part(face.display_file.clone(), part);

            // 特征值，如果有
            if let Some(ref fea) = face.feature_file {
                if let Some(ref fea_buf) = face.feature_buf {
                    let part = fill_part(fea_buf.deref(), fea.clone(), "application/octet-stream")?;
                    form = form.part(fea.clone(), part);
                }
            }
        }

        let res = self.client.post(url).multipart(form).send().await?;

        if res.status() != StatusCode::OK {
            return Err(ApiError::HttpErr(res.status().as_u16()));
        }

        let body = res.bytes().await?;
        let res_data: ResponseData = serde_json::from_reader(body.reader())?;

        if res_data.status != 0 {
            return Err(ApiError::BizErr(format!(
                "return status:{}, message:{:?}",
                res_data.status, res_data.message
            )));
        }

        Ok(())
    }

    // 上传车辆
    pub async fn upload_car(&self, url: &str, item: NotifyCarQueueItem) -> ApiResult<()> {
        let json_content = serde_json::to_string(&item)?;
        let mut form = Form::new().text("json", json_content);

        form = form.text("type", "vehicletrack");

        // 背景图
        let bg_part = fill_part(
            &item.notify.background.image_buf,
            item.notify.background.image_file.clone(),
            "image/jpeg",
        )?;
        form = form.part(item.notify.background.image_file.clone(), bg_part);

        // 车身图
        for car in item.notify.vehicles.iter() {
            let part = fill_part(&car.img_buf, car.image_file.clone(), "image/jpeg")?;
            form = form.part(car.image_file.clone(), part);
        }

        // 车牌图
        if let Some(ref plate) = item.notify.plate_info {
            if let Some(ref img_fn) = plate.image_file {
                let part = fill_part(&plate.img_buf, img_fn.clone(), "image/bmp")?;
                form = form.part(img_fn.clone(), part);
            }

            if let Some(ref img_fn) = plate.binary_file {
                let part = fill_part(&plate.binary_buf, img_fn.clone(), "image/bmp")?;
                form = form.part(img_fn.clone(), part);
            }
        }

        let res = self.client.post(url).multipart(form).send().await?;

        if res.status() != StatusCode::OK {
            return Err(ApiError::HttpErr(res.status().as_u16()));
        }

        let body = res.bytes().await?;
        let res_data: ResponseData = serde_json::from_reader(body.reader())?;

        if res_data.status != 0 {
            return Err(ApiError::BizErr(format!(
                "return status:{}, message:{:?}",
                res_data.status, res_data.message
            )));
        }

        Ok(())
    }
}

fn fill_part(buf: &[u8], meta_fn: String, mime_str: &str) -> ApiResult<Part> {
    let buf = Vec::from(buf);
    let part = Part::bytes(buf).file_name(meta_fn).mime_str(mime_str);
    match part {
        Ok(v) => Ok(v),
        Err(e) => Err(ApiError::IoErr(format!("{:?}", e))),
    }
}
