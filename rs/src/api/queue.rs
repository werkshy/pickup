use actix_web::{get, post, web, Responder, Result};

use crate::{
    api::types::{ApiQueueInput, ApiQueueResponse, ApiTrack, ErrorResponse},
    app_state::AppState,
    error::AppError,
};

/// Add music to the queue: a single track for a track ID, or every track under
/// a folder for a category, artist, album or disc ID.
#[utoipa::path(
    tag = "queue",
    request_body = ApiQueueInput,
    responses(
        (status = 200, description = "The queue after adding", body = ApiQueueResponse),
        (status = 400, description = "The request body is not valid", body = ErrorResponse),
        (status = 404, description = "No matching music found in the collection", body = ErrorResponse),
    )
)]
#[post("/queue/add")]
pub async fn add(
    data: web::Data<AppState>,
    input: web::Json<ApiQueueInput>,
) -> Result<impl Responder, AppError> {
    let Some(tracks) = data.collection.get_tracks(&input.id) else {
        return Err(AppError::not_found(
            "No matching music found in the collection",
        ));
    };

    log::info!("Adding {} tracks to the queue", tracks.len());
    let mut queue = data.queue.write().unwrap();
    if input.clear {
        queue.clear();
    }
    queue.add_tracks(tracks.into_iter().cloned().collect());
    Ok(web::Json(ApiQueueResponse {
        tracks: queue.tracks.iter().map(ApiTrack::from_track).collect(),
        position: queue.position,
    }))
}

/// Clear the queue.
#[utoipa::path(
    tag = "queue",
    responses(
        (status = 200, description = "The empty queue", body = ApiQueueResponse),
    )
)]
#[post("/queue/clear")]
pub async fn clear(data: web::Data<AppState>) -> impl Responder {
    let mut queue = data.queue.write().unwrap();
    queue.clear();
    web::Json(ApiQueueResponse {
        tracks: queue.tracks.iter().map(ApiTrack::from_track).collect(),
        position: queue.position,
    })
}

/// Get the current queue.
#[utoipa::path(
    tag = "queue",
    responses(
        (status = 200, description = "The current queue", body = ApiQueueResponse),
    )
)]
#[get("/queue")]
pub async fn get_queue(data: web::Data<AppState>) -> impl Responder {
    let queue = data.queue.read().unwrap();
    web::Json(ApiQueueResponse {
        tracks: queue.tracks.iter().map(ApiTrack::from_track).collect(),
        position: queue.position,
    })
}
