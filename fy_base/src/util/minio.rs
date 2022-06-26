
use chrono::{DateTime, Local};
use s3::{Bucket, Region};
use s3::creds::Credentials;
use s3::error::S3Error;

pub fn new_bucket(endpoint: &str, user: &str, password: &str, bucket_name: &str) -> Result<Bucket, S3Error> {
    let region = Region::Custom {
        region: "minio".to_string(),
        endpoint: endpoint.to_string(),
    };

    let credentials = Credentials {
        access_key: Some(user.to_string()),
        secret_key: Some(password.to_string()),
        security_token: None,
        session_token: None,
    };

    let bucket = Bucket::new(bucket_name, region, credentials)?;
    Ok(bucket.with_path_style())
}


//-----------
// 网络失败 S3Error
pub async fn save_to_minio(bucket: &Bucket, path: &str, content: &[u8]) -> Result<(String, u16), S3Error> {
    let rst = bucket.put_object(path, content).await?;
    let code = rst.1;
    let etag = match String::from_utf8(rst.0) {
        Ok(v) => v,
        Err(e) => {
            return Err(S3Error::FromUtf8(e));
        }
    };

    Ok((etag, code))
}


//---------------------------------
/*
facetrack
/2022/06/25/abcdef/
acdef_bg.jpg

acdef_1_s.jpg
acdef_1_l.jpg
acdef_1_fea.txt

acdef_2_s.jpg
acdef_2_l.jpg
acdef_2_fea.txt
*/

fn get_ts_prefix(ts: DateTime<Local>) -> String {
    ts.format("/%Y/%m/%d").to_string()
}

pub fn get_facetrack_relate_bg_path(uuid: &str, ts: DateTime<Local>) -> String {
    let ts_prefix = get_ts_prefix(ts);
    format!("{}/{}/{}_bg.jpg", ts_prefix, uuid, uuid)
}

pub fn get_facetrack_relate_small_path(uuid: &str, ts: DateTime<Local>, face_id: u8) -> String {
    let ts_prefix = get_ts_prefix(ts);
    format!("{}/{}/{}_{}_s.jpg", ts_prefix, uuid, uuid, face_id)
}

pub fn get_facetrack_relate_large_path(uuid: &str, ts: DateTime<Local>, face_id: u8) -> String {
    let ts_prefix = get_ts_prefix(ts);
    format!("{}/{}/{}_{}_l.jpg", ts_prefix, uuid, uuid, face_id)
}

pub fn get_facetrack_relate_fea_path(uuid: &str, ts: DateTime<Local>, face_id: u8) -> String {
    let ts_prefix = get_ts_prefix(ts);
    format!("{}/{}/{}_{}_fea.txt", ts_prefix, uuid, uuid, face_id)
}

//-------------------------------------
/*
cartrack
/2022/06/25/abcdef/
acdef_bg.jpg

acdef_1.jpg
acdef_2.jpg
acdef_3.jpg

acdef_plate.jpg

 */

pub fn get_cartrack_relate_bg_path(uuid: &str, ts: DateTime<Local>) -> String {
    let ts_prefix = get_ts_prefix(ts);
    format!("{}/{}/{}_bg.jpg", ts_prefix, uuid, uuid)
}

pub fn get_cartrack_relate_car_path(uuid: &str, ts: DateTime<Local>, car_id: u8) -> String {
    let ts_prefix = get_ts_prefix(ts);
    format!("{}/{}/{}_{}.jpg", ts_prefix, uuid, uuid, car_id)
}

pub fn get_cartrack_relate_plate_path(uuid: &str, ts: DateTime<Local>) -> String {
    let ts_prefix = get_ts_prefix(ts);
    format!("{}/{}/{}_plate.jpg", ts_prefix, uuid, uuid)
}

