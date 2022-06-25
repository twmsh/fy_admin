use s3::{Bucket, Region};
use s3::creds::Credentials;
use s3::error::S3Error;

pub fn new_bucket(endpoint:&str, user:&str, password:&str, bucket_name:&str) -> Result<Bucket,S3Error> {
    let region = Region::Custom {
        region: "minio".to_string(),
        endpoint: endpoint.to_string()
    };

    let credentials = Credentials{
        access_key: Some(user.to_string()),
        secret_key: Some(password.to_string()),
        security_token: None,
        session_token: None
    };

    let bucket = Bucket::new(bucket_name,region,credentials)?;
    Ok(bucket.with_path_style())
}