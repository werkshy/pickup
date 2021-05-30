use actix_web::middleware::Logger;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use clap::Parser;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

mod api;
mod app_state;
mod filemanager;
mod player;

use app_state::AppState;
use env_logger::Env;
use filemanager::{load, refresh, MusicDb};
use player::{Command, Player};

const DEFAULT_MUSIC_DIR: &str = "../music";

// Clap CLI definition struct
// TODO:
// Can change the music-dir arg to PathBuf
// Add subcommands e.g.
//    serve
//    db refresh
//    db list
//    db search
//    add
//    play

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Set the root music directory
    #[arg(short, long, value_name = "DIR", default_value_t = DEFAULT_MUSIC_DIR.to_string())]
    music_dir: String,

    /// Refresh the music files (can be slow on network drives)
    #[arg(short, long)]
    refresh: bool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    let music_dir = cli.music_dir;

    let files: MusicDb;
    if cli.refresh {
        files = refresh(String::from(music_dir)).unwrap();
    } else {
        files = load(String::from(music_dir)).unwrap();
    }
    log::info!("We have got {} files", files.len());

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
            .wrap(Logger::default())
            .service(api::hello)
            .service(api::control::play)
            .service(api::control::stop)
            .service(api::control::volume)
    })
    .workers(2)
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
