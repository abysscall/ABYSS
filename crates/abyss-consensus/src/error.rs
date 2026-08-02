//! Consensus error types — shared across all abyss-consensus modules.

use crate::round::Phase;
use crate::validator::ValidatorId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConsensusError {
    ConflictingVotes,
    DuplicateValidator,
    DuplicateVote,
    EmptyValidatorSet,
    InsufficientQuorum { signed_power: u64, required_power: u64 },
    InvalidValidatorId,
    NotCommitted,
    NotTheLeader { expected: ValidatorId, actual: ValidatorId },
    UnexpectedPhase { expected: Phase, actual: Phase },
    UnknownValidator,
    ValidatorSlashed,
    VotingPowerOverflow,
    WrongVoteType,
    ZeroVotingPower,
}

impl std::fmt::Display for ConsensusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConflictingVotes => write!(f, "conflicting votes in vote set"),
            Self::DuplicateValidator => write!(f, "duplicate validator id"),
            Self::DuplicateVote => write!(f, "duplicate vote from same validator"),
            Self::EmptyValidatorSet => write!(f, "validator set is empty"),
            Self::InsufficientQuorum { signed_power, required_power } =>
                write!(f, "insufficient quorum: {signed_power}/{required_power}"),
            Self::InvalidValidatorId => write!(f, "validator id must not be empty"),
            Self::NotCommitted => write!(f, "round is not in Commit phase"),
            Self::NotTheLeader { expected, actual } =>
                write!(f, "proposal from {actual}, expected leader {expected}"),
            Self::UnexpectedPhase { expected, actual } =>
                write!(f, "unexpected phase: expected {expected:?}, got {actual:?}"),
            Self::UnknownValidator => write!(f, "unknown validator"),
            Self::ValidatorSlashed => write!(f, "validator has been slashed"),
            Self::VotingPowerOverflow => write!(f, "voting power overflow"),
            Self::WrongVoteType => write!(f, "wrong vote type for current phase"),
            Self::ZeroVotingPower => write!(f, "validator voting power must be non-zero"),
        }
    }
}

impl std::error::Error for ConsensusError {}
