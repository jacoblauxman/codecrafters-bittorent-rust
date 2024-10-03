#[derive(thiserror::Error, Debug, PartialEq)]
pub enum BencodeError {
    #[error("Data type error: unknown or unhandled bencode data type `{0}`")]
    UnknownValue(u8),
    #[error("Data format error: {0}")]
    DataFormat(String),
    #[error("Unexpected end of bencoded input data")]
    UnexpectedEnd,
}

pub fn decode_bencoded_value(
    encoded_value: &[u8],
) -> Result<(serde_json::Value, &[u8]), BencodeError> {
    match encoded_value.first() {
        Some(b'0'..=b'9') => decode_bencoded_str(encoded_value),
        Some(b'i') => decode_bencoded_int(encoded_value),
        Some(b'l') => decode_bencoded_list(encoded_value),
        Some(b'd') => decode_bencoded_dict(encoded_value),
        Some(&c) => Err(BencodeError::UnknownValue(c)),
        None => Err(BencodeError::UnexpectedEnd),
    }
}

fn decode_bencoded_str(encoded_value: &[u8]) -> Result<(serde_json::Value, &[u8]), BencodeError> {
    let delim = encoded_value
        .iter()
        .position(|&c| c == b':')
        .ok_or_else(|| {
            BencodeError::DataFormat(
                "missing length `:` delimiter for bencoded string value.".to_string(),
            )
        })?;

    let len = std::str::from_utf8(&encoded_value[..delim])
        .map_err(|_| {
            BencodeError::DataFormat(
                "invalid length value provided for bencoded string value.".to_string(),
            )
        })?
        .parse::<usize>()
        .map_err(|_| {
            BencodeError::DataFormat(
                "invalid length value provided for bencoded string value.".to_string(),
            )
        })?;

    let remainder = &encoded_value[delim + 1..];
    if remainder.len() < len {
        return Err(BencodeError::DataFormat(format!(
            "provided string value's length `{}` exceeds remaining input length `{}`.",
            len,
            remainder.len()
        )));
    }

    let value = std::str::from_utf8(&remainder[..len])
        .map(|s| serde_json::Value::String(s.to_string()))
        .unwrap_or_else(|_| {
            serde_json::Value::Array(
                remainder[..len]
                    .iter()
                    .map(|&b| serde_json::Value::from(b))
                    .collect(),
            )
        });

    Ok((value, &remainder[len..]))
}

fn decode_bencoded_int(encoded_value: &[u8]) -> Result<(serde_json::Value, &[u8]), BencodeError> {
    let delim = encoded_value
        .iter()
        .position(|&c| c == b'e')
        .ok_or_else(|| {
            BencodeError::DataFormat(
                "missing ending `e` delimiter for bencoded integer value.".to_string(),
            )
        })?;

    let num_str = std::str::from_utf8(&encoded_value[1..delim]).map_err(|_| {
        BencodeError::DataFormat("Invalid UTF-8 sequence in bencoded integer value.".to_string())
    })?;

    if num_str == "-0" {
        return Err(BencodeError::DataFormat("invalid bencoded value `-0` found when parsing to integer value, expects valid `i64` value.".to_string()));
    }

    let num = num_str.parse::<i64>().map_err(|_| {
        BencodeError::DataFormat(format!(
            "expected valid `i64` value when parsing bencoded data to integer value, received `{}`.",
            num_str
        ))
    })?;

    Ok((
        serde_json::Value::Number(num.into()),
        &encoded_value[delim + 1..],
    ))
}

fn decode_bencoded_list(encoded_value: &[u8]) -> Result<(serde_json::Value, &[u8]), BencodeError> {
    let mut values = Vec::new();
    let mut remainder = &encoded_value[1..];

    while !remainder.is_empty() && remainder.first() != Some(&b'e') {
        let (value, rest) = decode_bencoded_value(remainder)?;
        values.push(value);
        remainder = rest;
    }

    if remainder.is_empty() {
        return Err(BencodeError::DataFormat(
            "missing ending `e` delimiter for bencoded list.".to_string(),
        ));
    }

    Ok((serde_json::Value::Array(values), &remainder[1..]))
}

fn decode_bencoded_dict(encoded_value: &[u8]) -> Result<(serde_json::Value, &[u8]), BencodeError> {
    // dictionary keys must be strings and appear in sorted order (serde_json::Map uses BTreeMap)
    let mut map = serde_json::Map::new();
    let mut remainder = &encoded_value[1..];

    while !remainder.is_empty() && remainder.first() != Some(&b'e') {
        let (key, rest) = decode_bencoded_value(remainder)?;
        let key = match key {
            serde_json::Value::String(k) => k,
            _ => {
                return Err(BencodeError::DataFormat(format!(
                "bencoded dictionary must contain valid `string` data type for `key` value, received `{}`.",
                key
            )));
            }
        };

        let (val, rest) = decode_bencoded_value(rest)?;
        map.insert(key, val);
        remainder = rest;
    }

    if remainder.is_empty() {
        return Err(BencodeError::DataFormat(
            "missing ending `e` delimiter for bencoded dictionary.".to_string(),
        ));
    }

    Ok((serde_json::Value::Object(map), &remainder[1..]))
}

#[cfg(test)]
mod test {
    use super::*;

    mod decode_bencoded_str {
        use super::*;

        #[test]
        fn valid_bencoded_string() {
            let encoded_value = &"7:testing".as_bytes();

            let res = decode_bencoded_str(encoded_value);

            assert!(res.is_ok());
            assert_eq!(
                res.unwrap().0,
                serde_json::Value::String("testing".to_string())
            );
        }

