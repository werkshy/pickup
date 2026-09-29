use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use std::fmt;

use crate::api::types::{ApiErrorCode, ErrorResponse};
use crate::player::PlayerError;

#[derive(Debug)]
pub struct AppError {
    pub code: ApiErrorCode,
    pub msg: String,
    pub status: StatusCode,
}

impl AppError {
    pub fn new(code: ApiErrorCode, msg: impl Into<String>, status: StatusCode) -> AppError {
        AppError {
            code,
            msg: msg.into(),
            status,
        }
    }

    pub fn not_found(msg: impl Into<String>) -> AppError {
        AppError::new(ApiErrorCode::NotFound, msg, StatusCode::NOT_FOUND)
    }

    pub fn internal(msg: impl Into<String>) -> AppError {
        AppError::new(
            ApiErrorCode::Internal,
            msg,
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    }
}

/**
 * 503 when the Player thread is gone (there is no in-process recovery, so
 * every player request fails until the server is restarted), 500 when it is
 * alive but not answering in time or the action itself failed (e.g. the
 * track's file is missing).
 */
impl From<PlayerError> for AppError {
    fn from(error: PlayerError) -> AppError {
        match error {
            PlayerError::Dead => AppError::new(
                ApiErrorCode::PlayerUnavailable,
                "player is unavailable",
                StatusCode::SERVICE_UNAVAILABLE,
            ),
            PlayerError::Timeout | PlayerError::Aborted => AppError::new(
                ApiErrorCode::PlayerTimeout,
                "player did not respond in time",
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            PlayerError::Failed(error) => AppError::new(
                ApiErrorCode::PlaybackFailed,
                format!("playing failed: {}", error),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for AppError {
    // builds the actual response to send back when an error occurs
    fn error_response(&self) -> HttpResponse {
        let body = ErrorResponse {
            code: self.code,
            message: self.msg.clone(),
        };
        HttpResponse::build(self.status).json(body)
    }
}
