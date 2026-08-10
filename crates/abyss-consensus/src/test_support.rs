//! Shared test helpers for abyss-consensus's module test suites.
//! Test-only; not part of the public API (crate-private, cfg(test) gated
//! from lib.rs).

use abyss_core::hashing::Hash256;

use crate::validator::{Validator, ValidatorId, ValidatorSet};

pub(crate) fn vid(s: &str) -> ValidatorId {
    ValidatorId::new(s).unwrap()
}

pub(crate) fn validator(id: &str, power: u64) -> Validator {
    Validator {
        id: vid(id),
        voting_power: power,
    }
}

pub(crate) fn hash(byte: u8) -> Hash256 {
    [byte; 32]
}

pub(crate) fn three_validator_set() -> ValidatorSet {
    ValidatorSet::new(vec![
        validator("alice", 1),
        validator("bob", 1),
        validator("carol", 1),
    ])
    .unwrap()
}
