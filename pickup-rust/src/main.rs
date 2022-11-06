mod cli;

use std::io::Write;

use clap::Parser;
use env_logger::Env;

use cli::{Cli, Commands};
use pickup::filemanager::options::CollectionOptions;
use pickup::filemanager::{cache, list::list};
use pickup::{serve, ServeOptions};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // TODO: enable a different log format in prod (timestamps, maybe JSON?)
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();

    let cli = Cli::parse();

    let music_dir: &str = cli.music_dir.as_str();

    let collection_options = CollectionOptions {
        dir: music_dir.to_string(),
        ignores: None,
    };

    match cli.command {
        Commands::List {} => list(collection_options),
        Commands::Refresh {} => {
            let result = cache::refresh(collection_options.clone());
            result.map(|_| ())
        }
        Commands::Serve { port } => {
            serve(ServeOptions {
                collection_options: collection_options.clone(),
                port,
            })
            .await
        }
    }
}
