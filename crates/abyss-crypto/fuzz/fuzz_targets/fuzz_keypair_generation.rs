#![no_main]

use libfuzzer_sys::fuzz_target;
use abyss_crypto::primitives::keys::Keypair;

fuzz_target!(|data: &[u8]| {
    // Fuzz keypair deserialization and import to catch edge cases
    if data.len() >= 32 {
        let _ = Keypair::from_secret_bytes(&data[..32]);
    }
});
