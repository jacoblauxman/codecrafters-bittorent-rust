use bittorrent_starter_rust::{
    commands::{Args, Commands},
    decode::{decode_bencoded_value, BencodeError},
};
use clap::Parser;

fn main() -> Result<(), BencodeError> {
    let args = Args::parse();

    match &args.command {
        Commands::Decode { encoded_value } => match decode_bencoded_value(encoded_value.as_bytes())
        {
            Ok((decoded_value, _)) => println!("{}", decoded_value),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        },
        Commands::Info { torrent_file } => {
            let path = std::path::Path::new(torrent_file);
            let bytes = std::fs::read(path).unwrap();

            // let bytes_str = String::from_utf8_lossy(&bytes);
            // println!("bytes_str: {}", bytes_str);

            match decode_bencoded_value(&bytes) {
                Ok((data, _)) => {
                    // println!("{}", data.clone());

                    let tracker_url = data["announce"].as_str().unwrap();
                    let length = &data["info"]["length"];

                    println!("Tracker URL: {}", tracker_url);
                    println!("Length: {}", length);
                }
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
