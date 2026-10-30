use std::path::Path;

use actix_web::{get, post, web, Responder};

use crate::{
    api::types::{ErrorResponse, PlaybackStatusResponse, StatusResponse, VolumeResponse},
    app_state::AppState,
    error::AppError,
    player::{PlayerClient, PlayerError},
};

/// Play a single track, by ID.
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

    // TODO shouldn't the path be absolute or relative already? Or maybe the Player needs to know the prefix
    // Joined as a Path so a track whose name isn't valid UTF-8 still resolves.
    let path = Path::new(&data.collection.dir).join(&track.path);

    data.player.play(path).await.map_err(AppError::from)?;
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
