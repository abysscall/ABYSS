//! abyss-crypto-api — lightweight trait-based abstraction for crypto implementations
//!
//! Define minimal traits so the repository can switch between a dev-only
//! implementation and a production audited implementation without changing
//! higher-level code.

/// A minimal Keypair abstraction.
pub trait Keypair {
    /// Public key type (bytes or structured type)
    type Public;
    /// Secret key type (kept private)
    type Secret;

    /// Return public key reference
    fn public(&self) -> &Self::Public;

    /// Return secret key reference (implementations should keep this private).
    fn secret(&self) -> &Self::Secret;
}

/// Signer abstraction: sign a message and produce a signature blob.
pub trait Signer {
    /// Signature output type
    type Signature;

    /// Sign a message
    fn sign(&self, msg: &[u8]) -> Self::Signature;
}

/// Verifier abstraction: verify that a signature is valid for a message.
pub trait Verifier {
    /// Public key type used for verification
    type Public;
    /// Signature type to verify
    type Signature;

    /// Verify signature for message using public key. Return true if valid.
    fn verify(pubkey: &Self::Public, msg: &[u8], sig: &Self::Signature) -> bool;
}

// Convenience re-exports and adapters can be implemented in consumer crates.
