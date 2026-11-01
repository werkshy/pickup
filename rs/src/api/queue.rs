use actix_web::{get, post, web, Responder, Result};

use crate::{
    api::types::{
        ApiQueueInput, ApiQueueMoveInput, ApiQueuePosition, ApiQueueRemoveInput, ApiQueueResponse,
        ApiTrack, ErrorResponse,
    },
    app_state::AppState,
    error::AppError,
};

/**
 * Add music to the queue: a single track for a track ID, or every track under
 * a folder for a category, artist, album or disc ID.
 *
 * position:
 *   - tail: add to the back of the queue (default)
 *   - next: after the currently-playing track, if there is one, else the front.
 *   - replace: stop the current track (if any), clear queue, add new tracks.
 * play_now: Start the queue playing if it is not already.
 */
#[utoipa::path(
    tag = "queue",
    request_body = ApiQueueInput,
    responses(
        (status = 200, description = "The queue after adding", body = ApiQueueResponse),
        (status = 400, description = "The request body is not valid", body = ErrorResponse),
        (status = 404, description = "No matching music found in the collection", body = ErrorResponse),
        (status = 500, description = "The player did not respond in time", body = ErrorResponse),
        (status = 503, description = "The player thread is gone", body = ErrorResponse),
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
    let status = data.player.status().await.map_err(AppError::from)?;
    let playing = status.active;

    // If we are replacing the queue, we should stop playing first. This is
    // neater and it also closes a race between the player finishing the current
    // track and us calling stop.
    if playing && input.position == ApiQueuePosition::Replace {
        data.player.stop().await.map_err(AppError::from)?;
    }

    let start_now = input.play_now && (!playing || input.position == ApiQueuePosition::Replace);

    // block to scope the RwLock
    {
        let mut queue = data.queue.write().unwrap();
        let tracks: Vec<_> = tracks.into_iter().cloned().collect();
        match input.position {
            ApiQueuePosition::Replace => queue.replace_with(tracks),
            // "Next" means behind the playing track, or at the front when the
            // player is idle.
            ApiQueuePosition::Next => {
                if playing {
                    queue.insert_next(tracks);
                } else {
                    queue.insert_front(tracks);
                }
            }
            ApiQueuePosition::Tail => queue.add_tracks(tracks),
        }
    }

    if start_now {
        data.player
            .play_queue_if_idle()
            .await
            .map_err(AppError::from)?;
    }

    Ok(web::Json(queue_response(&data)))
}

/// Remove one track from the queue, by index. Removing the track that is
/// playing (index 0) skips to the next track, or stops when it was the last.
#[utoipa::path(
    tag = "queue",
    request_body = ApiQueueRemoveInput,
    responses(
        (status = 200, description = "The queue after removing", body = ApiQueueResponse),
        (status = 400, description = "The index is outside the queue", body = ErrorResponse),
        (status = 500, description = "The player did not respond in time", body = ErrorResponse),
        (status = 503, description = "The player thread is gone", body = ErrorResponse),
    )
)]
#[post("/queue/remove")]
pub async fn remove(
    data: web::Data<AppState>,
    input: web::Json<ApiQueueRemoveInput>,
) -> Result<impl Responder, AppError> {
    let index = input.index;
    let playing = data.player.status().await.map_err(AppError::from)?.active;

    if playing && index == 0 {
        // Removing the playing track is exactly a skip: the Player pops it
        // off the front of the queue and starts the next one (or stops when
        // the queue has run out). The queue must not be mutated here: skip()
        // reads the front as the track it is discarding.
        data.player.skip().await.map_err(AppError::from)?;
        return Ok(web::Json(queue_response(&data)));
    }

    let removed = data.queue.write().unwrap().remove(index);
    if removed.is_none() {
        return Err(AppError::invalid_request(format!(
            "Index {index} is outside the queue"
        )));
    }
    Ok(web::Json(queue_response(&data)))
}

/// Move a track within the queue (reorder). While a track is playing, the
/// front of the queue is the track playing now, so position 0 can be neither
/// moved nor moved onto.
#[utoipa::path(
    tag = "queue",
    request_body = ApiQueueMoveInput,
    responses(
        (status = 200, description = "The queue after moving", body = ApiQueueResponse),
        (status = 400, description = "Either index is outside the queue, or position 0 was moved while playing", body = ErrorResponse),
        (status = 500, description = "The player did not respond in time", body = ErrorResponse),
        (status = 503, description = "The player thread is gone", body = ErrorResponse),
    )
)]
#[post("/queue/move")]
pub async fn move_track(
    data: web::Data<AppState>,
    input: web::Json<ApiQueueMoveInput>,
) -> Result<impl Responder, AppError> {
    let playing = data.player.status().await.map_err(AppError::from)?.active;
    if playing && (input.from == 0 || input.to == 0) {
        return Err(AppError::invalid_request(
            "Position 0 is the track playing now; it cannot be moved or replaced while playing",
        ));
    }

    let moved = data.queue.write().unwrap().move_track(input.from, input.to);
    if !moved {
        return Err(AppError::invalid_request(format!(
            "Cannot move from {} to {}: outside the queue",
            input.from, input.to
        )));
    }
    Ok(web::Json(queue_response(&data)))
}

/// Stop playback and clear queue.
#[utoipa::path(
    tag = "queue",
    responses(
        (status = 200, description = "The empty queue", body = ApiQueueResponse),
        (status = 500, description = "The player did not respond in time", body = ErrorResponse),
        (status = 503, description = "The player thread is gone", body = ErrorResponse),
    )
)]
#[post("/queue/clear")]
pub async fn clear(data: web::Data<AppState>) -> Result<impl Responder, AppError> {
    data.player.stop().await.map_err(AppError::from)?;
    data.queue.write().unwrap().clear();
    Ok(web::Json(queue_response(&data)))
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
    web::Json(queue_response(&data))
}

/// Snapshot the queue for an API response. Must be called while holding (or
/// re-acquiring) the queue lock.
fn queue_response(data: &web::Data<AppState>) -> ApiQueueResponse {
    let queue = data.queue.read().unwrap();
    ApiQueueResponse {
        tracks: queue.tracks.iter().map(ApiTrack::from_track).collect(),
    }
}
