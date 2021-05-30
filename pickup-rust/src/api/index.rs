use actix_web::{get, HttpResponse, Responder};

#[get("/")]
pub async fn hello() -> impl Responder {
    return HttpResponse::Ok().body("Hello");
}
