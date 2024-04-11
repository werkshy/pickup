use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub struct AppError {
    pub msg: String,
    pub status: StatusCode,
}

impl AppError {
    pub fn new(msg: &str, status: StatusCode) -> AppError {
        AppError {
            msg: msg.to_string(),
            status,
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
        let err_json = serde_json::json!({ "error": self.msg });
        HttpResponse::build(self.status).json(err_json)
    }
}
