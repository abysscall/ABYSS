//! Slashing evidence collection.
//!
//! Per ADR-0016 Stage 1 and ADR-0021: this module provides the
//! **API** for recording and checking misbehaviour evidence
//! (double-voting, double-proposing). It does NOT implement the
//! economic penalty (bond reduction, jailing duration, reward
//! forfeiture) — that lives in `ValidatorState` per ADR-0019/0021,
//! not yet implemented. This is intentionally scoped as "Slashing API",
//! not "Slashing Complete" — see ROADMAP.md Stage 1 notes.

use std::collections::BTreeSet;

use abyss_core::hashing::Hash256;

use crate::validator::ValidatorId;

/// Evidence of validator misbehaviour that warrants a slashing penalty.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SlashingEvidence {
    /// Validator voted for two different blocks at the same height and round
    /// (equivocation / double-vote).
    DoubleVote {
        validator: ValidatorId,
        height: u64,
        round: u32,
        hash_a: Hash256,
        hash_b: Hash256,
    },
    /// Validator proposed two different blocks at the same height
    /// (equivocation by proposer).
    DoubleProposal {
        validator: ValidatorId,
        height: u64,
        round: u32,
        hash_a: Hash256,
        hash_b: Hash256,
    },
}

impl SlashingEvidence {
    pub fn validator(&self) -> &ValidatorId {
        match self {
            Self::DoubleVote { validator, .. } => validator,
            Self::DoubleProposal { validator, .. } => validator,
        }
    }
}

/// Collects and deduplicates slashing evidence.
/// The actual penalty (stake reduction) is applied by the chain's State
/// module — this struct only records and validates evidence.
#[derive(Clone, Debug, Default)]
pub struct SlashingRegistry {
    evidence: Vec<SlashingEvidence>,
    slashed: BTreeSet<ValidatorId>,
}

impl SlashingRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Submit evidence. Returns true if this is new (not a duplicate).
    pub fn submit(&mut self, evidence: SlashingEvidence) -> bool {
        let validator = evidence.validator().clone();
        if self.slashed.contains(&validator) {
            return false;
        }
        self.slashed.insert(validator);
        self.evidence.push(evidence);
        true
    }

    pub fn is_slashed(&self, id: &ValidatorId) -> bool {
        self.slashed.contains(id)
    }

    pub fn pending_evidence(&self) -> &[SlashingEvidence] {
        &self.evidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    #[test]
    fn slashing_registry_accepts_new_evidence() {
        let mut reg = SlashingRegistry::new();
        let evidence = SlashingEvidence::DoubleVote {
            validator: vid("alice"),
            height: 1,
            round: 0,
            hash_a: hash(1),
            hash_b: hash(2),
        };
        assert!(reg.submit(evidence));
        assert!(reg.is_slashed(&vid("alice")));
    }

    #[test]
    fn slashing_registry_deduplicates() {
        let mut reg = SlashingRegistry::new();
        let e1 = SlashingEvidence::DoubleVote {
            validator: vid("alice"),
            height: 1,
            round: 0,
            hash_a: hash(1),
            hash_b: hash(2),
        };
        let e2 = SlashingEvidence::DoubleProposal {
            validator: vid("alice"),
            height: 2,
            round: 0,
            hash_a: hash(3),
            hash_b: hash(4),
        };
        assert!(reg.submit(e1));
        assert!(!reg.submit(e2));
        assert_eq!(reg.pending_evidence().len(), 1);
    }
}
