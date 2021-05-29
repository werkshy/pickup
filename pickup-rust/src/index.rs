use actix_web::{get, post, web, HttpResponse, Responder};

use crate::app_state::AppState;

#[get("/")]
async fn hello() -> impl Responder {
    return HttpResponse::Ok().body("Hello");
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[post("/play")]
async fn play(data: web::Data<AppState>) -> impl Responder {
    let sender = data.sender.lock().unwrap();
    let _ = sender.send(String::from("play"));
    return HttpResponse::Ok().body("ok");
}

#[post("/stop")]
async fn stop(data: web::Data<AppState>) -> impl Responder {
    let sender = data.sender.lock().unwrap();
    let _ = sender.send(String::from("stop"));
    return HttpResponse::Ok().body("ok");
}
