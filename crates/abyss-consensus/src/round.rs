//! Round state — the Propose → PreVote → PreCommit → Commit phase machine
//! for a single (height, round) pair.

use abyss_core::hashing::Hash256;

use crate::error::ConsensusError;
use crate::validator::{ValidatorId, ValidatorSet};
use crate::vote::{QuorumCertificate, Vote, VoteSet, VoteType};

/// Current phase within a BFT round.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    /// Waiting for the leader's block proposal.
    Propose,
    /// Collecting PreVote messages.
    PreVote,
    /// Collecting PreCommit messages.
    PreCommit,
    /// Block finalised — advance height.
    Commit,
}

/// State of a single BFT round at a given height.
#[derive(Clone, Debug)]
pub struct RoundState {
    pub height: u64,
    pub round: u32,
    pub phase: Phase,
    pub proposed_block_hash: Option<Hash256>,
    pub prevotes: VoteSet,
    pub precommits: VoteSet,
    pub prevote_qc: Option<QuorumCertificate>,
    pub precommit_qc: Option<QuorumCertificate>,
    /// Monotonic timeout counter — incremented when a phase times out.
    pub timeout_count: u32,
}

impl RoundState {
    pub fn new(height: u64, round: u32) -> Self {
        Self {
            height,
            round,
            phase: Phase::Propose,
            proposed_block_hash: None,
            prevotes: VoteSet::new(),
            precommits: VoteSet::new(),
            prevote_qc: None,
            precommit_qc: None,
            timeout_count: 0,
        }
    }

    /// Receive a proposal from the leader.
    pub fn receive_proposal(
        &mut self,
        block_hash: Hash256,
        leader: &ValidatorId,
        from: &ValidatorId,
    ) -> Result<(), ConsensusError> {
        if self.phase != Phase::Propose {
            return Err(ConsensusError::UnexpectedPhase {
                expected: Phase::Propose,
                actual: self.phase,
            });
        }
        if from != leader {
            return Err(ConsensusError::NotTheLeader {
                expected: leader.clone(),
                actual: from.clone(),
            });
        }
        self.proposed_block_hash = Some(block_hash);
        self.phase = Phase::PreVote;
        Ok(())
    }

    /// Add a PreVote. Returns the QC if quorum is reached.
    pub fn add_prevote(
        &mut self,
        vote: Vote,
        validator_set: &ValidatorSet,
    ) -> Result<Option<QuorumCertificate>, ConsensusError> {
        if vote.vote_type != VoteType::PreVote {
            return Err(ConsensusError::WrongVoteType);
        }
        self.prevotes.push(vote);
        match validator_set.certify(self.prevotes.clone()) {
            Ok(qc) => {
                self.prevote_qc = Some(qc.clone());
                self.phase = Phase::PreCommit;
                Ok(Some(qc))
            }
            Err(ConsensusError::InsufficientQuorum { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Add a PreCommit. Returns the QC if quorum is reached (block finalised).
    pub fn add_precommit(
        &mut self,
        vote: Vote,
        validator_set: &ValidatorSet,
    ) -> Result<Option<QuorumCertificate>, ConsensusError> {
        if vote.vote_type != VoteType::PreCommit {
            return Err(ConsensusError::WrongVoteType);
        }
        self.precommits.push(vote);
        match validator_set.certify(self.precommits.clone()) {
            Ok(qc) => {
                self.precommit_qc = Some(qc.clone());
                self.phase = Phase::Commit;
                Ok(Some(qc))
            }
            Err(ConsensusError::InsufficientQuorum { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn is_committed(&self) -> bool {
        self.phase == Phase::Commit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    #[test]
    fn round_advances_through_phases() {
        let set = three_validator_set();
        let leader = set.leader(1, 0).clone();
        let mut round = RoundState::new(1, 0);
        let bh = hash(42);

        assert_eq!(round.phase, Phase::Propose);
        round.receive_proposal(bh, &leader, &leader).unwrap();
        assert_eq!(round.phase, Phase::PreVote);

        for id in ["alice", "bob", "carol"] {
            let vote = Vote {
                validator: vid(id),
                height: 1,
                round: 0,
                block_hash: bh,
                vote_type: VoteType::PreVote,
            };
            round.add_prevote(vote, &set).unwrap();
        }
        assert_eq!(round.phase, Phase::PreCommit);

        for id in ["alice", "bob", "carol"] {
            let vote = Vote {
                validator: vid(id),
                height: 1,
                round: 0,
                block_hash: bh,
                vote_type: VoteType::PreCommit,
            };
            round.add_precommit(vote, &set).unwrap();
        }
        assert_eq!(round.phase, Phase::Commit);
        assert!(round.is_committed());
    }

    #[test]
    fn proposal_from_wrong_validator_is_rejected() {
        let set = three_validator_set();
        let leader = set.leader(1, 0).clone();
        let mut round = RoundState::new(1, 0);
        let non_leader = set
            .validator_ids()
            .iter()
            .find(|id| *id != &leader)
            .unwrap()
            .clone();
        let result = round.receive_proposal(hash(1), &leader, &non_leader);
        assert!(matches!(result, Err(ConsensusError::NotTheLeader { .. })));
    }
}
