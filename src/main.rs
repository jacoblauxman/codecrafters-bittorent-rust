use bittorrent_starter_rust::{
    commands::{Args, Commands},
    decode::{decode_bencoded_value, BencodeError},
    torrent::Torrent,
};
use clap::Parser;
use sha1::Digest;

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
            let bytes = std::fs::read(path).map_err(|_| {
                BencodeError::DataFormat(format!(
                    "the provided file path `{}` does not exist.",
                    torrent_file
                ))
            })?;

            match decode_bencoded_value(&bytes) {
                Ok((data, _)) => {
                    // println!("data: {}\n\n", data);
                    let torrent: Torrent = serde_json::from_value(data).unwrap();
                    let info = &torrent.info;
                    let bencoded_info_bytes = serde_bencode::to_bytes(&info)
                        .expect("should re-encode torrent's `info` field");

                    let mut hasher = sha1::Sha1::new();
                    hasher.update(&bencoded_info_bytes);
                    let hash_res = hasher.finalize();

                    let hex_hash_info = hex::encode(hash_res);

                    println!("Tracker URL: {}", torrent.announce);
                    println!("Length: {}", torrent.info.length);
                    println!("Info Hash: {}", hex_hash_info);
                    println!("Piece Length: {}", torrent.info.piece_length);
                    println!("Piece Hashses:");
                    for piece in torrent.info.pieces.chunks(20) {
                        println!("{}", hex::encode(piece));
                    }
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
