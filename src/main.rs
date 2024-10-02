use bittorrent_starter_rust::{
    commands::{Args, Commands},
    decode_bencoded_value, BencodeError,
};
use clap::Parser;

fn main() -> Result<(), BencodeError> {
    let args = Args::parse();

    match &args.command {
        Commands::Decode { encoded_value } => match decode_bencoded_value(encoded_value) {
            Ok((decoded_value, _)) => println!("{}", decoded_value.to_string()),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        },
    }

    Ok(())
}
