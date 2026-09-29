use actix_web::{get, web, Responder};

use crate::api::types::IndexResponse;

/// Identify the server and its version.
#[utoipa::path(
    tag = "index",
    responses(
        (status = 200, description = "Server info", body = IndexResponse)
    )
)]
#[get("/")]
pub async fn hello() -> impl Responder {
    web::Json(IndexResponse {
        name: "pickup".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
