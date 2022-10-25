use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use clap::Parser;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use std::{io::Write, sync::Arc};

mod api;
mod app_state;
mod cli;
mod filemanager;
mod player;

use app_state::AppState;
use cli::{Cli, Commands};
use env_logger::Env;
use filemanager::cache::CacheOptions;
use filemanager::{cache, list::list};
use player::{Command, Player};

// Enable assert_matches in tests
#[cfg(test)]
#[macro_use]
extern crate assert_matches;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // TODO: enable a different log format in prod (timestamps, maybe JSON?)
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();

    let cli = Cli::parse();

    let music_dir: &str = cli.music_dir.as_str();

    let cache_options = CacheOptions {
        dir: music_dir.to_string(),
        ignores: None,
    };

    match cli.command {
        Commands::List {} => {
            return list(cache_options);
        }
        Commands::Refresh {} => {
            let result = cache::refresh(cache_options.clone());
            return result.map(|_| ());
        }
        Commands::Serve { port } => {
            return serve(ServeOptions {
                cache_options: cache_options.clone(),
                port,
            })
            .await;
        }
    }
}

struct ServeOptions {
    cache_options: CacheOptions,
    port: u32,
}

async fn serve(options: ServeOptions) -> std::io::Result<()> {
    let sender = spawn_player();
    let collection = cache::init(options.cache_options.clone()).unwrap();
    let collection_arc = Arc::new(collection);

    let address = format!("0.0.0.0:{}", options.port);
    log::info!("Starting on http://{}", address);
    HttpServer::new(move || {
        log::info!("Building app");
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
