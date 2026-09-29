use actix_web::{get, post, web, Responder};

use crate::{
    api::types::{ErrorResponse, StatusResponse, VolumeResponse},
    app_state::AppState,
    error::AppError,
    filemanager::model::Track,
    player::{PlayerClient, PlayerError},
};

fn get_first_track(app_state: &AppState) -> &Track {
    // TODO for now let's just look for the first track, it seems to work on our demo music
    return app_state
        .collection
        .values()
        .next()
        .unwrap()
        .artists
        .values()
        .next()
        .unwrap()
        .albums
        .values()
        .next()
        .unwrap()
        .tracks
        .first()
        .unwrap();
}

/// Start playback.
///
/// TODO for now plays the first track, it seems to work on our demo music.
#[utoipa::path(
    tag = "control",
    responses(
        (status = 200, description = "Playback started", body = StatusResponse),
        (status = 500, description = "The player did not respond in time or the action itself failed", body = ErrorResponse),
        (status = 503, description = "The player thread is gone", body = ErrorResponse),
    )
)]
#[post("/play")]
pub async fn play(data: web::Data<AppState>) -> Result<impl Responder, AppError> {
    let track = get_first_track(&data);
    // TODO shouldn't the path be absolute or relative already? Or maybe the Player needs to know the prefix
    let path = format!("../music/{}", track.path.as_os_str().to_str().unwrap());

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
