use rand::{thread_rng, Rng};

pub fn random_hex(byte_len: usize) -> String {
    let bytes: Vec<u8> = (0..byte_len).map(|_| thread_rng().gen::<u8>()).collect();
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

pub fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

pub fn to_alphanumeric_snake(s: &str) -> String {
    let mut out = String::new();
    for char in s.chars() {
        if char.is_alphanumeric() {
            out += &char.to_ascii_lowercase().to_string();
        } else {
            if out.chars().last().unwrap() != '-' {
                out += "-";
            }
        }
    }

    if out.chars().last().unwrap() == '-' {
        out.pop();
    }

    out
}
