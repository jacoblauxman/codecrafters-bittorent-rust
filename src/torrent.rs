#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct Torrent {
    pub announce: String, // is a URL, could use more explicit type?? -- note: `reqwest::Url` does NOT impl `serde::Deserialize`
    pub info: TorrentInfo,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct TorrentInfo {
    pub length: i64, // size of file in bytes
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: i64, // num bytes each piece of file split into (`Chunks` size)
    #[serde(with = "serde_bytes")]
    pub pieces: Vec<u8>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::decode::decode_bencoded_value;
    use sha1::Digest;
    mod sample_torrent_file {
        use super::*;

        fn data() -> Vec<u8> {
            std::fs::read(std::path::Path::new("./sample.torrent"))
                .expect("should read sample.torrent file data")
        }

        #[test]
        fn valid_torrent_type_generated_from_file() {
            let encoded_value = data();

            let res = decode_bencoded_value(&encoded_value);

            assert!(res.is_ok());

            let val = res.unwrap().0;

            let torrent_res: Result<Torrent, _> = serde_json::from_value(val);
            assert!(torrent_res.is_ok());
        }

        #[test]
        fn generated_torrent_type_matches_decoded_sample_file_data() {
            let encoded_value = data();

            let res = decode_bencoded_value(&encoded_value);

            assert!(res.is_ok());

            let val = res.unwrap().0;
            let val_pieces: Vec<u8> = val["info"]["pieces"]
                .as_array()
                .unwrap()
                .iter()
                .map(|b| b.as_i64().unwrap() as u8)
                .collect();

            let torrent_res: Result<Torrent, _> = serde_json::from_value(val);

            assert!(torrent_res.is_ok());

            let torrent: Torrent = torrent_res.expect("torrent_res is already ok'd");

            assert_eq!(
                torrent.announce,
                "http://bittorrent-test-tracker.codecrafters.io/announce".to_string()
            );
            assert_eq!(torrent.info.length, 92063);
            assert_eq!(torrent.info.name, "sample.txt".to_string());
            assert_eq!(torrent.info.piece_length, 32768);
            assert!(!torrent.info.pieces.is_empty());
            assert_eq!(torrent.info.pieces, val_pieces);
        }

        #[test]
        fn generated_torrent_info_hash_match_to_serde_bencode() {
            let encoded_value = data();

            let (decoded_val, _) = decode_bencoded_value(&encoded_value)
                .expect("failed to decode the encoded value file");

            let torrent: Torrent =
                serde_json::from_value(decoded_val.clone()).expect("failed to parse to `Torrent`");

            let serde_torrent: Torrent = serde_bencode::from_bytes(&encoded_value)
                .expect("failed to parse bytes to torrent");

            let bencoded_info = serde_bencode::to_bytes(&torrent.info)
                .expect("Failed to bencode `Torrent` `info` dictionary");

            let mut hasher = sha1::Sha1::new();
            hasher.update(&bencoded_info);
            let hash = hasher.finalize();

            let hex_hash = hex::encode(hash);

            assert_eq!(
                hex_hash, "d69f91e6b2ae4c542468d1073a71d4ea13879a7f",
                "Info hash does not match expected value"
            );
            assert_eq!(torrent, serde_torrent);
        }
    }
}
