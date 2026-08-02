//! ConsensusEngine — the top-level driver tying together ValidatorSet,
//! RoundState, ViewChangeCollector, and SlashingRegistry into a single
//! per-node BFT state machine.
//!
//! In Stage 3 (P2P), this engine will be connected to a network
//! transport layer. Currently it drives consensus through direct method
//! calls — suitable for multi-validator simulation and testing.

use abyss_core::hashing::Hash256;

use crate::error::ConsensusError;
use crate::round::RoundState;
use crate::slashing::{SlashingEvidence, SlashingRegistry};
use crate::validator::{ValidatorId, ValidatorSet};
use crate::view_change::{TimeoutVote, ViewChangeCollector, ViewChangeDecision};
use crate::vote::{QuorumCertificate, Vote, VoteType};

pub struct ConsensusEngine {
    pub validator_set: ValidatorSet,
    pub local_validator: ValidatorId,
    pub current_round: RoundState,
    pub committed_height: u64,
    pub view_change: ViewChangeCollector,
    pub slashing: SlashingRegistry,
}

impl ConsensusEngine {
    pub fn new(
        validator_set: ValidatorSet,
        local_validator: ValidatorId,
        starting_height: u64,
    ) -> Result<Self, ConsensusError> {
        if !validator_set.contains(&local_validator) {
            return Err(ConsensusError::UnknownValidator);
        }
        Ok(Self {
            current_round: RoundState::new(starting_height, 0),
            committed_height: starting_height.saturating_sub(1),
            view_change: ViewChangeCollector::new(),
            slashing: SlashingRegistry::new(),
            validator_set,
            local_validator,
        })
    }

    /// Returns the current leader (deterministic, based on height + round).
    pub fn current_leader(&self) -> &ValidatorId {
        self.validator_set.leader(
            self.current_round.height,
            self.current_round.round,
        )
    }

    pub fn is_leader(&self) -> bool {
        self.current_leader() == &self.local_validator
    }

    pub fn receive_proposal(
        &mut self,
        block_hash: Hash256,
        from: &ValidatorId,
    ) -> Result<(), ConsensusError> {
        let leader = self.current_leader().clone();
        self.current_round.receive_proposal(block_hash, &leader, from)
    }

    /// Process a vote from the network.
    /// Returns Some(QC) if the vote completes a quorum.
    pub fn receive_vote(
        &mut self,
        vote: Vote,
    ) -> Result<Option<QuorumCertificate>, ConsensusError> {
        if self.slashing.is_slashed(&vote.validator) {
            return Err(ConsensusError::ValidatorSlashed);
        }
        match vote.vote_type {
            VoteType::PreVote => self.current_round.add_prevote(vote, &self.validator_set),
            VoteType::PreCommit => self.current_round.add_precommit(vote, &self.validator_set),
        }
    }

    /// Process a timeout vote from a validator.
    pub fn receive_timeout(
        &mut self,
        timeout: TimeoutVote,
    ) -> Result<ViewChangeDecision, ConsensusError> {
        self.view_change.add_timeout(timeout, &self.validator_set)
    }

    /// Advance to the next round (view change).
    pub fn advance_round(&mut self) {
        let next_round = self.current_round.round + 1;
        self.current_round = RoundState::new(self.current_round.height, next_round);
        self.view_change = ViewChangeCollector::new();
    }

    /// Commit the current round and advance to the next height.
    ///
    /// NOTE (ADR-0017): this currently only advances the consensus
    /// engine's own height counter. It does NOT yet call
    /// `abyss_core::Chain::apply_block()` to commit ledger state —
    /// wiring that connection is tracked follow-up work per ADR-0017.
    pub fn commit(&mut self) -> Result<u64, ConsensusError> {
        if !self.current_round.is_committed() {
            return Err(ConsensusError::NotCommitted);
        }
        let committed_height = self.current_round.height;
        self.committed_height = committed_height;
        let next_height = committed_height + 1;
        self.current_round = RoundState::new(next_height, 0);
        self.view_change = ViewChangeCollector::new();
        Ok(committed_height)
    }

    pub fn submit_evidence(&mut self, evidence: SlashingEvidence) -> bool {
        self.slashing.submit(evidence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

    #[test]
    fn engine_identifies_local_leader() {
        let set = three_validator_set();
        let engine = ConsensusEngine::new(set, vid("alice"), 1).unwrap();
        // height=1, round=0 → (1+0)%3 = 1 → bob (not alice)
        assert!(!engine.is_leader());
    }

    #[test]
    fn engine_full_round_commit_advances_height() {
        let set = three_validator_set();
        let leader = set.leader(1, 0).clone();
        let mut engine = ConsensusEngine::new(set, leader.clone(), 1).unwrap();
        let bh = hash(99);

        engine.receive_proposal(bh, &leader).unwrap();

        for id in ["alice", "bob", "carol"] {
            engine.receive_vote(Vote { validator: vid(id), height: 1, round: 0,
                block_hash: bh, vote_type: VoteType::PreVote }).unwrap();
        }
        for id in ["alice", "bob", "carol"] {
            engine.receive_vote(Vote { validator: vid(id), height: 1, round: 0,
                block_hash: bh, vote_type: VoteType::PreCommit }).unwrap();
        }

        assert!(engine.current_round.is_committed());
        let committed = engine.commit().unwrap();
        assert_eq!(committed, 1);
        assert_eq!(engine.current_round.height, 2);
        assert_eq!(engine.committed_height, 1);
    }

    #[test]
    fn engine_advance_round_resets_state() {
        let set = three_validator_set();
        let leader = set.leader(1, 0).clone();
        let mut engine = ConsensusEngine::new(set, leader.clone(), 1).unwrap();

        for id in ["alice", "bob"] {
            engine.receive_timeout(TimeoutVote {
                validator: vid(id), height: 1, round: 0,
            }).unwrap();
        }
        engine.advance_round();
        assert_eq!(engine.current_round.round, 1);
        assert_eq!(engine.current_round.phase, crate::round::Phase::Propose);
    }

    #[test]
    fn slashed_validator_votes_are_rejected_by_engine() {
        let set = three_validator_set();
        let leader = set.leader(1, 0).clone();
        let mut engine = ConsensusEngine::new(set, leader.clone(), 1).unwrap();
        let bh = hash(1);

        engine.receive_proposal(bh, &leader).unwrap();
        engine.submit_evidence(SlashingEvidence::DoubleVote {
            validator: vid("alice"), height: 1, round: 0,
            hash_a: hash(1), hash_b: hash(2),
        });

        let result = engine.receive_vote(Vote {
            validator: vid("alice"), height: 1, round: 0,
            block_hash: bh, vote_type: VoteType::PreVote,
        });
        assert_eq!(result, Err(ConsensusError::ValidatorSlashed));
    }

    // ── Byzantine test ────────────────────────────────────────────────────

    #[test]
    fn engine_rejects_vote_from_validator_not_in_set() {
        let set = three_validator_set();
        let leader = set.leader(1, 0).clone();
        let mut engine = ConsensusEngine::new(set, leader.clone(), 1).unwrap();
        let bh = hash(1);
        engine.receive_proposal(bh, &leader).unwrap();

        let outsider_vote = Vote {
            validator: vid("mallory-not-a-validator"),
            height: 1, round: 0,
            block_hash: bh, vote_type: VoteType::PreVote,
        };
        let result = engine.receive_vote(outsider_vote);
        assert_eq!(result, Err(ConsensusError::UnknownValidator));
    }
}
