use std::sync::Arc;
use std::time::Instant;

use axum::extract::{ContentLengthLimit, Multipart};
use axum::Extension;
use bytes::Bytes;
use chrono::Local;
use fy_base::api::upload_api::{NotifyCarQueueItem, NotifyFaceQueueItem, ResponseData};
use fy_base::util::image as image_util;
use fy_base::util::multipart_form::{parse_multi_form, MultipartFormValues};
use serde_json::{self, Result as JsonResult};
use tracing::{debug, error, info};

use crate::service::web::WebState;

//----------------------------------------
fn build_err_response(err_msg: &str) -> ResponseData {
    ResponseData {
        status: 500,
        message: Some(err_msg.to_string()),
    }
}

fn build_ok_response() -> ResponseData {
    ResponseData {
        status: 0,
        message: Some("success".to_string()),
    }
}

//-----------------------------------
pub async fn track_upload(
    Extension(web_state): Extension<Arc<WebState>>,
    ContentLengthLimit(parts): ContentLengthLimit<Multipart, { 1024 * 1024 * 10 }>,
) -> ResponseData {

    let begin_ts = Instant::now();

    let part_values = match parse_multi_form(parts).await {
        Ok(v) => v,
        Err(e) => {
            error!("error, track_upload, parse_multi_form, err: {:?}", e);
            return build_err_response(&format!("error, {:?}", e));
        }
    };

    if let Some(track_type) = part_values.get_string_value("type") {
        match track_type.as_str() {
            "facetrack" => handle_face(web_state, part_values,begin_ts).await,
            "vehicletrack" => handle_car(web_state, part_values,begin_ts).await,
            _ => {
                error!("error, track_upload, unknown type: {}", track_type);
                build_err_response(&format!("error, unknown type: {}", track_type))
            }
        }
    } else {
        error!("error, track_upload, field: type not found");
        build_err_response("error, field: type not found")
    }
}

async fn handle_face(data: Arc<WebState>, values: MultipartFormValues,begin_ts: Instant) -> ResponseData {
    let now = Local::now();
    let json_str = match values.get_string_value("json") {
        Some(v) => v,
        None => {
            error!("error, field: json not found");
            return build_err_response("error, field: json not found");
        }
    };

    debug!("->face:{}", json_str);

    let face_queue_item: JsonResult<NotifyFaceQueueItem> =
        serde_json::from_reader(json_str.as_bytes());
    if let Err(e) = face_queue_item {
        error!("error, json parse error, err: {:?}", e);
        return build_err_response(&format!("json parse error, err: {:?}", e));
    }

    let mut face_queue_item = face_queue_item.unwrap();
    face_queue_item.ts = now;
    let item = &mut face_queue_item.notify;

    debug!("recv track, {}, index:{}, ft", item.id, item.index);

    // ????????????
    item.background.image_buf = match values.get_file_value(item.background.image_file.as_str()) {
        Some((_, v)) => v,
        None => {
            error!("error, can't find para: {}", item.background.image_file);
            return build_err_response(&format!(
                "error, can't find field: {}",
                item.background.image_file
            ));
        }
    };

    for x in item.faces.iter_mut() {
        x.aligned_buf = match get_jpg_file_value(&values, x.aligned_file.as_str()) {
            Ok(v) => v,
            Err(e) => {
                error!("error, {}", e);
                return build_err_response(&e);
            }
        };

        x.display_buf = match get_jpg_file_value(&values, x.display_file.as_str()) {
            Ok(v) => v,
            Err(e) => {
                error!("error, {}", e);
                return build_err_response(&e);
            }
        };

        if let Some(ref feature_file) = x.feature_file {
            if !feature_file.is_empty() {
                x.feature_buf = match values.get_file_value(feature_file.as_str()) {
                    Some((_, v)) => Some(v),
                    None => {
                        error!("error, can't find field: {}", feature_file);
                        return build_err_response(&format!("can't find field: {}", feature_file));
                    }
                }
            } else {
                x.feature_file = None;
                debug!("{}, has no feature", item.id);
            }
        } else {
            x.feature_file = None;
            debug!("{}, has no feature", item.id);
        }
    }

    data.face_queue.push(face_queue_item);

    info!("track_upload, process face, use: {}",begin_ts.elapsed().as_millis() );
    build_ok_response()
}

async fn handle_car(data: Arc<WebState>, values: MultipartFormValues,begin_ts: Instant) -> ResponseData {
    let now = Local::now();

    let json_str = match values.get_string_value("json") {
        Some(v) => v,
        None => {
            error!("error, field: json not found");
            return build_err_response("error, field: json not found");
        }
    };
    debug!("->car:{}", json_str);

    let car_queue_item: JsonResult<NotifyCarQueueItem> =
        serde_json::from_reader(json_str.as_bytes());
    if let Err(e) = car_queue_item {
        error!("error, json parse error, err: {:?}", e);
        return build_err_response(&format!("json parse error, err: {:?}", e));
    }
    let mut car_queue_item = car_queue_item.unwrap();
    car_queue_item.ts = now;
    let item = &mut car_queue_item.notify;

    debug!("recv track, {}, index:{}, ct", item.id, item.index);

    // ????????????
    item.background.image_buf = match values.get_file_value(item.background.image_file.as_str()) {
        Some((_, v)) => v,
        None => {
            error!("error, can't find field: {}", item.background.image_file);
            return build_err_response(&format!(
                "error, can't find field: {}",
                item.background.image_file
            ));
        }
    };

    for x in item.vehicles.iter_mut() {
        x.img_buf = match get_jpg_file_value(&values, x.image_file.as_str()) {
            Ok(v) => v,
            Err(e) => {
                error!("error, {}", e);
                return build_err_response(&e);
            }
        };
    }

    // ???????????????
    if item.has_plate_info() {
        let x = item.plate_info.as_mut().unwrap();
        if let Some(ref img) = x.image_file {
            x.img_buf = match get_jpg_file_value(&values, img.as_str()) {
                Ok(v) => v,
                Err(e) => {
                    error!("error, {}", e);
                    return build_err_response(&e);
                }
            };
        } else {
            error!("error, has plate text, but hasn't plate img");
        }
    }

    if item.has_plate_binary() {
        let x = item.plate_info.as_mut().unwrap();
        if let Some(ref img) = x.binary_file {
            x.binary_buf = match get_jpg_file_value(&values, img.as_str()) {
                Ok(v) => v,
                Err(e) => {
                    error!("error, {}", e);
                    return build_err_response(&e);
                }
            };
        } else {
            error!("error, has plate binary, but hasn't plate binary img");
        }
    }

    data.car_queue.push(car_queue_item);
    info!("track_upload, process car, use: {}",begin_ts.elapsed().as_millis());

    build_ok_response()
}

fn get_jpg_file_value(
    values: &MultipartFormValues,
    name: &str,
) -> std::result::Result<Bytes, String> {
    let buf = match values.get_file_value(name) {
        Some((_, v)) => v,
        None => {
            return Err(format!("can't find field: {}", name));
        }
    };

    // ??????jpg
    match image_util::escape_bmp(buf) {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("can't escape bmp: {}, {:?}", name, e)),
    }
}
