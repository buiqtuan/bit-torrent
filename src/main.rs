use serde::Deserialize;
use serde::Deserializer;
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
    piece: Hashes,
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

#[derive(Debug, Clone)]
struct Hashes(Vec<[u8; 20]>);

struct HashesVisitor;

impl<'de> Visitor<'de> for HashesVisitor {
    type Value = Hashes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte string whose length is multiple of 20")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error, {

        if v.len() % 20 != 0 {
            return Err(E::custom(format!("length is {}", v.len())));
        }

        // use array_chunks when stable
        return Ok(Hashes(
            v.chunks_exact(20)
            .map(|slice_20| {
                slice_20.try_into().expect("The lenght of this slice should be 20")
            })
            .collect()
        ));
    }

    // Similar for other methods:
    //   - visit_i16
    //   - visit_u8
    //   - visit_u16
    //   - visit_u32
    //   - visit_u64
}

impl<'de> Deserialize<'de> for Hashes {
    fn deserialize<D>(deserializer: D) -> Result<Hashes, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(HashesVisitor)
    }
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
