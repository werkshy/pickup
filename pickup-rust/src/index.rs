use actix_web::{get, post, web, HttpResponse, Responder};

use crate::player::{PlayCommand, StopCommand, VolumeCommand};
use crate::{app_state::AppState, player::Command};

#[get("/")]
pub async fn hello() -> impl Responder {
    return HttpResponse::Ok().body("Hello");
}

#[post("/play")]
pub async fn play(data: web::Data<AppState>) -> impl Responder {
    let mp3_file = "../music/Kevin MacLeod/Album 1/01 - Lukewarm Banjo.mp3";
    let command = Box::new(PlayCommand {
        file: String::from(mp3_file),
    }) as Box<dyn Command>;
    let _ = data.sender.send(command);
    return HttpResponse::Ok().body("ok");
}

#[post("/stop")]
pub async fn stop(data: web::Data<AppState>) -> impl Responder {
    let command = Box::new(StopCommand {}) as Box<dyn Command>;
    let _ = data.sender.send(command);
    return HttpResponse::Ok().body("ok");
}

#[post("/volume/{volume}")]
pub async fn volume(data: web::Data<AppState>, volume: web::Path<f32>) -> impl Responder {
    let command = Box::new(VolumeCommand {
        volume: volume.into_inner(),
    }) as Box<dyn Command>;
    let _ = data.sender.send(command);
    return HttpResponse::Ok().body("ok");
}
