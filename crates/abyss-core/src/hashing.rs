use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub type Hash256 = [u8; 32];

pub const ZERO_HASH: Hash256 = [0_u8; 32];

pub fn dev_hash<T: Hash>(value: &T) -> Hash256 {
    let mut out = [0_u8; 32];

    for domain in 0_u64..4 {
        let mut hasher = DefaultHasher::new();
        domain.hash(&mut hasher);
        value.hash(&mut hasher);
        let chunk = hasher.finish().to_le_bytes();
        out[(domain as usize) * 8..(domain as usize + 1) * 8].copy_from_slice(&chunk);
    }

    out
}

pub fn hex(hash: &Hash256) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for byte in hash {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}
