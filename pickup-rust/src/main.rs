use actix_web::{App, HttpServer};
use player::Command;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

mod app_state;
mod index;
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
            .data(AppState {
                sender: sender.clone(),
            })
            .service(index::hello)
            .service(index::play)
            .service(index::stop)
            .service(index::volume)
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
