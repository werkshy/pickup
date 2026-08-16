use std::sync::mpsc::Sender;

use actix_web::{get, post, web, HttpResponse, Responder};

use crate::{app_state::AppState, player::Command};
use crate::{
    filemanager::model::Track,
    player::commands::{GetVolumeCommand, PlayCommand, StopCommand, VolumeCommand},
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

    let command = Box::new(PlayCommand { file: path }) as Box<dyn Command>;
    let _ = data.player_sender.send(command);
    HttpResponse::Ok().body("ok")
}

#[post("/stop")]
pub async fn stop(data: web::Data<AppState>) -> impl Responder {
    let command = Box::new(StopCommand {}) as Box<dyn Command>;
    let _ = data.player_sender.send(command);
    HttpResponse::Ok().body("ok")
}

#[post("/volume/{volume}")]
pub async fn volume(data: web::Data<AppState>, volume: web::Path<u8>) -> impl Responder {
    let clamped_volume = volume.into_inner().clamp(0, 100);
    let command = Box::new(VolumeCommand {
        volume: clamped_volume,
    }) as Box<dyn Command>;
    let _ = data.player_sender.send(command);

    let new_volume = _get_volume(data.player_sender.clone()).await;
    HttpResponse::Ok().json(serde_json::json!({ "volume": new_volume }))
}

/**
 * This demonstrates how to get info out of the Player thread. We create a
 * short-lived channel pair and pass the sender into the player thread then
 * wait for a response. The whole thing is wrapped in web::block() since it's
 * synchronous code that will block until it receives a response.
 */
#[get("/volume")]
pub async fn get_volume(data: web::Data<AppState>) -> impl Responder {
    let data = data.clone();
    let current_volume = _get_volume(data.player_sender.clone()).await;
    HttpResponse::Ok().json(serde_json::json!({ "volume": current_volume }))
}

async fn _get_volume(player_sender: Sender<Box<dyn Command>>) -> u8 {
    let current_volume = web::block(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let command = Box::new(GetVolumeCommand { reply: tx }) as Box<dyn Command>;
        let _ = player_sender.send(command);
        rx.recv().unwrap()
    })
    .await
    .unwrap_or(0);
    current_volume
}
