use bittorrent_starter_rust::{
    commands::{Args, Commands},
    decode_bencoded_value, BencodeError,
};
use clap::Parser;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> Result<(), BencodeError> {
    let args = Args::parse();

    match &args.command {
        Commands::Decode { encoded_value } => {
            let (decoded_value, _) = decode_bencoded_value(&encoded_value)?;
            println!("{}", decoded_value.to_string());
        }
    }

    Ok(())
}
