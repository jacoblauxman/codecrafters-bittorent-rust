pub mod commands;

#[derive(thiserror::Error, Debug)]
pub enum BencodeError {
    #[error("Data type error: unknown or unhandled bencode data type `{0}`")]
    UnknownValue(char),
    #[error("Data format error: {0}")]
    DataFormat(String),
    // #[error("Unexpected end of bencoded input data")]
    // UnexpectedEnd,
}

pub fn decode_bencoded_value(
    encoded_value: &str,
) -> Result<(serde_json::Value, &str), BencodeError> {
    match encoded_value.chars().next() {
        Some('0'..='9') => decode_bencoded_str(encoded_value),
        Some('i') => decode_bencoded_int(encoded_value),
        Some('l') => decode_bencoded_list(encoded_value),
        Some(c) => Err(BencodeError::UnknownValue(c)),
        None => Ok((serde_json::Value::Null, &"")),
    }
}

fn decode_bencoded_str(encoded_value: &str) -> Result<(serde_json::Value, &str), BencodeError> {
    let (len_str, remainder) = encoded_value.split_once(':').ok_or_else(|| {
        BencodeError::DataFormat(
            "missing length `:` delimiter for bencoded string value".to_string(),
        )
    })?;

    let len = len_str.parse::<usize>().map_err(|_| {
        BencodeError::DataFormat(format!(
            "invalid length value `{len_str}` provided for bencoded string value."
        ))
    })?;

    if remainder.len() < len {
        return Err(BencodeError::DataFormat(format!(
            "provided string value's length `{}` exceeds remaining input length `{}`",
            len,
            remainder.len()
        )));
    }

    let string = remainder[..len].to_string();

    Ok((serde_json::Value::String(string), &remainder[len..]))
}

fn decode_bencoded_int(encoded_value: &str) -> Result<(serde_json::Value, &str), BencodeError> {
    let (num_str, remainder) = encoded_value[1..].split_once('e').ok_or_else(|| {
        BencodeError::DataFormat(
            "missing ending `e` delimiter for bencoded integer value".to_string(),
        )
    })?;

    let num = num_str
        .parse::<i64>()
        .map_err(|_| {
            BencodeError::DataFormat(
                format!("expected valid `i64` value when parsing bencoded data to integer value, received `{num_str}`")
            )
        })?;

    Ok((
        serde_json::Value::Number(serde_json::Number::from(num)),
        remainder,
    ))
}

fn decode_bencoded_list(encoded_value: &str) -> Result<(serde_json::Value, &str), BencodeError> {
    let mut values = Vec::new();
    let mut remainder = &encoded_value[1..];

    while !remainder.is_empty() && remainder.chars().next() != Some('e') {
        let (value, rest) = decode_bencoded_value(remainder)?;
        values.push(value);
        remainder = rest;
    }

    if remainder.is_empty() {
        return Err(BencodeError::DataFormat(
            "missing `e` delimiter for ending of bencoded list".to_string(),
        ));
    }

    Ok((serde_json::Value::Array(values), &remainder[1..])) // consume (skip) `e` delimiter
}
