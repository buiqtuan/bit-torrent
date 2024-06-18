use serde::Deserialize;
use serde_json;
use std::env;
use std::fmt;
use serde::de::{self, Visitor};

#[derive(Debug)]
struct Torrent {
    annouce: reqwest::Url,
    info: Info
}

#[derive(Debug, Clone, Deserialize)]
struct Info {
    #[serde(rename = "piece length")]
    plength: usize,
    name: String,
    piece: Vec<u8>,
    #[serde(flatten)]
    key: Keys,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum Keys {
    SingleFile { length: usize },
    MultiFile { files: File },
}

#[derive(Debug, Clone, Deserialize)]
struct File {
    length: usize,
    path: Vec<String>,
}

struct HashStrVisitor;

impl<'de> Visitor<'de> for I32Visitor {
    type Value = Vec<[u8;20]>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte string whose lenght is multiple of 20")
    }

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(i32::from(value))
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        use std::i32;
        if value >= i64::from(i32::MIN) && value <= i64::from(i32::MAX) {
            Ok(value as i32)
        } else {
            Err(E::custom(format!("i32 out of range: {}", value)))
        }
    }

    // Similar for other methods:
    //   - visit_i16
    //   - visit_u8
    //   - visit_u16
    //   - visit_u32
    //   - visit_u64
}

fn decode_bencoded_value(encode_value: &str) -> (serde_json::Value, &str) {
    const CLASSIFICATION_CHAR: usize = 1;

    match encode_value.chars().next() {
        Some('i') => {
            if let Some((n, rest)) = encode_value
                .split_at(CLASSIFICATION_CHAR)
                .1 // skip the first char
                .split_once('e')
                .and_then(|(digits, rest)| {
                    let n = digits.parse::<i64>().ok()?;
                    Some((n, rest))
                })
            {
                return (n.into(), rest);
            }
        }
        Some('d') => {
            let mut values = serde_json::Map::new();
            let mut rest = encode_value.split_at(CLASSIFICATION_CHAR).1;
            while !rest.is_empty() && !rest.starts_with('e') {
                let (k, remainder) = decode_bencoded_value(rest);
                let k = match k {
                    serde_json::Value::String(k) => k,
                    k => {
                        panic!("K is not a string {k:?}");
                    }
                };
                let (v, remainder) = decode_bencoded_value(remainder);

                values.insert(k, v);
                rest = remainder;
            }

            return (values.into(), &rest[1..]);
        }
        Some('l') => {
            let mut values = Vec::new();
            let mut rest = encode_value.split_at(CLASSIFICATION_CHAR).1; // skip the first char
            while !rest.is_empty() && !rest.starts_with('e') {
                let (v, remainder) = decode_bencoded_value(rest);
                values.push(v);
                rest = remainder;
            }
            return (values.into(), &rest[1..]);
        }
        Some('0'..='9') => {
            if let Some((len, rest)) = encode_value.split_once(':') {
                if let Ok(len) = len.parse::<usize>() {
                    return (rest[..len].to_string().into(), &rest[len..]);
                }
            }
        }
        _ => {}
    }

    panic!("Unhandled encoded value {}", encode_value);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];

        let decoded_value = decode_bencoded_value(encoded_value);

        println!("Decode from {} is {:?}", encoded_value, decoded_value);
    } else {
        eprintln!("Wrong command!")
    }
}
