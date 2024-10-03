use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about , long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "decode provided bencoded string")]
    Decode {
        encoded_value: String,
    },
    Info {
        torrent_file: String,
    }, // ...
}
