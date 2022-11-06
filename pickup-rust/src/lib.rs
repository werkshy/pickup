pub mod api;
pub mod app_state;
pub mod filemanager;
pub mod player;

use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{App, HttpServer};

use app_state::AppState;
use filemanager::collection;
use filemanager::options::CollectionOptions;
use player::{Command, Player};

// Enable assert_matches in tests
#[cfg(test)]
#[macro_use]
extern crate assert_matches;

#[derive(Clone)]
pub struct ServeOptions {
    pub collection_options: CollectionOptions,
    pub port: u32,
}

pub async fn serve(options: ServeOptions) -> std::io::Result<()> {
    let sender = spawn_player();
    let collection = collection::init(options.collection_options.clone()).unwrap();
    let collection_arc = Arc::new(collection);

    let address = format!("0.0.0.0:{}", options.port);
    log::info!("Starting on http://{}", address);
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                // Note - we have to call .clone() within this `move` block so that each worker gets it's own clone of
                // the channel.
                // https://docs.rs/actix-web/4.0.1/actix_web/struct.App.html#shared-mutable-state
                sender: sender.clone(),
                collection: collection_arc.clone(),
            }))
            .wrap(Logger::default())
            .service(api::hello)
            .service(api::control::play)
            .service(api::control::stop)
            .service(api::control::volume)
            .service(api::list::list_categories)
    })
    .workers(2)
    .bind(address.as_str())?
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
