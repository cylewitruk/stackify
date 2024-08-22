#[derive(Clone, Default, PartialOrd, Ord, PartialEq, Eq, Hash)]
/// A Bitcoin script
pub struct Script(pub Box<[u8]>);

/// Helper to encode an integer in script format
pub fn build_scriptint(n: i64) -> Vec<u8> {
    if n == 0 {
        return vec![];
    }

    let neg = n < 0;

    let mut abs = n.unsigned_abs() as usize;
    let mut v = Vec::with_capacity(size_of::<usize>() + 1);
    while abs > 0xFF {
        v.push((abs & 0xFF) as u8);
        abs >>= 8;
    }
    // If the number's value causes the sign bit to be set, we need an extra
    // byte to get the correct value and correct sign bit
    if abs & 0x80 != 0 {
        v.push(abs as u8);
        v.push(if neg { 0x80u8 } else { 0u8 });
    }
    // Otherwise we just set the sign bit ourselves
    else {
        abs |= if neg { 0x80 } else { 0 };
        v.push(abs as u8);
    }
    v
}