        #[test]
        fn invalid_no_length_delimiter() {
            let encoded_value = &"7testing".as_bytes();

            let res = decode_bencoded_str(encoded_value);

            assert!(res.is_err());
            assert_eq!(
                res,
                Err(BencodeError::DataFormat(
                    "missing length `:` delimiter for bencoded string value.".to_string()
                ))
            )
        }

        #[test]
        fn invalid_length_value() {
            let encoded_value = &"7a:testing".as_bytes();

            let res = decode_bencoded_str(encoded_value);

            assert!(res.is_err());
            assert_eq!(
                res,
                Err(BencodeError::DataFormat(format!(
                    "invalid length value provided for bencoded string value."
                )))
            )
        }

        #[test]
        fn invalid_length_greater_than_input() {
            let encoded_value = &"7:test".as_bytes();

            let res = decode_bencoded_str(encoded_value);

            assert!(res.is_err());
            assert_eq!(
                res,
                Err(BencodeError::DataFormat(format!(
                    "provided string value's length `{}` exceeds remaining input length `{}`.",
                    7, 4
                )))
            );
        }
    }

    mod decode_bencoded_int {
        use super::*;
        #[test]
        fn valid_bencoded_int() {
            let encoded_value = &"i-53e".as_bytes();

            let res = decode_bencoded_int(encoded_value);

            assert!(res.is_ok());
            assert_eq!(
                res.unwrap().0,
                serde_json::Value::Number(serde_json::Number::from(-53))
            );
        }

        #[test]
        fn invalid_bencoded_int_value() {
            let encoded_value = &"i-53ae".as_bytes();

            let res = decode_bencoded_int(encoded_value);

            assert!(res.is_err());
            assert_eq!(res, Err(BencodeError::DataFormat(
                format!("expected valid `i64` value when parsing bencoded data to integer value, received `-53a`.")
            )));
        }

        #[test]
        fn invalid_int_no_delimiter() {
            let encoded_value = &"i-53".as_bytes();

            let res = decode_bencoded_int(encoded_value);

            assert!(res.is_err());
            assert_eq!(
                res,
                Err(BencodeError::DataFormat(
                    "missing ending `e` delimiter for bencoded integer value.".to_string(),
                ))
            );
        }
    }

    mod decode_bencoded_list {
        use super::*;

        #[test]
        fn valid_with_string_and_int_values() {
            let encoded_value = &"l7:testingi-53e4:teste".as_bytes();

            let res = decode_bencoded_list(encoded_value);

            assert!(res.is_ok());
            assert_eq!(
                res.unwrap().0,
                serde_json::Value::Array(vec![
                    serde_json::Value::String("testing".to_string()),
                    serde_json::Value::Number((-53).into()),
                    serde_json::Value::String("test".to_string())
                ])
            );
        }

        #[test]
        fn valid_with_nested_list() {
            let encoded_value = &"ll7:testingi-53ee4:teste".as_bytes();

            let res = decode_bencoded_list(encoded_value);

            assert!(res.is_ok());
            assert_eq!(
                res.unwrap().0,
                serde_json::Value::Array(vec![
                    serde_json::Value::Array(vec![
                        serde_json::Value::String("testing".to_string()),
                        serde_json::Value::Number((-53).into()),
                    ]),
                    serde_json::Value::String("test".to_string())
                ])
            );
        }

        #[test]
        fn invalid_missing_delimiter() {
            let encoded_value = &"l7:testingi-53e4:test".as_bytes();

            let res = decode_bencoded_list(encoded_value);

            assert!(res.is_err());
            assert_eq!(
                res,
                Err(BencodeError::DataFormat(
                    "missing ending `e` delimiter for bencoded list.".to_string(),
                ))
            );
        }
    }

    mod decode_bencoded_dict {
        use super::*;

        #[test]
        fn valid_key_values() {
            let encoded_value = &"d7:testingi-53e4:test7:testing9:list-testlee".as_bytes();
            let mut expected = serde_json::Map::new();
            expected.insert(
                "testing".to_string(),
                serde_json::Value::Number((-53).into()),
            );
            expected.insert(
                "test".to_string(),
                serde_json::Value::String("testing".to_string()),
            );
            expected.insert("list-test".to_string(), serde_json::Value::Array(vec![]));

            let res = decode_bencoded_dict(encoded_value);

            assert!(res.is_ok());
            assert_eq!(res.unwrap().0, serde_json::Value::Object(expected));
        }
    }

    #[test]
    fn valid_with_nested_obj() {
        let encoded_value = &"d7:testingd4:testi-53eee".as_bytes();
        let mut expected = serde_json::Map::new();
        let mut expected_inner = serde_json::Map::new();
        expected_inner.insert("test".to_string(), serde_json::Value::Number((-53).into()));
        expected.insert(
            "testing".to_string(),
            serde_json::Value::Object(expected_inner),
        );

        let res = decode_bencoded_dict(encoded_value);

        assert!(res.is_ok());
        assert_eq!(res.unwrap().0, serde_json::Value::Object(expected));
    }

    #[test]
    fn invalid_missing_delimiter() {
        let encoded_value = &"d7:testingi-53e".as_bytes();

        let res = decode_bencoded_dict(encoded_value);

        assert!(res.is_err());
        assert_eq!(
            res,
            Err(BencodeError::DataFormat(
                "missing ending `e` delimiter for bencoded dictionary.".to_string(),
            ))
        );
    }
}
