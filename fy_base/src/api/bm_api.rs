use std::{
    fmt::Debug,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use bytes::{buf::Buf, Bytes};
use http::StatusCode;
use jsonrpc_core::{Id, MethodCall, Output, Params, Version};

use reqwest::header;
use reqwest::{Client, Error as Reqwest_Error};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{Error as Serde_Error, Value};

#[derive(Debug)]
pub enum ApiError {
    NetErr(String),
    JsonrpcErr(i64, String),
    JsonErr(String),
    BizErr(String),
}

pub type ApiResult<T> = std::result::Result<T, ApiError>;

static ID: AtomicU64 = AtomicU64::new(1);

//------------------------ ApiError ------------------------
impl ApiError {
    pub fn net_err(s: &str) -> Self {
        ApiError::NetErr(s.to_string())
    }
    pub fn json_err(s: &str) -> Self {
        ApiError::JsonErr(s.to_string())
    }
    pub fn jsonrpc_err(code: i64, msg: &str) -> Self {
        ApiError::JsonrpcErr(code, msg.to_string())
    }
    pub fn biz_err(code: i64, msg: &str) -> Self {
        ApiError::BizErr(format!("code:{}, msg:{}", code, msg))
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

// ----------- notify face struct -----------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiRect {
    pub x: i64,
    pub y: i64,
    pub w: i64,
    pub h: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiPosition {
    pub end: i64,
    pub end_frame: i64,
    pub end_real_time: i64,
    pub start: i64,
    pub start_frame: i64,
    pub start_real_time: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiFaceProps {
    pub age: i64,
    pub gender: i64,
    pub glasses: i64,
    pub move_direction: i64,
}

type ApiAngles = [f64; 3];

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotifyBackground {
    pub frame_num: i64,
    pub height: i64,
    pub width: i64,
    pub image: Option<String>,
    pub image_file: String,
    pub rect: ApiRect,
    pub video_width: i64,
    pub video_height: i64,

    #[serde(skip)]
    pub image_buf: Bytes,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotifyFace {
    pub aligned: Option<String>,
    pub aligned_file: String,
    pub angles: ApiAngles,
    pub display: Option<String>,
    pub display_file: String,
    pub feature_file: Option<String>,
    pub frame_num: i64,
    pub quality: f64,
    pub rect: ApiRect,

    #[serde(skip)]
    pub aligned_buf: Bytes,
    #[serde(skip)]
    pub display_buf: Bytes,
    #[serde(skip)]
    pub feature_buf: Option<Bytes>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotifyTrip {
    pub x: Option<i64>,
    pub y: Option<i64>,
    pub w: Option<i64>,
    pub h: Option<i64>,
    pub real_time: Option<i64>,
    pub pts: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FaceNotifyParams {
    pub background: NotifyBackground,
    pub faces: Vec<NotifyFace>,
    pub id: String,
    pub index: i64,
    pub position: ApiPosition,
    pub props: Option<ApiFaceProps>,
    pub source: String,
    pub trip: Option<NotifyTrip>,
    pub version: String,
}

// ----------- notify car  struct -----------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotifyCar {
    pub image: Option<String>,
    pub image_file: String,
    pub frame_num: i64,
    pub rect: ApiRect,

    #[serde(skip)]
    pub img_buf: Bytes,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiCarPlateType {
    pub value: String,
    pub conf: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiCarPlateBit {
    pub value: String,
    pub conf: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiCarPlateInfo {
    pub binary_file: Option<String>,
    pub text: Option<String>,
    pub image: Option<String>,
    pub image_file: Option<String>,

    #[serde(rename = "type")]
    pub plate_type: Option<ApiCarPlateType>,
    pub bits: Option<Vec<Vec<ApiCarPlateBit>>>,

    #[serde(skip)]
    pub img_buf: Bytes,

    #[serde(skip)]
    pub binary_buf: Bytes,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiScoreValue {
    pub value: String,
    pub score: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiCarProps {
    pub move_direction: Option<i64>,
    pub brand: Option<Vec<ApiScoreValue>>,
    pub direction: Option<Vec<ApiScoreValue>>,
    pub color: Option<Vec<ApiScoreValue>>,
    pub mid_type: Option<Vec<ApiScoreValue>>,
    pub series: Option<Vec<ApiScoreValue>>,
    pub top_series: Option<Vec<ApiScoreValue>>,
    pub top_type: Option<Vec<ApiScoreValue>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CarNotifyParams {
    pub id: String,
    pub index: i64,
    pub source: String,
    pub vehicles: Vec<NotifyCar>,
    pub plate_info: Option<ApiCarPlateInfo>,
    pub background: NotifyBackground,
    pub position: ApiPosition,
    pub trip: Option<NotifyTrip>,
    pub props: Option<ApiCarProps>,
    pub version: String,
}

// ----------- AnalysisApi struct -----------------
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateSourceReqConfigProduce {
    pub bg_width: i64,
    pub draw_rect_on_bg: bool,
    pub max_targets_in_track: i64,
    pub size_filter_rate: f64,
    pub remove_vehicle_without_plate: bool,
    pub feature_count: i64,
    pub min_plate_width: i64,
    pub plate_binary: bool,
    pub plate_filter_rate: f64,
    pub min_plate_bit_score: f64,
    pub enable_plate: bool,
    pub enable_vehicle_classify: bool,
    pub enable_plate_merge: bool,
    pub enable_portrait: bool,
    pub vehicle_history_max_duration: i64,
    pub vehicle_history_max_size: i64,
    pub plate_precision: i64,
    pub plate_width: i64,
    pub plate_height: i64,
    pub max_plate_height: f64,
    pub stationary_iou_threshold: f64,
    pub face_duplicate_threshold: f64,
    pub face_cluster_threshold: f64,
    pub face_merge_threshold: f64,
    pub face_merge_window: i64,
    pub min_face_history_quality: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateSourceReqConfigFace {
    pub left: i64,
    pub top: i64,
    pub width: i64,
    pub height: i64,
    pub detector_type: i64,
    pub detect_interval: i64,
    pub track_interval: i64,
    pub sample_interval: i64,
    pub onet_interval: i64,
    pub angle_tb_threshold: i64,
    pub angle_lf_threshold: i64,
    pub display_margin: f64,
    pub display_width: i64,
    pub enable_display: bool,
    pub expire_duration: i64,
    pub silent_ttl: i64,
    pub max_tracker: i64,
    pub min_obj_count: i64,
    pub min_width: i64,
    pub mtcnn_factor: f64,
    pub mtcnn_min_size: i64,
    pub merge_iou: f64,
    pub ie_margin_right: i64,
    pub ie_margin_bottom: i64,
    pub ie_margin_left: i64,
    pub ie_margin_top: i64,
    pub remove_abnormal_detection: bool,
    pub horizontal_trip_line: i64,
    pub thresh: Option<Vec<f64>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateSourceReqConfigVehicle {
    pub left: i64,
    pub top: i64,
    pub width: i64,
    pub height: i64,
    pub detect_interval: i64,
    pub sample_interval: i64,
    pub display_margin: f64,
    pub display_width: i64,
    pub silent_ttl: i64,
    pub expire_duration: i64,
    pub max_tracker: i64,
    pub min_obj_count: i64,
    pub min_width: i64,
    pub merge_iou: f64,
    pub ie_margin_right: i64,
    pub ie_margin_bottom: i64,
    pub ie_margin_left: i64,
    pub ie_margin_top: i64,
    pub remove_abnormal_detection: bool,
    pub horizontal_trip_line: i64,

    pub store_ratio: f64,
    pub activate_station_ratio: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateSourceReqConfig {
    pub enable_face: bool,
    pub enable_vehicle: bool,
    pub jpg_encode_threshold: i64,

    #[serde(rename = "loop")]
    pub is_loop: bool,

    pub player: i64,
    pub upload_url: String,
    pub decode_result_queue_size: i64,
    pub produce: CreateSourceReqConfigProduce,
    pub face: CreateSourceReqConfigFace,
    pub vehicle: CreateSourceReqConfigVehicle,

    pub live_height: i64,
    pub live_server: i64,
    pub live_width: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateSourceReq {
    pub url: String,
    pub id: Option<String>,
    pub config: CreateSourceReqConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateSourceRes {
    pub code: i64,
    pub msg: String,
    pub id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateSourceReq {
    pub url: String,
    pub id: String,
    pub config: CreateSourceReqConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateSourceRes {
    pub code: i64,
    pub msg: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteSourceReq {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteSourceRes {
    pub code: i64,
    pub msg: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetSourcesReq {}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetSourcesResSource {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetSourcesRes {
    pub code: i64,
    pub msg: String,
    pub sources: Option<Vec<GetSourcesResSource>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetSourceInfoReq {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetSourceInfoRes {
    pub code: i64,
    pub msg: String,

    pub state: Option<i64>,
    pub url: Option<String>,
    pub config: Option<CreateSourceReqConfig>,
    pub last_active_time: Option<i64>,
    pub duration: Option<i64>,
}

// ----------- RecognitionApi struct -----------------
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiPoint {
    pub x: i64,
    pub y: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiDetectFaceAttr {
    pub age: i64,
    pub gender: i64,
    pub glasses: i64,
    pub direction: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiDetectFace {
    pub aligned: String,

    #[serde(rename = "box")]
    pub api_box: ApiRect,
    pub pts: Vec<ApiPoint>,
    pub score: f64,
    pub feature: Option<String>,
    pub attr: Option<ApiDetectFaceAttr>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DetectReq {
    pub image: String,
    pub retfeat: bool,
    pub retattr: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DetectRes {
    pub code: i64,
    pub msg: String,

    pub faces: Option<Vec<ApiDetectFace>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetFeaturesReq {
    pub images: Vec<String>,
    pub retattr: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetFeaturesRes {
    pub code: i64,
    pub msg: String,

    pub features: Option<Vec<String>>,
    pub attrs: Option<Vec<ApiDetectFaceAttr>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateDbReq {
    pub id: Option<String>,
    pub volume: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateDbRes {
    pub code: i64,
    pub msg: String,

    pub id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetDbInfoReq {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetDbInfoRes {
    pub code: i64,
    pub msg: String,

    pub volume: Option<i64>,
    pub usage: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteDbReq {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteDbRes {
    pub code: i64,
    pub msg: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlushDbReq {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FlushDbRes {
    pub code: i64,
    pub msg: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetDbsReq {}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetDbsRes {
    pub code: i64,
    pub msg: String,
    pub dbs: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiFeatureQuality {
    pub feature: String,
    pub quality: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePersonsReq {
    pub features: Vec<Vec<ApiFeatureQuality>>,
    pub ids: Vec<String>,
    pub db: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePersonsResPerson {
    pub id: String,
    pub faces: Vec<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePersonsRes {
    pub code: i64,
    pub msg: String,
    pub persons: Option<Vec<CreatePersonsResPerson>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeletePersonReq {
    pub db: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeletePersonRes {
    pub code: i64,
    pub msg: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetDbPersonsReq {
    pub id: String,
    pub offset: i64,
    pub count: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetDbPersonsRes {
    pub code: i64,
    pub msg: String,
    pub persons: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MovePersonsReq {
    pub db_src: String,
    pub db_dst: String,
    pub ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MovePersonsRes {
    pub code: i64,
    pub msg: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPersonInfoReq {
    pub db: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPersonInfoResFace {
    pub id: i64,
    pub feature: String,
    pub quality: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetPersonInfoRes {
    pub code: i64,
    pub msg: String,

    pub aggregate_feature: Option<String>,
    pub faces: Option<Vec<GetPersonInfoResFace>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddFeaturesToPersonReq {
    pub db: String,
    pub id: String,
    pub features: Vec<ApiFeatureQuality>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddFeaturesToPersonRes {
    pub code: i64,
    pub msg: String,

    pub ids: Option<Vec<i64>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddAggregateFeatureToPersonReq {
    pub db: String,
    pub id: String,
    pub feature: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddAggregateFeatureToPersonRes {
    pub code: i64,
    pub msg: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeletePersonFeatureReq {
    pub db: String,
    pub person: String,
    pub id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeletePersonFeatureRes {
    pub code: i64,
    pub msg: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchReq {
    pub features: Vec<Vec<ApiFeatureQuality>>,
    pub top: Vec<i64>,
    pub threshold: Vec<i64>,
    pub db: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResPerson {
    pub id: String,
    pub score: i64,
    pub db: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchRes {
    pub code: i64,
    pub msg: String,
    pub persons: Option<Vec<Vec<SearchResPerson>>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompareReq {
    pub a: String,
    pub b: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompareRes {
    pub code: i64,
    pub msg: String,
    pub score: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompareNReq {
    pub a: Vec<ApiFeatureQuality>,
    pub b: Vec<Vec<ApiFeatureQuality>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CompareNRes {
    pub code: i64,
    pub msg: String,
    pub scores: Option<Vec<i64>>,
}

//---------------------------------------- do_post ------------------------
async fn do_post<R: Serialize, T: DeserializeOwned>(
    client: &Client,
    url: &str,
    method: &str,
    req: &R,
) -> ApiResult<T> {
    let req = serde_json::to_value(req)?;

    let params = match req {
        Value::Object(m) => m,
        _ => return Err(ApiError::json_err("not a object")),
    };

    let id = ID.fetch_add(1, Ordering::SeqCst);

    let msg = MethodCall {
        jsonrpc: Some(Version::V2),
        method: method.to_string(),
        params: Params::Map(params),
        id: Id::Num(id),
    };

    // println!("id:{}", id);

    let response = client
        .post(url)
        // .header("content-type", "application/json;charset=utf-8")
        .json(&msg)
        .send()
        .await?;

    if response.status() != StatusCode::OK {
        return Err(ApiError::net_err(&format!(
            "http status:{}",
            response.status()
        )));
    }

    let body = response.bytes().await?;
    let rpc_res: Output = serde_json::from_reader(body.reader())?;

    match rpc_res {
        Output::Success(suc) => Ok(serde_json::from_value::<T>(suc.result)?),
        Output::Failure(fail) => Err(ApiError::jsonrpc_err(
            fail.error.code.code(),
            &fail.error.message,
        )),
    }
}

//------------------------ AnalysisApi ------------------------
pub struct AnalysisApi {
    pub url: String,
    pub client: Client,
    pub debug: bool,
}

impl AnalysisApi {
    pub fn new(url: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("bm-api/1.0"),
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
            .connect_timeout(Duration::from_secs(3))
            .pool_max_idle_per_host(0)
            .default_headers(headers)
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        Self {
            url: url.to_string(),
            client,
            debug: false,
        }
    }

    pub async fn create_source(
        &self,
        src_id: Option<String>,
        src_url: String,
        notify_url: String,
        config: CreateSourceReqConfig,
    ) -> ApiResult<CreateSourceRes> {
        let src_id = src_id.or_else(|| Some("".to_string()));
        let mut req = CreateSourceReq {
            url: src_url,
            id: src_id,
            config,
        };
        req.config.upload_url = notify_url;
        do_post(&self.client, self.url.as_str(), "create_source", &req).await
    }

    pub async fn update_source(
        &self,
        src_id: String,
        src_url: String,
        notify_url: String,
        config: CreateSourceReqConfig,
    ) -> ApiResult<UpdateSourceRes> {
        let mut req = UpdateSourceReq {
            url: src_url,
            id: src_id,
            config,
        };
        req.config.upload_url = notify_url;
        do_post(&self.client, self.url.as_str(), "update_source", &req).await
    }

    pub async fn delete_source(&self, id: String) -> ApiResult<DeleteSourceRes> {
        let req = DeleteSourceReq { id };
        do_post(&self.client, self.url.as_str(), "delete_source", &req).await
    }

    pub async fn get_sources(&self) -> ApiResult<GetSourcesRes> {
        let req = GetSourcesReq {};
        do_post(&self.client, self.url.as_str(), "get_sources", &req).await
    }

    pub async fn get_source_info(&self, id: String) -> ApiResult<GetSourceInfoRes> {
        let req = GetSourceInfoReq { id };
        do_post(&self.client, self.url.as_str(), "get_source_info", &req).await
    }
}

//------------------------ RecognitionApi ------------------------
pub struct RecognitionApi {
    pub url: String,
    pub client: Client,
    pub debug: bool,
}

impl RecognitionApi {
    pub fn new(url: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("bm-api/1.0"),
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
            .connect_timeout(Duration::from_secs(3))
            .pool_max_idle_per_host(0)
            .default_headers(headers)
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        Self {
            url: url.to_string(),
            client,
            debug: false,
        }
    }

    pub async fn detect(&self, data: String, retfeat: bool, retattr: bool) -> ApiResult<DetectRes> {
        let req = DetectReq {
            image: data,
            retfeat,
            retattr,
        };
        do_post(&self.client, self.url.as_str(), "detect", &req).await
    }

    pub async fn get_features(
        &self,
        images: Vec<String>,
        retattr: bool,
    ) -> ApiResult<GetFeaturesRes> {
        let req = GetFeaturesReq { images, retattr };
        do_post(&self.client, self.url.as_str(), "get_features", &req).await
    }

    pub async fn create_db(&self, id: Option<String>, volume: i64) -> ApiResult<CreateDbRes> {
        let id = id.or_else(|| Some("".to_string()));

        let req = CreateDbReq { id, volume };
        do_post(&self.client, self.url.as_str(), "create_db", &req).await
    }

    pub async fn get_db_info(&self, id: String) -> ApiResult<GetDbInfoRes> {
        let req = GetDbInfoReq { id };
        do_post(&self.client, self.url.as_str(), "get_db_info", &req).await
    }

    pub async fn delete_db(&self, id: String) -> ApiResult<DeleteDbRes> {
        let req = DeleteDbReq { id };
        do_post(&self.client, self.url.as_str(), "delete_db", &req).await
    }

    pub async fn flush_db(&self, id: String) -> ApiResult<FlushDbRes> {
        let req = FlushDbReq { id };
        do_post(&self.client, self.url.as_str(), "flush_db", &req).await
    }

    pub async fn get_dbs(&self) -> ApiResult<GetDbsRes> {
        let req = GetDbsReq {};
        do_post(&self.client, self.url.as_str(), "get_dbs", &req).await
    }

    pub async fn create_persons(
        &self,
        db: String,
        ids: Vec<String>,
        features: Vec<Vec<ApiFeatureQuality>>,
    ) -> ApiResult<CreatePersonsRes> {
        let req = CreatePersonsReq { features, ids, db };
        do_post(&self.client, self.url.as_str(), "create_persons", &req).await
    }

    pub async fn delete_person(&self, db: String, id: String) -> ApiResult<DeletePersonRes> {
        let req = DeletePersonReq { db, id };
        do_post(&self.client, self.url.as_str(), "delete_person", &req).await
    }

    pub async fn get_db_persons(
        &self,
        id: String,
        offset: i64,
        count: i64,
    ) -> ApiResult<GetDbPersonsRes> {
        let req = GetDbPersonsReq { id, offset, count };
        do_post(&self.client, self.url.as_str(), "get_db_persons", &req).await
    }

    pub async fn move_persons(
        &self,
        db_src: String,
        db_dst: String,
        ids: Vec<String>,
    ) -> ApiResult<MovePersonsRes> {
        let req = MovePersonsReq {
            db_src,
            db_dst,
            ids,
        };
        do_post(&self.client, self.url.as_str(), "move_persons", &req).await
    }

    pub async fn get_person_info(&self, db: String, id: String) -> ApiResult<GetPersonInfoRes> {
        let req = GetPersonInfoReq { db, id };
        do_post(&self.client, self.url.as_str(), "get_person_info", &req).await
    }

    pub async fn add_features_to_person(
        &self,
        db: String,
        id: String,
        features: Vec<ApiFeatureQuality>,
    ) -> ApiResult<AddFeaturesToPersonRes> {
        let req = AddFeaturesToPersonReq { db, id, features };
        do_post(
            &self.client,
            self.url.as_str(),
            "add_features_to_person",
            &req,
        )
        .await
    }

    pub async fn add_aggregate_feature_to_person(
        &self,
        db: String,
        id: String,
        feature: String,
    ) -> ApiResult<AddAggregateFeatureToPersonRes> {
        let req = AddAggregateFeatureToPersonReq { db, id, feature };
        do_post(
            &self.client,
            self.url.as_str(),
            "add_aggregate_feature_to_person",
            &req,
        )
        .await
    }

    pub async fn delete_person_feature(
        &self,
        db: String,
        id: String,
        face_id: i64,
    ) -> ApiResult<DeletePersonFeatureRes> {
        let req = DeletePersonFeatureReq {
            db,
            person: id,
            id: face_id,
        };
        do_post(
            &self.client,
            self.url.as_str(),
            "delete_person_feature",
            &req,
        )
        .await
    }

    pub async fn search(
        &self,
        db: Vec<String>,
        top: Vec<i64>,
        threshold: Vec<i64>,
        features: Vec<Vec<ApiFeatureQuality>>,
    ) -> ApiResult<SearchRes> {
        let req = SearchReq {
            features,
            db,
            top,
            threshold,
        };
        do_post(&self.client, self.url.as_str(), "search", &req).await
    }

    pub async fn compare(&self, aligned_a: String, aligned_b: String) -> ApiResult<CompareRes> {
        let req = CompareReq {
            a: aligned_a,
            b: aligned_b,
        };
        do_post(&self.client, self.url.as_str(), "compare", &req).await
    }

    pub async fn compare_n(
        &self,
        a: Vec<ApiFeatureQuality>,
        b: Vec<Vec<ApiFeatureQuality>>,
    ) -> ApiResult<CompareNRes> {
        let req = CompareNReq { a, b };
        do_post(&self.client, self.url.as_str(), "compare_N", &req).await
    }
}

//------------------------ impl notify ------------------------
impl FaceNotifyParams {
    pub fn has_trip_info(&self) -> bool {
        if let Some(ref v) = self.trip {
            return v.w.is_some();
        }
        false
    }
}

type CarPropsType = (
    i32,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

impl CarNotifyParams {
    /// 返回 (牌照号码,牌照类型)
    pub fn get_plate_tuple(&self) -> (Option<String>, Option<String>) {
        let mut plate_content = None;
        let mut plate_type = None;

        if let Some(ref plate) = self.plate_info {
            plate_content = plate.text.clone();

            if let Some(ref v) = plate.plate_type {
                plate_type = Some(v.value.clone());
            }
        }
        (plate_content, plate_type)
    }

    fn get_scorevalue_fromlist(&self, list: &Option<Vec<ApiScoreValue>>) -> Option<String> {
        if let Some(v) = list {
            return v.get(0).map(|x| x.value.clone());
        }

        None
    }

    /// 返回 车辆8个属性
    /// (move_direction,direct,color, brand,top_series,series,top_type,mid_type)
    /// (运动方向,车辆方向,车身颜色,品牌,车系,车款,车粗分类别,车类别)
    pub fn get_props_tuple(&self) -> CarPropsType {
        let mut move_direct = 0;
        let mut car_direct = None;
        let mut car_color = None;
        let mut car_brand = None;
        let mut car_top_series = None;
        let mut car_series = None;
        let mut car_top_type = None;
        let mut car_mid_type = None;

        if let Some(ref props) = self.props {
            if let Some(v) = props.move_direction {
                move_direct = v as i32;
            }

            car_direct = self.get_scorevalue_fromlist(&props.direction);
            car_color = self.get_scorevalue_fromlist(&props.color);
            car_brand = self.get_scorevalue_fromlist(&props.brand);
            car_top_series = self.get_scorevalue_fromlist(&props.top_series);
            car_series = self.get_scorevalue_fromlist(&props.series);
            car_top_type = self.get_scorevalue_fromlist(&props.top_type);
            car_mid_type = self.get_scorevalue_fromlist(&props.mid_type);
        }

        (
            move_direct,
            car_direct,
            car_color,
            car_brand,
            car_top_series,
            car_series,
            car_top_type,
            car_mid_type,
        )
    }

    pub fn has_plate_info(&self) -> bool {
        if let Some(ref v) = self.plate_info {
            // 根据车牌号码来判断
            if let Some(ref p) = v.text {
                return !p.is_empty();
            }
        }
        false
    }

    /// 是否有二值图
    pub fn has_plate_binary(&self) -> bool {
        if let Some(ref v) = self.plate_info {
            if let Some(ref p) = v.binary_file {
                return !p.is_empty();
            }
        }
        false
    }

    pub fn has_props_info(&self) -> bool {
        if let Some(ref v) = self.props {
            return v.color.is_some();
        }
        false
    }

    pub fn has_trip_info(&self) -> bool {
        if let Some(ref v) = self.trip {
            return v.w.is_some();
        }
        false
    }

    /// 车牌号码置信度
    pub fn get_plate_confidence(&self) -> Option<f64> {
        if let Some(ref v) = self.plate_info {
            if let Some(ref bits) = v.bits {
                let mut amount = 0_f64;
                let mut count = 0;
                bits.iter().for_each(|x| {
                    let conf = x.get(0);
                    if let Some(bit_conf) = conf {
                        amount += bit_conf.conf;
                        count += 1;
                    }
                });
                if count != 0 {
                    return Some(amount / count as f64);
                }
            }
        }

        None
    }
}
