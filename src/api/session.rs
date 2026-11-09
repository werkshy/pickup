use actix_web::{get, post, web, HttpRequest, Responder, Result};

use crate::{
    api::types::{
        ApiQueueInput, ApiQueueMoveInput, ApiQueuePosition, ApiQueueRemoveInput, ApiQueueResponse,
        ApiSessionNextResponse, ApiTrack, ErrorResponse,
    },
    app_state::AppState,
    error::AppError,
    filemanager::model::Track,
    session,
};

/*
 * The per-browser playback queue, addressed by the session cookie. Unlike
 * the jukebox queue, the browser owns playback: it reports when a track has
 * finished (or is skipped) via `/session/queue/next`, and the server only ever
 * pops the finished track then. The front of the queue is the track playing
 * right now, or the next one to play when the browser is idle.
 */
fn request_session(req: &HttpRequest) -> Result<String, AppError> {
    session::request_session_id(req)
}

fn queue_snapshot(state: &AppState, id: &str) -> Result<ApiQueueResponse, AppError> {
    state
        .sessions
        .read_queue(id, |queue| ApiQueueResponse {
            tracks: queue.tracks.iter().map(ApiTrack::from_track).collect(),
        })
        .ok_or_else(|| AppError::internal("Session not found"))
}

/**
 * The session queue operations, shared with the web UI actions (which the
 * browser drives through /web/... in browser playback mode). Each takes the
 * session id the session middleware resolved for the request.
 */
pub(crate) fn session_add(
    state: &AppState,
    id: &str,
    input: &ApiQueueInput,
) -> Result<(), AppError> {
    let collection = state.collection.get();
    let Some(tracks) = collection.get_tracks(&input.id) else {
        return Err(AppError::not_found(
            "No matching music found in the collection",
        ));
    };
    let tracks: Vec<_> = tracks.into_iter().cloned().collect();
    let play_now = input.play_now;

    state
        .sessions
        .update_queue(id, |queue| match input.position {
            ApiQueuePosition::Replace => queue.replace_with(tracks),
            ApiQueuePosition::Next => queue.insert_next(tracks),
            ApiQueuePosition::Tail => {
                if play_now && queue.tracks.is_empty() {
                    queue.insert_front(tracks);
                } else {
                    queue.add_tracks(tracks);
                }
            }
        })
        .ok_or_else(|| AppError::internal("Session not found"))?;
    Ok(())
}

pub(crate) fn session_remove(state: &AppState, id: &str, index: usize) -> Result<(), AppError> {
    let removed = state
        .sessions
        .update_queue(id, |queue| queue.remove(index))
        .ok_or_else(|| AppError::internal("Session not found"))?;
    if removed.is_none() {
        return Err(AppError::invalid_request(format!(
            "Index {index} is outside the queue"
        )));
    }
    Ok(())
}

pub(crate) fn session_move(
    state: &AppState,
    id: &str,
    from: usize,
    to: usize,
) -> Result<(), AppError> {
    let moved = state
        .sessions
        .update_queue(id, |queue| queue.move_track(from, to))
        .ok_or_else(|| AppError::internal("Session not found"))?;
    if !moved {
        return Err(AppError::invalid_request(format!(
            "Cannot move from {from} to {to}: outside the queue"
        )));
    }
    Ok(())
}

pub(crate) fn session_clear(state: &AppState, id: &str) -> Result<(), AppError> {
    state
        .sessions
        .update_queue(id, |queue| queue.clear())
        .ok_or_else(|| AppError::internal("Session not found"))?;
    Ok(())
}

/// Finish the track at the front: pop it and return the next track, if any.
pub(crate) fn session_next(state: &AppState, id: &str) -> Result<Option<Track>, AppError> {
    state
        .sessions
        .update_queue(id, |queue| queue.advance())
        .ok_or_else(|| AppError::internal("Session not found"))
}

/// Get this session's queue.
#[utoipa::path(
    tag = "session",
    responses(
        (status = 200, description = "The session's queue", body = ApiQueueResponse),
    )
)]
#[get("/session/queue")]
pub async fn get_queue(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let id = request_session(&req)?;
    Ok(web::Json(queue_snapshot(&data, &id)?))
}

