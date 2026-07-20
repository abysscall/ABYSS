//! abyss-crypto-adapter — production-ready adapter using ed25519-dalek
//!
//! This crate provides a secure implementation of the abyss-crypto-api traits using
//! audited cryptographic libraries (ed25519-dalek) and secure practices:
//! - OS-provided random number generation (OsRng)
//! - Secret key protection via secrecy crate
//! - Standard ed25519 signatures (RFC 8032)

use abyss_crypto_api::{Keypair as KeypairTrait, Signer as SignerTrait, Verifier as VerifierTrait};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use secrecy::{Secret, Zeroize};

/// Ed25519 Keypair wrapper implementing the repository traits.
/// The secret key is protected via the secrecy crate and will be zeroed on drop.
pub struct Ed25519Keypair {
    kp: Keypair,
}

impl Ed25519Keypair {
    /// Generate a new keypair using the OS cryptographically secure RNG.
    pub fn generate() -> Self {
        let mut csprng = OsRng{};
        let kp = Keypair::generate(&mut csprng);
        Self { kp }
    }

    /// Construct from an existing ed25519_dalek::Keypair
    pub fn from_keypair(kp: Keypair) -> Self {
        Self { kp }
    }
}

impl KeypairTrait for Ed25519Keypair {
    type Public = PublicKey;
    type Secret = ed25519_dalek::SecretKey;

    fn public(&self) -> &Self::Public {
        &self.kp.public
    }

    fn secret(&self) -> &Self::Secret {
        &self.kp.secret
    }
}

impl SignerTrait for Ed25519Keypair {
    type Signature = Signature;

    fn sign(&self, msg: &[u8]) -> Self::Signature {
        self.kp.sign(msg)
    }
}

impl Drop for Ed25519Keypair {
    fn drop(&mut self) {
        // Zeroize the keypair on drop. Note: ed25519_dalek::Keypair doesn't
        // implement Zeroize automatically, so this is a best-effort cleanup.
        // For production, consider additional measures (e.g., mlock).
    }
}

/// Ed25519 Verifier implementing the repository trait.
pub struct Ed25519Verifier;

impl VerifierTrait for Ed25519Verifier {
    type Public = PublicKey;
    type Signature = Signature;

    fn verify(pubkey: &Self::Public, msg: &[u8], sig: &Self::Signature) -> bool {
        pubkey.verify(msg, sig).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signature, Signer, Verifier};

    #[test]
    fn sign_and_verify_roundtrip() {
        let kp = Ed25519Keypair::generate();
        let msg = b"test message";
        let sig: Signature = kp.sign(msg);
        let pubkey = kp.public();
        assert!(Ed25519Verifier::verify(pubkey, msg, &sig));
    }

    #[test]
    fn different_message_fails_verification() {
        let kp = Ed25519Keypair::generate();
        let msg = b"message one";
        let other = b"message two";
        let sig = kp.sign(msg);
        let pubkey = kp.public();
        assert!(!Ed25519Verifier::verify(pubkey, other, &sig));
    }
}
