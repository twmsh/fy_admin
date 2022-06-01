use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use tokio::task::JoinError;
use box_base::api::bm_api::ApiError;

pub type AppResult<T> = std::result::Result<T, AppError>;

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


impl From<tokio::task::JoinError> for AppError {
    fn from(e: JoinError) -> Self {
        AppError::from_debug(e)
    }
}

impl From<box_base::api::bm_api::ApiError> for AppError {
    fn from(e: ApiError) -> Self {
        AppError::from_debug(e)
    }
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        Self {
            msg,
        }
    }
}