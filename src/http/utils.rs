use std::collections::HashMap;

pub fn decode_component(s: &str) -> String {
    let mut bytes: Vec<u8> = Vec::with_capacity(s.len());
    let mut iter = s.bytes().peekable();

    while let Some(b) = iter.next() {
        if b == b'%' {
            let h1 = iter.next();
            let h2 = iter.next();
            match (h1, h2) {
                (Some(c1), Some(c2)) => {
                    let d1 = (c1 as char).to_digit(16);
                    let d2 = (c2 as char).to_digit(16);
                    match (d1, d2) {
                        (Some(d1), Some(d2)) => bytes.push((d1 * 16 + d2) as u8),
                        _ => {
                            bytes.push(b'%');
                            bytes.push(c1);
                            bytes.push(c2);
                        }
                    }
                }
                (Some(c1), None) => {
                    bytes.push(b'%');
                    bytes.push(c1);
                }
                _ => bytes.push(b'%'),
            }
        } else if b == b'+' {
            bytes.push(b' ');
        } else {
            bytes.push(b);
        }
    }

    String::from_utf8_lossy(&bytes).into_owned()
}

pub fn decode_query(query_string: &str) -> HashMap<String, String> {
    query_string
        .split('&')
        .filter(|s| !s.is_empty())
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().map(decode_component)?;
            let value = parts.next().map(decode_component).unwrap_or_default();
            Some((key, value))
        })
        .collect()
}
