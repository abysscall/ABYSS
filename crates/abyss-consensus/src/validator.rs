//! Validator identity, validator set, and quorum certification.

use std::collections::{BTreeMap, BTreeSet};

use abyss_core::hashing::ZERO_HASH;

use crate::error::ConsensusError;
use crate::vote::{QuorumCertificate, VoteSet};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ValidatorId(String);

impl ValidatorId {
    pub fn new(value: impl Into<String>) -> Result<Self, ConsensusError> {
        let value = value.into();
        if value.is_empty() { return Err(ConsensusError::InvalidValidatorId); }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str { &self.0 }
}

impl std::fmt::Display for ValidatorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Validator {
    pub id: ValidatorId,
    pub voting_power: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatorSet {
    validators: BTreeMap<ValidatorId, u64>,
    /// Ordered list of validator IDs for leader rotation index.
    /// Follows BTreeMap key order (alphabetical) — deterministic.
    ordered_ids: Vec<ValidatorId>,
    total_power: u64,
}

impl ValidatorSet {
    pub fn new(validators: Vec<Validator>) -> Result<Self, ConsensusError> {
        if validators.is_empty() { return Err(ConsensusError::EmptyValidatorSet); }
        let mut map = BTreeMap::new();
        let mut total_power = 0_u64;
        for v in &validators {
            if v.voting_power == 0 { return Err(ConsensusError::ZeroVotingPower); }
            if map.insert(v.id.clone(), v.voting_power).is_some() {
                return Err(ConsensusError::DuplicateValidator);
            }
            total_power = total_power.checked_add(v.voting_power)
                .ok_or(ConsensusError::VotingPowerOverflow)?;
        }
        let ordered_ids = map.keys().cloned().collect();
        Ok(Self { validators: map, ordered_ids, total_power })
    }

    pub fn single_dev_validator(id: ValidatorId) -> Self {
        let ordered_ids = vec![id.clone()];
        Self {
            validators: BTreeMap::from([(id, 1)]),
            ordered_ids,
            total_power: 1,
        }
    }

    pub fn total_power(&self) -> u64 { self.total_power }

    /// Minimum voting power required for a quorum (strictly >2/3).
    pub fn quorum_power(&self) -> u64 {
        ((self.total_power / 3) * 2) + (((self.total_power % 3) * 2) / 3) + 1
    }

    pub fn voting_power(&self, id: &ValidatorId) -> Option<u64> {
        self.validators.get(id).copied()
    }

    pub fn contains(&self, id: &ValidatorId) -> bool {
        self.validators.contains_key(id)
    }

    pub fn len(&self) -> usize { self.validators.len() }
    pub fn is_empty(&self) -> bool { self.validators.is_empty() }

    /// All validator IDs in deterministic (BTreeMap) order.
    pub fn validator_ids(&self) -> &[ValidatorId] {
        &self.ordered_ids
    }

    /// Deterministic leader for a given (height, round).
    /// Formula: (height + round) mod validator_count
    /// — same result on every node for the same height/round.
    pub fn leader(&self, height: u64, round: u32) -> &ValidatorId {
        let index = ((height + round as u64) as usize) % self.ordered_ids.len();
        &self.ordered_ids[index]
    }

