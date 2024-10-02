use serde_json;
use std::env;
// use serde_bencode

fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    match encoded_value.chars().next() {
        Some('0'..='9') => decode_bencoded_str(encoded_value),
        Some('i') => decode_bencoded_int(encoded_value),
        Some(c) => panic!("Unhandled bencoded value: {}", c),
        None => serde_json::Value::Null,
    }
}

fn decode_bencoded_str(encoded_value: &str) -> serde_json::Value {
    let (len_str, remainder) = encoded_value
        .split_once(':')
        .expect("length `:` delimiter for bencoded string");

    let len = len_str
        .parse::<usize>()
        .expect("valid length integer value for bencoded string");
    let string = remainder[..len].to_string();

    serde_json::Value::String(string)
}

fn decode_bencoded_int(encoded_value: &str) -> serde_json::Value {
    let (num_str, _) = encoded_value[1..]
        .split_once('e')
        .expect("integer ending `e` delimiter for bencoded integer");
    let num = num_str
        .parse::<i64>()
        .expect("valid `i64` value for bencoded integer");

    serde_json::Value::Number(serde_json::Number::from(num))
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
