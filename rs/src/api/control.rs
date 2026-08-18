use actix_web::{get, post, web, HttpResponse, Responder};

use crate::{
    app_state::AppState,
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

#[post("/play")]
pub async fn play(data: web::Data<AppState>) -> impl Responder {
    // TODO for now let's just look for the first track, it seems to work on our demo music
    let track = get_first_track(&data);
    // TODO shouldn't the path be absolute or relative already? Or maybe the Player needs to know the prefix
    let path = format!("../music/{}", track.path.as_os_str().to_str().unwrap());

    match data.player.play(path).await {
        Ok(()) => HttpResponse::Ok().body("ok"),
        Err(error) => player_error_response(error),
    }
}

#[post("/stop")]
pub async fn stop(data: web::Data<AppState>) -> impl Responder {
    match data.player.stop().await {
        Ok(()) => HttpResponse::Ok().body("ok"),
        Err(error) => player_error_response(error),
    }
}

#[post("/volume/{volume}")]
pub async fn volume(data: web::Data<AppState>, volume: web::Path<u8>) -> impl Responder {
    let clamped_volume = volume.into_inner().clamp(0, 100);
    match data.player.set_volume(clamped_volume).await {
        Ok(new_volume) => HttpResponse::Ok().json(serde_json::json!({ "volume": new_volume })),
        Err(error) => player_error_response(error),
    }
}

#[get("/volume")]
pub async fn get_volume(data: web::Data<AppState>) -> impl Responder {
    match get_current_volume(&data.player).await {
        Ok(current_volume) => {
            HttpResponse::Ok().json(serde_json::json!({ "volume": current_volume }))
        }
        Err(error) => player_error_response(error),
    }
}

async fn get_current_volume(player: &PlayerClient) -> Result<u8, PlayerError> {
    player.request_async(|player| player.get_volume()).await
}

/**
 * 503 when the Player thread is gone (there is no in-process recovery, so
 * every player request fails until the server is restarted), 500 when it is
 * alive but not answering in time or the action itself failed (e.g. the
 * track's file is missing).
 */
fn player_error_response(error: PlayerError) -> HttpResponse {
    match error {
        PlayerError::Dead => HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({ "error": "player is unavailable" })),
        PlayerError::Timeout | PlayerError::Aborted => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": "player did not respond in time" })),
        PlayerError::Failed(error) => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": format!("playing failed: {}", error) })),
    }
}