/**
 * Add music to the session's queue: a single track for a track ID, or every
 * track under a folder for a category, artist, album or disc ID.
 *
 * position: tail (default) adds to the back of the queue; next puts the
 * tracks behind the one at the front (the track playing, or the next to
 * play), or at the front of an empty queue; replace clears the queue and
 * adds the new tracks.
 *
 * play_now: on an empty queue, the added tracks go to the front so the
 * browser can start playing them. Otherwise the browser decides whether to
 * start playing, so this only matters to it.
 */
#[utoipa::path(
    tag = "session",
    request_body = ApiQueueInput,
    responses(
        (status = 200, description = "The session's queue after adding", body = ApiQueueResponse),
        (status = 400, description = "The request body is not valid", body = ErrorResponse),
        (status = 404, description = "No matching music found in the collection", body = ErrorResponse),
    )
)]
#[post("/session/queue/add")]
pub async fn add(
    data: web::Data<AppState>,
    req: HttpRequest,
    input: web::Json<ApiQueueInput>,
) -> Result<impl Responder, AppError> {
    let id = request_session(&req)?;
    session_add(&data, &id, &input)?;
    Ok(web::Json(queue_snapshot(&data, &id)?))
}

/// Remove one track from the session's queue, by index. The browser handles
/// the removal of the track it is playing: the server just takes it off the
/// queue.
#[utoipa::path(
    tag = "session",
    request_body = ApiQueueRemoveInput,
    responses(
        (status = 200, description = "The session's queue after removing", body = ApiQueueResponse),
        (status = 400, description = "The index is outside the queue", body = ErrorResponse),
    )
)]
#[post("/session/queue/remove")]
pub async fn remove(
    data: web::Data<AppState>,
    req: HttpRequest,
    input: web::Json<ApiQueueRemoveInput>,
) -> Result<impl Responder, AppError> {
    let id = request_session(&req)?;
    session_remove(&data, &id, input.index)?;
    Ok(web::Json(queue_snapshot(&data, &id)?))
}

/// Move a track within the session's queue (reorder). The browser knows
/// whether the front is playing right now, so any positions may be moved.
#[utoipa::path(
    tag = "session",
    request_body = ApiQueueMoveInput,
    responses(
        (status = 200, description = "The session's queue after moving", body = ApiQueueResponse),
        (status = 400, description = "Either index is outside the queue", body = ErrorResponse),
    )
)]
#[post("/session/queue/move")]
pub async fn move_track(
    data: web::Data<AppState>,
    req: HttpRequest,
    input: web::Json<ApiQueueMoveInput>,
) -> Result<impl Responder, AppError> {
    let id = request_session(&req)?;
    session_move(&data, &id, input.from, input.to)?;
    Ok(web::Json(queue_snapshot(&data, &id)?))
}

/// Clear the session's queue. The browser stops playing whatever it was
/// playing, since it is no longer on the queue.
#[utoipa::path(
    tag = "session",
    responses(
        (status = 200, description = "The empty queue", body = ApiQueueResponse),
    )
)]
#[post("/session/queue/clear")]
pub async fn clear(
    data: web::Data<AppState>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let id = request_session(&req)?;
    session_clear(&data, &id)?;
    Ok(web::Json(queue_snapshot(&data, &id)?))
}

/**
 * Finish the track at the front of the session's queue: pop it and return
 * the next track, if any. The browser calls this when a track ends or is
 * skipped, then plays the returned track.
 */
#[utoipa::path(
    tag = "session",
    responses(
        (status = 200, description = "The next track to play, or null when the queue is exhausted", body = ApiSessionNextResponse),
    )
)]
#[post("/session/queue/next")]
pub async fn next(data: web::Data<AppState>, req: HttpRequest) -> Result<impl Responder, AppError> {
    let id = request_session(&req)?;
    let next = session_next(&data, &id)?;
    Ok(web::Json(ApiSessionNextResponse {
        track: next.map(|track| ApiTrack::from_track(&track)),
    }))
}
