use actix_web::{get, post, web, Responder};

use crate::{
    api::types::{ErrorResponse, PlaybackStatusResponse, StatusResponse, VolumeResponse},
    app_state::AppState,
    error::AppError,
    player::{PlayOutcome, PlayerClient, PlayerError},
};

/// Play a single track, by ID, replacing anything in the queue.
#[utoipa::path(
    tag = "control",
    responses(
        (status = 200, description = "Playback started", body = StatusResponse),
        (status = 404, description = "No track with this ID", body = ErrorResponse),
        (status = 500, description = "The player did not respond in time or the action itself failed", body = ErrorResponse),
        (status = 503, description = "The player thread is gone", body = ErrorResponse),
    )
)]
#[post("/play/{id}")]
pub async fn play(
    data: web::Data<AppState>,
    id: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = id.into_inner();
    let Some(track) = data.collection.get_track(&id) else {
        return Err(AppError::not_found(format!(
            "No track found with ID '{id}'"
        )));
    };

    // Validate before touching anything: a track that cannot be played must
    // not stop the current playback or replace the queue.
    data.player
        .check_playable(&track.path)
        .await
        .map_err(AppError::from)?;

    // Stop first: if the old track finished between the queue mutation and
    // the new playback starting, it would advance past the new queue's
    // first track.
    data.player.stop().await.map_err(AppError::from)?;
    data.queue
        .write()
        .unwrap()
        .replace_with(vec![track.clone()]);

    // Tell the player to play the new queue.
    data.player
        .play_from_queue()
        .await
        .map_err(AppError::from)?;
    Ok(web::Json(StatusResponse {
        status: "ok".to_string(),
    }))
}

/// Resume playback from the front of the queue. Does nothing if the
/// player is already playing.
#[utoipa::path(
    tag = "control",
    responses(
        (status = 200, description = "Playback started (or was already running)", body = StatusResponse),
        (status = 400, description = "The queue is empty or has played out", body = ErrorResponse),
        (status = 500, description = "The player did not respond in time", body = ErrorResponse),
        (status = 503, description = "The player thread is gone", body = ErrorResponse),
    )
)]
#[post("/play")]
pub async fn resume(data: web::Data<AppState>) -> Result<impl Responder, AppError> {
    let outcome = data
        .player
        .play_queue_if_idle()
        .await
        .map_err(AppError::from)?;

    match outcome {
        PlayOutcome::NothingQueued => Err(AppError::invalid_request(
            "Nothing to play: the queue is empty or has played out",
        )),
        PlayOutcome::Started | PlayOutcome::AlreadyPlaying => Ok(web::Json(StatusResponse {
            status: "ok".to_string(),
        })),
    }
}

/// Skip to the next track in the queue. No-op when nothing is playing.
#[utoipa::path(
    tag = "control",
    responses(
        (status = 200, description = "Skipped to the next track (or nothing was playing)", body = StatusResponse),
        (status = 500, description = "The player did not respond in time", body = ErrorResponse),
        (status = 503, description = "The player thread is gone", body = ErrorResponse),
    )
)]
#[post("/skip")]
pub async fn skip(data: web::Data<AppState>) -> Result<impl Responder, AppError> {
    data.player.skip().await.map_err(AppError::from)?;
    Ok(web::Json(StatusResponse {
        status: "ok".to_string(),
    }))
}

/// Stop playback.
#[utoipa::path(
    tag = "control",
    responses(
        (status = 200, description = "Playback stopped", body = StatusResponse),
        (status = 500, description = "The player did not respond in time", body = ErrorResponse),
        (status = 503, description = "The player thread is gone", body = ErrorResponse),
    )
)]
#[post("/stop")]
pub async fn stop(data: web::Data<AppState>) -> Result<impl Responder, AppError> {
    data.player.stop().await.map_err(AppError::from)?;
    Ok(web::Json(StatusResponse {
        status: "ok".to_string(),
    }))
}

/// Set the volume (0-100) and get the value that was actually set back.
#[utoipa::path(
    tag = "control",
    responses(
        (status = 200, description = "The new volume", body = VolumeResponse),
        (status = 500, description = "The player did not respond in time", body = ErrorResponse),
        (status = 503, description = "The player thread is gone", body = ErrorResponse),
    )
)]
#[post("/volume/{volume}")]
pub async fn volume(
    data: web::Data<AppState>,
    volume: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let clamped_volume = volume.into_inner().clamp(0, 100) as u8;
    let new_volume = data
        .player
        .set_volume(clamped_volume)
        .await
        .map_err(AppError::from)?;
    Ok(web::Json(VolumeResponse { volume: new_volume }))
}

/// Get the current playback status.
#[utoipa::path(
    tag = "control",
    responses(
        (status = 200, description = "The current playback status", body = PlaybackStatusResponse),
        (status = 500, description = "The player did not respond in time", body = ErrorResponse),
        (status = 503, description = "The player thread is gone", body = ErrorResponse),
    )
)]
#[get("/status")]
pub async fn status(data: web::Data<AppState>) -> Result<impl Responder, AppError> {
    let status = data.player.status().await.map_err(AppError::from)?;
    Ok(web::Json(PlaybackStatusResponse::from(status)))
}

/// Get the current volume.
#[utoipa::path(
    tag = "control",
    responses(
        (status = 200, description = "The current volume", body = VolumeResponse),
        (status = 500, description = "The player did not respond in time", body = ErrorResponse),
        (status = 503, description = "The player thread is gone", body = ErrorResponse),
    )
)]
#[get("/volume")]
pub async fn get_volume(data: web::Data<AppState>) -> Result<impl Responder, AppError> {
    let current_volume = get_current_volume(&data.player)
        .await
        .map_err(AppError::from)?;
    Ok(web::Json(VolumeResponse {
        volume: current_volume,
    }))
}

async fn get_current_volume(player: &PlayerClient) -> Result<u8, PlayerError> {
    player.request_async(|player| player.get_volume()).await
}
