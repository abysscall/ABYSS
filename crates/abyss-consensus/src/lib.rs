//! Early consensus-domain primitives for ABYSS.
//!
//! This is not a complete BFT engine yet. It models validator voting power,
//! duplicate-vote prevention, and quorum certificates so the node can evolve
//! toward a real HotStuff/Tendermint-style protocol.

use std::collections::{BTreeMap, BTreeSet};

use abyss_core::hashing::{Hash256, ZERO_HASH};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ValidatorId(String);

impl ValidatorId {
    pub fn new(value: impl Into<String>) -> Result<Self, ConsensusError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ConsensusError::InvalidValidatorId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
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
    total_power: u64,
}

impl ValidatorSet {
    pub fn new(validators: Vec<Validator>) -> Result<Self, ConsensusError> {
        let mut map = BTreeMap::new();
        let mut total_power = 0_u64;

        for validator in validators {
            if validator.voting_power == 0 {
                return Err(ConsensusError::ZeroVotingPower);
            }
            if map.insert(validator.id, validator.voting_power).is_some() {
                return Err(ConsensusError::DuplicateValidator);
            }
            total_power = total_power
                .checked_add(validator.voting_power)
                .ok_or(ConsensusError::VotingPowerOverflow)?;
        }

        if total_power == 0 {
            return Err(ConsensusError::EmptyValidatorSet);
        }

        Ok(Self {
            validators: map,
            total_power,
        })
    }

    pub fn single_dev_validator(id: ValidatorId) -> Self {
        Self {
            validators: BTreeMap::from([(id, 1)]),
            total_power: 1,
        }
    }

    pub fn total_power(&self) -> u64 {
        self.total_power
    }

    pub fn quorum_power(&self) -> u64 {
        ((self.total_power / 3) * 2) + (((self.total_power % 3) * 2) / 3) + 1
    }

    pub fn voting_power(&self, id: &ValidatorId) -> Option<u64> {
        self.validators.get(id).copied()
    }

    pub fn certify(&self, vote_set: VoteSet) -> Result<QuorumCertificate, ConsensusError> {
        let mut seen = BTreeSet::new();
        let mut signed_power = 0_u64;
        let mut block_hash = None;
        let mut height = None;

        for vote in &vote_set.votes {
            if !seen.insert(vote.validator.clone()) {
                return Err(ConsensusError::DuplicateVote);
            }

            let power = self
                .voting_power(&vote.validator)
                .ok_or(ConsensusError::UnknownValidator)?;

            match block_hash {
                Some(hash) if hash != vote.block_hash => {
                    return Err(ConsensusError::ConflictingVotes);
                }
                None => block_hash = Some(vote.block_hash),
                _ => {}
            }

            match height {
                Some(existing_height) if existing_height != vote.height => {
                    return Err(ConsensusError::ConflictingVotes);
                }
                None => height = Some(vote.height),
                _ => {}
            }

            signed_power = signed_power
                .checked_add(power)
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
            signed_power,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vote {
    pub validator: ValidatorId,
    pub height: u64,
    pub block_hash: Hash256,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VoteSet {
    votes: Vec<Vote>,
}

impl VoteSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, vote: Vote) {
        self.votes.push(vote);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuorumCertificate {
    pub height: u64,
    pub block_hash: Hash256,
    pub signed_power: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConsensusError {
    ConflictingVotes,
    DuplicateValidator,
    DuplicateVote,
    EmptyValidatorSet,
    InsufficientQuorum { signed_power: u64, required_power: u64 },
    InvalidValidatorId,
    UnknownValidator,
    VotingPowerOverflow,
    ZeroVotingPower,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validator(id: &str, voting_power: u64) -> Validator {
        Validator {
            id: ValidatorId::new(id).unwrap(),
            voting_power,
        }
    }

    #[test]
    fn quorum_requires_more_than_two_thirds() {
        let set = ValidatorSet::new(vec![
            validator("a", 1),
            validator("b", 1),
            validator("c", 1),
        ])
        .unwrap();

        assert_eq!(set.quorum_power(), 3);
    }

    #[test]
    fn certifies_matching_votes_with_quorum() {
        let set = ValidatorSet::new(vec![
            validator("a", 1),
            validator("b", 1),
            validator("c", 1),
        ])
        .unwrap();
        let block_hash = [7_u8; 32];
        let mut votes = VoteSet::new();
        votes.push(Vote {
            validator: ValidatorId::new("a").unwrap(),
            height: 1,
            block_hash,
        });
        votes.push(Vote {
            validator: ValidatorId::new("b").unwrap(),
            height: 1,
            block_hash,
        });
        votes.push(Vote {
            validator: ValidatorId::new("c").unwrap(),
            height: 1,
            block_hash,
        });

        let qc = set.certify(votes).unwrap();

        assert_eq!(qc.height, 1);
        assert_eq!(qc.block_hash, block_hash);
        assert_eq!(qc.signed_power, 3);
    }

    #[test]
    fn rejects_conflicting_votes() {
        let set = ValidatorSet::new(vec![validator("a", 2), validator("b", 2)]).unwrap();
        let mut votes = VoteSet::new();
        votes.push(Vote {
            validator: ValidatorId::new("a").unwrap(),
            height: 1,
            block_hash: [1_u8; 32],
        });
        votes.push(Vote {
            validator: ValidatorId::new("b").unwrap(),
            height: 1,
            block_hash: [2_u8; 32],
        });

        assert_eq!(set.certify(votes), Err(ConsensusError::ConflictingVotes));
    }
}
