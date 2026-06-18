//! Development cryptographic identities for ABYSS.
//!
//! This crate is deliberately not production cryptography yet. It gives the
//! devnet deterministic keys and account identifiers while the protocol shape
//! is being built. Production signing, mnemonic handling, shielded keys, and
//! audited primitives belong in this crate later.

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use abyss_core::hashing::{dev_hash, hex, Hash256};
use abyss_core::Address;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretKey(Hash256);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PublicKey(Hash256);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DevKeypair {
    secret: SecretKey,
    public: PublicKey,
}

impl DevKeypair {
    pub fn from_seed(seed: impl AsRef<[u8]>) -> Self {
        let seed = seed.as_ref();
        let secret = SecretKey(dev_hash(&("abyss:dev-secret:v0", seed)));
        let public = PublicKey(dev_hash(&("abyss:dev-public:v0", secret.0)));
        Self { secret, public }
    }

    pub fn generate(label: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let pid = std::process::id();
        Self::from_seed(format!("abyss:{label}:{pid}:{now}"))
    }

    pub fn public(&self) -> &PublicKey {
        &self.public
    }

    pub fn secret(&self) -> &SecretKey {
        &self.secret
    }

    pub fn dev_address(&self) -> Address {
        self.public.dev_address()
    }
}

impl PublicKey {
    pub fn bytes(&self) -> Hash256 {
        self.0
    }

    pub fn fingerprint(&self) -> String {
        hex(&self.0)[..16].to_string()
    }

    pub fn dev_address(&self) -> Address {
        Address::new(format!("abyss:dev:{}", self.fingerprint()))
            .expect("dev address is generated from hex")
    }
}

impl SecretKey {
    pub fn expose_dev_hex(&self) -> String {
        hex(&self.0)
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex(&self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_produces_same_identity() {
        let first = DevKeypair::from_seed("alice");
        let second = DevKeypair::from_seed("alice");

        assert_eq!(first, second);
        assert_eq!(first.dev_address(), second.dev_address());
    }

    #[test]
    fn different_seeds_produce_different_addresses() {
        let alice = DevKeypair::from_seed("alice").dev_address();
        let bob = DevKeypair::from_seed("bob").dev_address();

        assert_ne!(alice, bob);
    }
}

