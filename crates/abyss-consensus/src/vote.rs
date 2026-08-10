//! Vote types and quorum certificates.
//!
//! A `Vote` is one validator's signed statement about a block at a given
//! height/round/phase. A `QuorumCertificate` is proof that >2/3 of voting
//! power agreed — produced by `ValidatorSet::certify()` (validator.rs).

use abyss_core::hashing::Hash256;

use crate::validator::ValidatorId;

/// Phase of the BFT protocol a vote belongs to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VoteType {
    /// Validator signals readiness to accept a proposed block.
    PreVote,
    /// Validator commits to a block after seeing a PreVote quorum.
    PreCommit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vote {
    pub validator: ValidatorId,
    pub height: u64,
    pub round: u32,
    pub block_hash: Hash256,
    pub vote_type: VoteType,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VoteSet {
    pub(crate) votes: Vec<Vote>,
}

impl VoteSet {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&mut self, vote: Vote) {
        self.votes.push(vote);
    }
    pub fn len(&self) -> usize {
        self.votes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.votes.is_empty()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, Vote> {
        self.votes.iter()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuorumCertificate {
    pub height: u64,
    pub block_hash: Hash256,
    pub vote_type: VoteType,
    pub signed_power: u64,
}

impl QuorumCertificate {
    /// A PreCommit QC is what finalises a block in Tendermint-style BFT.
    pub fn is_commit_qc(&self) -> bool {
        self.vote_type == VoteType::PreCommit
    }
}
