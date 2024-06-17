use serde_json;
use std::env;

fn decode_bencoded_value(encode_value: &str) -> (serde_json::Value, &str) {
    
    match encode_value.chars().next() {
        Some('i') => {
            if let Some((n, rest)) = encode_value
                .split_at(1)
                .1
                .split_once('e')
                .and_then(|(digits, rest)| {
                    let n = digits.parse::<i64>().ok()?;
                    Some((n, rest))
                })
            {
                return (n.into(), rest);
            }
        },
        Some('l') => {
            let mut values = Vec::new();
            let mut rest = encode_value.split_at(1).1;
            while !rest.is_empty() && !rest.starts_with('a') {
                let (v, remainder) = decode_bencoded_value(rest);
                values.push(v);
                rest = remainder;
            }
            return (values.into(), &rest[1..])
        },
        Some('0'..='9') => {
            if let Some((len, rest)) = encode_value.split_once(':') {
                if let Ok(len) = len.parse::<usize>() {
                    return (rest[..len].to_string().into(), &rest[len..])
                }
            }
        },
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
