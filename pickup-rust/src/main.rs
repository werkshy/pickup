use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use clap::Parser;
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

mod api;
mod app_state;
mod cli;
mod filemanager;
mod player;

use app_state::AppState;
use cli::{Cli, Commands};
use env_logger::Env;
use filemanager::cache;
use filemanager::cache::CacheOptions;
use player::{Command, Player};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

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

// Temp, just for list until we have something more sophisticated
fn get_parts(path: &Path) -> Vec<String> {
    path.components()
        .map(|component| component.as_os_str().to_str().unwrap().to_string())
        .collect()
}

fn list(cache_options: CacheOptions) -> std::io::Result<()> {
    let files = cache::init(cache_options.clone()).unwrap();
    log::info!("We have got {} files", files.len());

    let root_parts = get_parts(Path::new(cache_options.dir.as_str()));

    for file in files {
        let parts = get_parts(file.as_path());
        let relative_parts = parts.strip_prefix(root_parts.as_slice()).unwrap();
        let num_parts: usize;
        // Figure out what is left after the category
        if relative_parts[0].starts_with("_") {
            num_parts = relative_parts.len() - 1;
        } else {
            num_parts = relative_parts.len();
        }
        if num_parts > 3 {
            log::info!("{} {:?}", num_parts, file);
        }
    }

    return Ok(());
}

struct ServeOptions {
    cache_options: CacheOptions,
    port: u32,
}

async fn serve(options: ServeOptions) -> std::io::Result<()> {
    let sender = spawn_player();

    let address = format!("127.0.0.1:{}", options.port);
    log::info!("Starting on http://{}", address);
    HttpServer::new(move || {
        log::info!("Building app");
        App::new()
            .app_data(Data::new(AppState {
                // Note - we have to call .clone() within this `move` block so that each worker gets it's own clone of
                // the channel.
                // https://docs.rs/actix-web/4.0.1/actix_web/struct.App.html#shared-mutable-state
                sender: sender.clone(),
            }))
            .wrap(Logger::default())
            .service(api::hello)
            .service(api::control::play)
            .service(api::control::stop)
            .service(api::control::volume)
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
