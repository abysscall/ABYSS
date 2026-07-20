//! abyss-crypto-adapter — production-ready adapter using ed25519-dalek

use abyss_crypto_api::{Keypair as KeypairTrait, Signer as SignerTrait, Verifier as VerifierTrait};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;

/// Ed25519 Keypair wrapper implementing the repository traits.
pub struct Ed25519Keypair {
    kp: Keypair,
}

impl Ed25519Keypair {
    /// Generate a new keypair using the OS RNG.
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

pub struct Ed25519Verifier;

impl VerifierTrait for Ed25519Verifier {
    type Public = PublicKey;
    type Signature = Signature;

    fn verify(pubkey: &Self::Public, msg: &[u8], sig: &Self::Signature) -> bool {
        pubkey.verify(msg, sig).is_ok()
    }
}
