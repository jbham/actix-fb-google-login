use actix_web::error::ResponseError;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use color_eyre::Report;
use serde::export::Formatter;
use serde::{Serialize, Serializer};
use std::{convert::From, error::Error};
use tracing::error;

#[derive(Debug, Serialize)]
pub struct AppError {
    message: String,
    code: AppErrorCode,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AppErrorCode(i32);

impl AppErrorCode {
    pub fn message(self, _message: String) -> AppError {
        AppError {
            message: _message,
            code: self,
        }
    }

    pub fn default(self) -> AppError {
        let message = match self {
            AppError::INVALID_INPUT => "Invalid input.",
            AppError::INVALID_CREDENTIALS => "Invalid username or password provided",
            AppError::CREDENTIAL_EXPIRED => "Expired Credentials",
            AppError::INVALID_CHOICE => "Invalid choice sent",
            AppError::ALREADY_VOTED => "Already voted",
            AppError::NOT_AUTHORIZED => "Not authorized.",
            AppError::NOT_FOUND => "Item not found.",
            _ => "An unexpected error has occurred.",
        };
        AppError {
            message: message.to_string(),
            code: self,
        }
    }
}

impl From<AppErrorCode> for AppError {
    fn from(error: AppErrorCode) -> Self {
        error.default()
    }
}

impl AppError {
    pub const INTERNAL_ERROR: AppErrorCode = AppErrorCode(1001);
    pub const INVALID_INPUT: AppErrorCode = AppErrorCode(2001);
    pub const INVALID_CREDENTIALS: AppErrorCode = AppErrorCode(3001);
    pub const NOT_AUTHORIZED: AppErrorCode = AppErrorCode(3002);
    pub const CREDENTIAL_EXPIRED: AppErrorCode = AppErrorCode(3003);
    pub const INVALID_CHOICE: AppErrorCode = AppErrorCode(3004);
    pub const ALREADY_VOTED: AppErrorCode = AppErrorCode(3005);
    pub const NOT_FOUND: AppErrorCode = AppErrorCode(4001);
}

impl Serialize for AppErrorCode {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.0)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        error!("{:?}", e);
        Self::INTERNAL_ERROR.message("An unexpected error ocurred.".to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        error!("{:?}", e);
        Self::INTERNAL_ERROR.message("An unexpected error ocurred.".to_string())
    }
}

impl From<Box<dyn Error>> for AppError {
    fn from(e: Box<dyn Error>) -> Self {
        error!("{:?}", e);
        Self::INTERNAL_ERROR.message("An unexpected error ocurred.".to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        error!("{:?}", e);
        Self::INTERNAL_ERROR.message("An unexpected error ocurred.".to_string())
    }
}

impl From<Report> for AppError {
    fn from(e: Report) -> Self {
        error!("{:?}", e);
        Self::INTERNAL_ERROR.message("An unexpected error ocurred.".to_string())
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self.code {
            AppError::INVALID_INPUT => StatusCode::BAD_REQUEST,
            AppError::NOT_FOUND => StatusCode::NOT_FOUND,
            AppError::INVALID_CREDENTIALS => StatusCode::UNAUTHORIZED,
            AppError::NOT_AUTHORIZED => StatusCode::UNAUTHORIZED,
            AppError::CREDENTIAL_EXPIRED => StatusCode::UNAUTHORIZED,
            AppError::INVALID_CHOICE => StatusCode::BAD_REQUEST,
            AppError::ALREADY_VOTED => StatusCode::BAD_REQUEST,

            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}
