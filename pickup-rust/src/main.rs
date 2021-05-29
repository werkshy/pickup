use actix_web::{App, HttpServer};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::thread;

mod app_state;
mod index;
mod player;

use app_state::AppState;
use player::Player;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let sender = spawn_player();

    log::info!("Starting on http://localhost:9090");
    HttpServer::new(move || {
        log::info!("Building app");
        App::new()
            .data(AppState {
                sender: Mutex::new(sender.clone()),
            })
            .service(index::hello)
            .service(index::echo)
            .service(index::play)
            .service(index::stop)
    })
    .bind("127.0.0.1:9090")?
    .shutdown_timeout(60) // <- Set shutdown timeout to 60 seconds
    .run()
    .await
}

fn spawn_player() -> Sender<String> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut player = Player::new();

        for command in rx {
            player.command(command);
        }
    });
    tx
}
