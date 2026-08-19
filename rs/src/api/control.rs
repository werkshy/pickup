use actix_web::{get, post, web, HttpResponse, Responder};

use crate::{app_state::AppState, filemanager::model::Track, player::PlayerClient};

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

    data.player.play(path);
    HttpResponse::Ok().body("ok")
}

#[post("/stop")]
pub async fn stop(data: web::Data<AppState>) -> impl Responder {
    data.player.stop();
    HttpResponse::Ok().body("ok")
}

#[post("/volume/{volume}")]
pub async fn volume(data: web::Data<AppState>, volume: web::Path<u8>) -> impl Responder {
    let clamped_volume = volume.into_inner().clamp(0, 100);
    let new_volume = data.player.set_volume(clamped_volume).await.unwrap_or(0);
    HttpResponse::Ok().json(serde_json::json!({ "volume": new_volume }))
}

#[get("/volume")]
pub async fn get_volume(data: web::Data<AppState>) -> impl Responder {
    let current_volume = get_current_volume(&data.player).await;
    HttpResponse::Ok().json(serde_json::json!({ "volume": current_volume }))
}

async fn get_current_volume(player: &PlayerClient) -> u8 {
    player
        .request_async(|player| player.get_volume())
        .await
        .unwrap_or(0) // TODO better error handling? Return a 'player-is-dead' error?
}
