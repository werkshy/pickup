use clap::{Parser, Subcommand};

const DEFAULT_MUSIC_DIR: &str = "../music";
const DEFAULT_PORT: u32 = 9090;

// Clap CLI definition struct
// TODO:
// Can change the music-dir arg to PathBuf
// Allow overriding ignores
// Add subcommands e.g.
//    db search
//    client add
//    client play

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Set the root music directory
    #[arg(short, long, value_name = "DIR", default_value_t = DEFAULT_MUSIC_DIR.to_string())]
    pub music_dir: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    // List
    List {},

    // Refresh the music files (can be slow on network drives).
    Refresh {},

    /// Start the HTTP server
    Serve {
        #[arg(short, long, default_value_t = DEFAULT_PORT)]
        port: u32,
    },
}
