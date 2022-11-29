pub mod api;
pub mod app_state;
pub mod filemanager;
pub mod player;

use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceResponse},
    web::Data,
    Error,
};
use actix_web::{dev::ServiceRequest, middleware::Logger};
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

/**
 * Create the app state we need to pass into the app.
 */
pub fn build_app_state(options: &ServeOptions) -> AppState {
    let sender = spawn_player();
    let collection = collection::init(options.collection_options.clone()).unwrap();
    let collection_arc = Arc::new(collection);
    AppState {
        sender,
        collection: collection_arc,
    }
}

/**
 * It's not well documented how to return an App from a function, but there is a test that shows how:
 * https://github.com/actix/actix-web/blob/b1c85ba85be91b5ea34f31264853b411fadce1ef/actix-web/src/app.rs#L698
 */
pub fn build_app(
    app_state: AppState,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
> {
    App::new()
        .app_data(Data::new(app_state))
        .wrap(Logger::default())
        .service(api::hello)
        .service(api::control::play)
        .service(api::control::stop)
        .service(api::control::volume)
        .service(api::list::list_categories)
}

pub async fn serve(options: ServeOptions) -> std::io::Result<()> {
    let app_state = build_app_state(&options);

    let address = format!("0.0.0.0:{}", options.port);
    log::info!("Starting on http://{}", address);
    HttpServer::new(move || build_app(app_state.clone()))
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
