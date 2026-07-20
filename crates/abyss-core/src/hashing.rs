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

/// Domain tag mixed into leaf hashes so a leaf can never be reinterpreted as an
/// internal node (second-preimage protection).
const MERKLE_LEAF_DOMAIN: &str = "abyss:merkle:leaf:v0";

/// Domain tag mixed into internal node hashes.
const MERKLE_NODE_DOMAIN: &str = "abyss:merkle:node:v0";

/// Computes the binary Merkle root over an ordered list of leaf hashes.
///
/// Leaves are first re-hashed under a leaf domain tag, then combined pairwise
/// under a node domain tag. When a level has an odd number of nodes the last
/// node is duplicated (the Bitcoin convention). An empty list yields
/// [`ZERO_HASH`], matching the transactions root of an empty block.
pub fn merkle_root(leaves: &[Hash256]) -> Hash256 {
    if leaves.is_empty() {
        return ZERO_HASH;
    }

    let mut level: Vec<Hash256> = leaves
        .iter()
        .map(|leaf| dev_hash(&(MERKLE_LEAF_DOMAIN, leaf)))
        .collect();

    while level.len() > 1 {
        let mut next = Vec::with_capacity((level.len() + 1) / 2);
        for pair in level.chunks(2) {
            let left = pair[0];
            let right = if pair.len() == 2 { pair[1] } else { pair[0] };
            next.push(dev_hash(&(MERKLE_NODE_DOMAIN, left, right)));
        }
        level = next;
    }

    level[0]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(n: u8) -> Hash256 {
        let mut h = ZERO_HASH;
        h[0] = n;
        h
    }

    #[test]
    fn empty_tree_is_zero_hash() {
        assert_eq!(merkle_root(&[]), ZERO_HASH);
    }

    #[test]
    fn single_leaf_root_is_domain_separated_leaf() {
        // A single-leaf tree is just that leaf hashed under the leaf domain,
        // and it must differ from the raw leaf value.
        let root = merkle_root(&[leaf(1)]);
        assert_eq!(root, dev_hash(&(MERKLE_LEAF_DOMAIN, &leaf(1))));
        assert_ne!(root, leaf(1));
    }

    #[test]
    fn root_is_order_sensitive() {
        let a = merkle_root(&[leaf(1), leaf(2)]);
        let b = merkle_root(&[leaf(2), leaf(1)]);
        assert_ne!(a, b);
    }

    #[test]
    fn root_is_deterministic() {
        let leaves = [leaf(1), leaf(2), leaf(3)];
        assert_eq!(merkle_root(&leaves), merkle_root(&leaves));
    }

    #[test]
    fn odd_level_duplicates_last_node() {
        // Three leaves: the lone right node is duplicated at the first level, so
        // the root equals hashing the same three leaves with the third repeated.
        let three = merkle_root(&[leaf(1), leaf(2), leaf(3)]);
        let four = merkle_root(&[leaf(1), leaf(2), leaf(3), leaf(3)]);
        assert_eq!(three, four);
    }

    #[test]
    fn different_leaf_counts_differ() {
        assert_ne!(merkle_root(&[leaf(1)]), merkle_root(&[leaf(1), leaf(2)]));
    }
}