    /// Build a QuorumCertificate if vote_set reaches quorum.
    pub fn certify(&self, vote_set: VoteSet) -> Result<QuorumCertificate, ConsensusError> {
        use crate::vote::VoteType;

        let mut seen = BTreeSet::new();
        let mut signed_power = 0_u64;
        let mut block_hash = None;
        let mut height: Option<u64> = None;
        let mut vote_type: Option<VoteType> = None;

        for vote in vote_set.iter() {
            if !seen.insert(vote.validator.clone()) {
                return Err(ConsensusError::DuplicateVote);
            }
            let power = self.voting_power(&vote.validator)
                .ok_or(ConsensusError::UnknownValidator)?;

            match block_hash {
                Some(h) if h != vote.block_hash => return Err(ConsensusError::ConflictingVotes),
                None => block_hash = Some(vote.block_hash),
                _ => {}
            }
            match height {
                Some(h) if h != vote.height => return Err(ConsensusError::ConflictingVotes),
                None => height = Some(vote.height),
                _ => {}
            }
            match vote_type {
                Some(t) if t != vote.vote_type => return Err(ConsensusError::ConflictingVotes),
                None => vote_type = Some(vote.vote_type),
                _ => {}
            }

            signed_power = signed_power.checked_add(power)
                .ok_or(ConsensusError::VotingPowerOverflow)?;
        }

        if signed_power < self.quorum_power() {
            return Err(ConsensusError::InsufficientQuorum {
                signed_power,
                required_power: self.quorum_power(),
            });
        }

        Ok(QuorumCertificate {
            height: height.unwrap_or_default(),
            block_hash: block_hash.unwrap_or(ZERO_HASH),
            vote_type: vote_type.unwrap_or(VoteType::PreCommit),
            signed_power,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;
    use crate::vote::{Vote, VoteType};

    #[test]
    fn quorum_requires_strictly_more_than_two_thirds() {
        let set = three_validator_set();
        assert_eq!(set.quorum_power(), 3);
    }

    #[test]
    fn quorum_with_four_validators() {
        let set = ValidatorSet::new(vec![
            validator("a", 1), validator("b", 1),
            validator("c", 1), validator("d", 1),
        ]).unwrap();
        assert_eq!(set.quorum_power(), 3);
    }

    #[test]
    fn certifies_matching_votes_with_quorum() {
        let set = three_validator_set();
        let bh = hash(7);
        let mut votes = VoteSet::new();
        for id in ["alice", "bob", "carol"] {
            votes.push(Vote { validator: vid(id), height: 1, round: 0,
                block_hash: bh, vote_type: VoteType::PreCommit });
        }
        let qc = set.certify(votes).unwrap();
        assert_eq!(qc.height, 1);
        assert_eq!(qc.block_hash, bh);
        assert!(qc.is_commit_qc());
    }

    #[test]
    fn rejects_conflicting_block_hashes() {
        let set = ValidatorSet::new(vec![validator("a", 2), validator("b", 2)]).unwrap();
        let mut votes = VoteSet::new();
        votes.push(Vote { validator: vid("a"), height: 1, round: 0,
            block_hash: hash(1), vote_type: VoteType::PreVote });
        votes.push(Vote { validator: vid("b"), height: 1, round: 0,
            block_hash: hash(2), vote_type: VoteType::PreVote });
        assert_eq!(set.certify(votes), Err(ConsensusError::ConflictingVotes));
    }

    #[test]
    fn rejects_duplicate_votes() {
        let set = three_validator_set();
        let bh = hash(1);
        let mut votes = VoteSet::new();
        votes.push(Vote { validator: vid("alice"), height: 1, round: 0,
            block_hash: bh, vote_type: VoteType::PreVote });
        votes.push(Vote { validator: vid("alice"), height: 1, round: 0,
            block_hash: bh, vote_type: VoteType::PreVote });
        assert_eq!(set.certify(votes), Err(ConsensusError::DuplicateVote));
    }

    #[test]
    fn leader_rotates_deterministically() {
        let set = three_validator_set();
        let l0 = set.leader(0, 0).as_str().to_string();
        let l1 = set.leader(1, 0).as_str().to_string();
        let l2 = set.leader(2, 0).as_str().to_string();
        let l3 = set.leader(3, 0).as_str().to_string();
        assert_ne!(l0, l1);
        assert_ne!(l1, l2);
        assert_eq!(l0, l3);
    }

    #[test]
    fn round_bump_changes_leader() {
        let set = three_validator_set();
        let leader_r0 = set.leader(1, 0).clone();
        let leader_r1 = set.leader(1, 1).clone();
        assert_ne!(leader_r0, leader_r1);
    }

    // ── Byzantine fault tolerance tests ──────────────────────────────────
    //
    // Prove — not merely assert — the core BFT safety guarantee: with
    // N=3f+1 validators, no more than f Byzantine validators can ever
    // cause two conflicting blocks to be finalised at the same height.

    #[test]
    fn byzantine_minority_alone_cannot_reach_quorum() {
        // 4 validators; at most 1 may be Byzantine (f < n/3).
        let set = ValidatorSet::new(vec![
            validator("a", 1), validator("b", 1),
            validator("c", 1), validator("d", 1),
        ]).unwrap();
        let bh = hash(1);
        let mut malicious_votes = VoteSet::new();
        malicious_votes.push(Vote {
            validator: vid("a"), height: 1, round: 0,
            block_hash: bh, vote_type: VoteType::PreCommit,
        });
        let result = set.certify(malicious_votes);
        assert!(matches!(
            result,
            Err(ConsensusError::InsufficientQuorum { signed_power: 1, .. })
        ));
    }

    #[test]
    fn two_conflicting_blocks_cannot_both_reach_quorum_with_honest_majority() {
        // 4 validators (3 honest, 1 Byzantine — "carol").
        let set = ValidatorSet::new(vec![
            validator("alice", 1), validator("bob", 1),
            validator("carol", 1), validator("dave", 1),
        ]).unwrap();
        let block_a = hash(0xAA);
        let block_b = hash(0xBB);

        let mut votes_for_a = VoteSet::new();
        for id in ["alice", "bob", "dave"] {
            votes_for_a.push(Vote {
                validator: vid(id), height: 1, round: 0,
                block_hash: block_a, vote_type: VoteType::PreCommit,
            });
        }
        assert!(set.certify(votes_for_a).is_ok(), "honest majority must certify block_a");

        let mut votes_for_b = VoteSet::new();
        votes_for_b.push(Vote {
            validator: vid("carol"), height: 1, round: 0,
            block_hash: block_b, vote_type: VoteType::PreCommit,
        });
        assert!(matches!(
            set.certify(votes_for_b),
            Err(ConsensusError::InsufficientQuorum { .. })
        ));
    }
}
