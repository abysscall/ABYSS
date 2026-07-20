#![no_main]

use libfuzzer_sys::fuzz_target;
use abyss_crypto::primitives::signature::Signature;

fuzz_target!(|data: &[u8]| {
    // Fuzz signature deserialization to catch parsing/validation bugs
    let _ = Signature::from_bytes(data);
});
