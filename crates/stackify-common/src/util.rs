use rand::{thread_rng, Rng};

pub fn random_hex(byte_len: usize) -> String {
    let bytes: Vec<u8> = (0..byte_len)
        .map(|_| thread_rng().gen::<u8>())
        .collect();
    bytes.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}