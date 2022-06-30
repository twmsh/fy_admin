use s3::error::S3Error;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use tokio::task::JoinError;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub struct AppError {
    pub msg: String,
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for AppError {}

impl AppError {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }

    pub fn from_debug<T: Debug>(t: T) -> Self {
        Self {
            msg: format!("{:?}", t),
        }
    }

    pub fn from_display<T: Display>(t: T) -> Self {
        Self {
            msg: format!("{}", t),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::from_debug(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::from_debug(e)
    }
}

impl From<JoinError> for AppError {
    fn from(e: JoinError) -> Self {
        AppError::from_debug(e)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::from_debug(e)
    }
}

impl From<lapin::Error> for AppError {
    fn from(e: lapin::Error) -> Self {
        AppError::from_debug(e)
    }
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        Self { msg }
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        Self { msg: msg.into() }
    }
}

impl From<fy_base::api::bm_api::ApiError> for AppError {
    fn from(e: fy_base::api::bm_api::ApiError) -> Self {
        AppError::from_debug(e)
    }
}

impl From<S3Error> for AppError {
    fn from(e: S3Error) -> Self {
        AppError::from_debug(e)
    }
}
