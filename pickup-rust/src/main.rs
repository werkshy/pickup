use actix_web::web::Data;
use actix_web::{App, HttpServer};
use player::Command;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

mod api;
mod app_state;
mod player;

use app_state::AppState;
use env_logger::Env;
use player::Player;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let sender = spawn_player();

    log::info!("Starting on http://localhost:9090");
    HttpServer::new(move || {
        log::info!("Building app");
        App::new()
            .app_data(Data::new(AppState {
                // Note - we have to call .clone() within this `move` block so that each worker gets it's own clone of
                // the channel.
                // https://docs.rs/actix-web/4.0.1/actix_web/struct.App.html#shared-mutable-state
                sender: sender.clone(),
            }))
            .service(api::hello)
            .service(api::control::play)
            .service(api::control::stop)
            .service(api::control::volume)
    })
    .bind("127.0.0.1:9090")?
    .shutdown_timeout(60) // <- Set shutdown timeout to 60 seconds
    .run()
    .await
}

fn spawn_player() -> Sender<Box<dyn Command>> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut player = Player::new();

        for command in rx {
            player.command(command);
        }
    });
    tx
}
