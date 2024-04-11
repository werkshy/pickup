use actix_web::{post, web, HttpResponse, Responder};

use crate::{app_state::AppState, player::Command};
use crate::{
    filemanager::model::Track,
    player::commands::{PlayCommand, StopCommand, VolumeCommand},
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
pub async fn volume(data: web::Data<AppState>, volume: web::Path<f32>) -> impl Responder {
    let command = Box::new(VolumeCommand {
        volume: volume.into_inner(),
    }) as Box<dyn Command>;
    let _ = data.player_sender.send(command);
    HttpResponse::Ok().body("ok")
}
