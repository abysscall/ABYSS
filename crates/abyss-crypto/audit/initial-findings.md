Initial audit findings for crates/abyss-crypto

Summary
- This crate implements development-only cryptographic identities (DevKeypair) using a local dev_hash function. It is explicitly marked as non-production.

Findings
1. Deterministic dev keys
   - DevKeypair::from_seed and DevKeypair::generate derive keys deterministically from seed or process/time. Not secure for production.
   - Risk: predictable keys, key reuse, exposure via seed/state.

2. Exposed secret hex
   - SecretKey::expose_dev_hex returns raw hex of the secret. This eases debugging but risks leaking secrets if used in production code or logs.

3. RNG usage
   - No use of OS-provided secure RNG (OsRng/getrandom) detected. Dev generation uses process/time-based entropy (not cryptographic).

4. Documentation
   - The crate includes a note that it is dev-only, but no migration or adapter guidance is provided for replacing with production primitives.

Recommendations
- Keep this crate as dev-only but clearly isolate API boundaries and feature-gate it (e.g., default off, feature "dev-keys"), so production builds cannot accidentally enable it.
- Implement an adapter trait (Signer, Verifier) in a separate crate (abyss-crypto-api). Provide a production implementation using audited crates (ed25519-dalek or RustCrypto) and an optional dev implementation in this crate under feature "dev-keys".
- Remove or gate SecretKey::expose_dev_hex behind a debug-only feature and ensure it is never compiled in release builds or when feature "dev-keys" is disabled.
- Use OsRng (rand_core/getrandom) for any non-deterministic key generation when needed; avoid process/time seeding.
- Add migration docs and a deprecation timeline with tests and CI to ensure production builds fail if dev features are enabled.

Next steps
1. Create feature-guarded dev keys and add feature = "dev-keys" default = false to crate Cargo.toml. (PR)
2. Add abyss-crypto-api crate defining traits and interfaces (Signer/Verifier/Keypair) so production implementation can be swapped in. (PR)
3. Implement production adapter using ed25519-dalek or curve25519-dalek with OsRng and secure serialization (secrecy crate). (PR)
4. Add CI checks to fail when dev features are enabled in release builds and add cargo-audit/cargo-deny. (PR)
5. Add tests and fuzz targets for key parsing/serialization. (PR)

Files touched
- crates/abyss-crypto/src/lib.rs (inspected)
- crates/abyss-crypto/Cargo.toml (inspected)

Owner: @APEIRON
