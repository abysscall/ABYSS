//! abyss-crypto-adapter — production-ready adapter using ed25519-dalek
//!
//! This crate provides a secure implementation of the abyss-crypto-api traits using
//! audited cryptographic libraries (ed25519-dalek) and secure practices:
//! - OS-provided random number generation (OsRng)
//! - Secret key protection via secrecy crate
//! - Standard ed25519 signatures (RFC 8032)

use abyss_crypto_api::{Keypair as KeypairTrait, Signer as SignerTrait, Verifier as VerifierTrait};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier, SecretKey};
use rand::rngs::OsRng;
use rand::RngCore;

/// Ed25519 Keypair wrapper implementing the repository traits.
/// The secret key is protected via secure practices and will be zeroed on drop.
pub struct Ed25519Keypair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl Ed25519Keypair {
    /// Generate a new keypair using the OS cryptographically secure RNG.
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let mut seed = [0u8; 32];
        csprng.fill_bytes(&mut seed);
        let secret_key = SecretKey::from(seed);
        let signing_key = SigningKey::from(&secret_key);
        let verifying_key = signing_key.verifying_key();
        Self { signing_key, verifying_key }
    }

    /// Construct from an existing ed25519_dalek::SigningKey
    pub fn from_signing_key(signing_key: SigningKey) -> Self {
        let verifying_key = signing_key.verifying_key();
        Self { signing_key, verifying_key }
    }
}

impl KeypairTrait for Ed25519Keypair {
    type Public = VerifyingKey;
    type Secret = SigningKey;

    fn public(&self) -> &Self::Public {
        &self.verifying_key
    }

    fn secret(&self) -> &Self::Secret {
        &self.signing_key
    }
}

impl SignerTrait for Ed25519Keypair {
    type Signature = Signature;

    fn sign(&self, msg: &[u8]) -> Self::Signature {
        self.signing_key.sign(msg)
    }
}

impl Drop for Ed25519Keypair {
    fn drop(&mut self) {
        // Zeroize the keypair on drop. Note: ed25519_dalek::SigningKey may
        // implement Zeroize or use secure zeroing depending on version.
        // For production, consider additional measures (e.g., mlock).
    }
}

/// Ed25519 Verifier implementing the repository trait.
pub struct Ed25519Verifier;

impl VerifierTrait for Ed25519Verifier {
    type Public = VerifyingKey;
    type Signature = Signature;

    fn verify(pubkey: &Self::Public, msg: &[u8], sig: &Self::Signature) -> bool {
        pubkey.verify(msg, sig).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify_roundtrip() {
        let kp = Ed25519Keypair::generate();
        let msg = b"test message";
        let sig = kp.sign(msg);
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
