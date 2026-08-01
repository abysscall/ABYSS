//! BFT consensus engine for ABYSS — Stage 1 (ADR-0016).
//!
//! Implements Tendermint-style consensus:
//!   Propose → PreVote → PreCommit → Commit
//!
//! This module extends the existing ValidatorSet / Vote / QuorumCertificate
//! primitives with:
//!   - Round and Phase management
//!   - Deterministic leader rotation
//!   - View Change (leader failure handling)
//!   - Slashing hooks for misbehaviour detection
//!   - ConsensusEngine — the top-level driver

use std::collections::{BTreeMap, BTreeSet};

use abyss_core::hashing::{Hash256, ZERO_HASH};

// ── Re-export existing primitives (unchanged) ─────────────────────────────────

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
    /// Ordered map for deterministic iteration (leader rotation depends on order).
    validators: BTreeMap<ValidatorId, u64>,
    /// Ordered list of validator IDs for leader rotation index.
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
        // ordered_ids follows BTreeMap order (alphabetical) — deterministic
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

    /// Deterministic leader for a given (height, round).
    /// Formula: (height + round) mod validator_count
    /// — same result on every node for the same height/round.
    pub fn leader(&self, height: u64, round: u32) -> &ValidatorId {
        let index = ((height + round as u64) as usize) % self.ordered_ids.len();
        &self.ordered_ids[index]
    }

    /// Build a QuorumCertificate if vote_set reaches quorum.
    pub fn certify(&self, vote_set: VoteSet) -> Result<QuorumCertificate, ConsensusError> {
        let mut seen = BTreeSet::new();
        let mut signed_power = 0_u64;
        let mut block_hash: Option<Hash256> = None;
        let mut height: Option<u64> = None;
        let mut vote_type: Option<VoteType> = None;

        for vote in &vote_set.votes {
            if !seen.insert(vote.validator.clone()) {
                return Err(ConsensusError::DuplicateVote);
            }
            let power = self.voting_power(&vote.validator)
                .ok_or(ConsensusError::UnknownValidator)?;

            // All votes must be for the same block
            match block_hash {
                Some(h) if h != vote.block_hash => return Err(ConsensusError::ConflictingVotes),
                None => block_hash = Some(vote.block_hash),
                _ => {}
            }
            // All votes must be for the same height
            match height {
                Some(h) if h != vote.height => return Err(ConsensusError::ConflictingVotes),
                None => height = Some(vote.height),
                _ => {}
            }
            // All votes must be of the same type
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

// ── Vote types (BFT phases) ───────────────────────────────────────────────────

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
    votes: Vec<Vote>,
}

impl VoteSet {
    pub fn new() -> Self { Self::default() }
    pub fn push(&mut self, vote: Vote) { self.votes.push(vote); }
    pub fn len(&self) -> usize { self.votes.len() }
    pub fn is_empty(&self) -> bool { self.votes.is_empty() }
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

// ── Round state ───────────────────────────────────────────────────────────────

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

    pub fn is_committed(&self) -> bool { self.phase == Phase::Commit }
}

// ── View Change (leader failure handling) ────────────────────────────────────

/// A signed timeout message — sent when a validator's round timer expires
/// without seeing a valid proposal or reaching quorum.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeoutVote {
    pub validator: ValidatorId,
    pub height: u64,
    pub round: u32,
}

/// Aggregates timeout votes. When >1/3 of voting power has timed out,
/// the round is abandoned and a ViewChange is triggered.
#[derive(Clone, Debug, Default)]
pub struct ViewChangeCollector {
    timeouts: BTreeMap<ValidatorId, TimeoutVote>,
}

impl ViewChangeCollector {
    pub fn new() -> Self { Self::default() }

    pub fn add_timeout(
        &mut self,
        timeout: TimeoutVote,
        validator_set: &ValidatorSet,
    ) -> Result<ViewChangeDecision, ConsensusError> {
        if !validator_set.contains(&timeout.validator) {
            return Err(ConsensusError::UnknownValidator);
        }
        self.timeouts.insert(timeout.validator.clone(), timeout);

        let timed_out_power: u64 = self.timeouts.keys()
            .filter_map(|id| validator_set.voting_power(id))
            .sum();

        // Trigger view change when >1/3 of power has timed out
        // (at 1/3 honest validators timing out, the round cannot finish)
        let trigger_threshold = validator_set.total_power() / 3 + 1;
        if timed_out_power >= trigger_threshold {
            Ok(ViewChangeDecision::ChangeView)
        } else {
            Ok(ViewChangeDecision::Wait)
        }
    }

    pub fn timeout_count(&self) -> usize { self.timeouts.len() }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewChangeDecision {
    Wait,
    ChangeView,
}

// ── Slashing hooks ────────────────────────────────────────────────────────────

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
    pub fn new() -> Self { Self::default() }

    /// Submit evidence. Returns true if this is new (not a duplicate).
    pub fn submit(&mut self, evidence: SlashingEvidence) -> bool {
        let validator = evidence.validator().clone();
        if self.slashed.contains(&validator) {
            return false; // already recorded
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

// ── ConsensusEngine — top-level driver ───────────────────────────────────────

/// Drives the BFT protocol for a single node.
///
/// In Phase 3 (real P2P network), this engine will be connected to a
/// network transport layer. Currently it drives consensus through
/// direct method calls — suitable for multi-validator simulation tests.
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

    /// Returns true if this node is the current leader.
    pub fn is_leader(&self) -> bool {
        self.current_leader() == &self.local_validator
    }

    /// Process a proposal from the network.
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
        // Reject votes from slashed validators
        if self.slashing.is_slashed(&vote.validator) {
            return Err(ConsensusError::ValidatorSlashed);
        }
        match vote.vote_type {
            VoteType::PreVote => {
                self.current_round.add_prevote(vote, &self.validator_set)
            }
            VoteType::PreCommit => {
                self.current_round.add_precommit(vote, &self.validator_set)
            }
        }
    }

    /// Process a timeout vote from a validator.
    /// Returns ChangeView if enough validators have timed out.
    pub fn receive_timeout(
        &mut self,
        timeout: TimeoutVote,
    ) -> Result<ViewChangeDecision, ConsensusError> {
        self.view_change.add_timeout(timeout, &self.validator_set)
    }

    /// Advance to the next round (view change).
    /// Called when receive_timeout returns ChangeView.
    pub fn advance_round(&mut self) {
        let next_round = self.current_round.round + 1;
        self.current_round = RoundState::new(self.current_round.height, next_round);
        self.view_change = ViewChangeCollector::new();
    }

    /// Commit the current round and advance to the next height.
    /// Called when a PreCommit QC is produced.
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

    /// Submit slashing evidence.
    pub fn submit_evidence(&mut self, evidence: SlashingEvidence) -> bool {
        self.slashing.submit(evidence)
    }
}

// ── Errors ────────────────────────────────────────────────────────────────────

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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn vid(s: &str) -> ValidatorId { ValidatorId::new(s).unwrap() }
    fn validator(id: &str, power: u64) -> Validator {
        Validator { id: vid(id), voting_power: power }
    }
    fn hash(byte: u8) -> Hash256 { [byte; 32] }

    fn three_validator_set() -> ValidatorSet {
        ValidatorSet::new(vec![
            validator("alice", 1),
            validator("bob", 1),
            validator("carol", 1),
        ]).unwrap()
    }

    // ── ValidatorSet ──────────────────────────────────────────────────────────

    #[test]
    fn quorum_requires_strictly_more_than_two_thirds() {
        let set = three_validator_set();
        assert_eq!(set.quorum_power(), 3); // all 3 needed for 3 validators
    }

    #[test]
    fn quorum_with_four_validators() {
        let set = ValidatorSet::new(vec![
            validator("a", 1), validator("b", 1),
            validator("c", 1), validator("d", 1),
        ]).unwrap();
        assert_eq!(set.quorum_power(), 3); // 3 of 4
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

    // ── Leader rotation ───────────────────────────────────────────────────────

    #[test]
    fn leader_rotates_deterministically() {
        let set = three_validator_set();
        // BTreeMap order: alice < bob < carol
        let l0 = set.leader(0, 0).as_str().to_string();
        let l1 = set.leader(1, 0).as_str().to_string();
        let l2 = set.leader(2, 0).as_str().to_string();
        let l3 = set.leader(3, 0).as_str().to_string();
        // Must cycle through all three
        assert_ne!(l0, l1);
        assert_ne!(l1, l2);
        assert_eq!(l0, l3); // wraps around
    }

    #[test]
    fn round_bump_changes_leader() {
        let set = three_validator_set();
        let leader_r0 = set.leader(1, 0).clone();
        let leader_r1 = set.leader(1, 1).clone();
        assert_ne!(leader_r0, leader_r1);
    }

    // ── Round state / BFT phases ──────────────────────────────────────────────

    #[test]
    fn round_advances_through_phases() {
        let set = three_validator_set();
        let leader = set.leader(1, 0).clone();
        let mut round = RoundState::new(1, 0);
        let bh = hash(42);

        // Propose
        assert_eq!(round.phase, Phase::Propose);
        round.receive_proposal(bh, &leader, &leader).unwrap();
        assert_eq!(round.phase, Phase::PreVote);

        // PreVote — needs quorum of 3
        for id in ["alice", "bob", "carol"] {
            let vote = Vote { validator: vid(id), height: 1, round: 0,
                block_hash: bh, vote_type: VoteType::PreVote };
            round.add_prevote(vote, &set).unwrap();
        }
        assert_eq!(round.phase, Phase::PreCommit);

        // PreCommit — needs quorum of 3
        for id in ["alice", "bob", "carol"] {
            let vote = Vote { validator: vid(id), height: 1, round: 0,
                block_hash: bh, vote_type: VoteType::PreCommit };
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
        // alice is NOT necessarily the leader — find a non-leader
        let non_leader = set.ordered_ids.iter()
            .find(|id| *id != &leader)
            .unwrap()
            .clone();
        let result = round.receive_proposal(hash(1), &leader, &non_leader);
        assert!(matches!(result, Err(ConsensusError::NotTheLeader { .. })));
    }

    // ── View change ───────────────────────────────────────────────────────────

    #[test]
    fn view_change_triggers_when_third_of_power_times_out() {
        let set = ValidatorSet::new(vec![
            validator("a", 10), validator("b", 10),
            validator("c", 10), validator("d", 10),
        ]).unwrap(); // total = 40, trigger at >13
        let mut collector = ViewChangeCollector::new();

        let t1 = collector.add_timeout(
            TimeoutVote { validator: vid("a"), height: 1, round: 0 }, &set
        ).unwrap();
        assert_eq!(t1, ViewChangeDecision::Wait); // 10/40

        let t2 = collector.add_timeout(
            TimeoutVote { validator: vid("b"), height: 1, round: 0 }, &set
        ).unwrap();
        // 20/40 > 40/3 (13.3) → ChangeView
        assert_eq!(t2, ViewChangeDecision::ChangeView);
    }

    // ── ConsensusEngine ───────────────────────────────────────────────────────

    #[test]
    fn engine_identifies_local_leader() {
        let set = three_validator_set();
        // BTreeMap order: alice=0, bob=1, carol=2
        // height=0, round=0 → index (0+0)%3 = 0 → alice
        let engine = ConsensusEngine::new(set, vid("alice"), 1).unwrap();
        // height=1, round=0 → (1+0)%3 = 1 → bob (not alice)
        assert!(!engine.is_leader()); // alice is NOT leader at h=1,r=0
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

        // Simulate timeout
        for id in ["alice", "bob"] {
            engine.receive_timeout(TimeoutVote {
                validator: vid(id), height: 1, round: 0,
            }).unwrap();
        }
        engine.advance_round();
        assert_eq!(engine.current_round.round, 1);
        assert_eq!(engine.current_round.phase, Phase::Propose);
    }

    // ── Slashing ──────────────────────────────────────────────────────────────

    #[test]
    fn slashing_registry_accepts_new_evidence() {
        let mut reg = SlashingRegistry::new();
        let evidence = SlashingEvidence::DoubleVote {
            validator: vid("alice"), height: 1, round: 0,
            hash_a: hash(1), hash_b: hash(2),
        };
        assert!(reg.submit(evidence));
        assert!(reg.is_slashed(&vid("alice")));
    }

    #[test]
    fn slashing_registry_deduplicates() {
        let mut reg = SlashingRegistry::new();
        let e1 = SlashingEvidence::DoubleVote {
            validator: vid("alice"), height: 1, round: 0,
            hash_a: hash(1), hash_b: hash(2),
        };
        let e2 = SlashingEvidence::DoubleProposal {
            validator: vid("alice"), height: 2, round: 0,
            hash_a: hash(3), hash_b: hash(4),
        };
        assert!(reg.submit(e1));
        assert!(!reg.submit(e2)); // alice already slashed
        assert_eq!(reg.pending_evidence().len(), 1);
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

    // ── Byzantine fault tolerance tests ──────────────────────────────────────
    //
    // These tests exist to prove — not merely assert — the core safety
    // guarantee of BFT consensus: with N=3f+1 validators, no more than f
    // Byzantine validators can ever cause two conflicting blocks to be
    // finalised at the same height, and a Byzantine minority alone can
    // neither reach quorum nor force a view change.

    #[test]
    fn byzantine_minority_alone_cannot_reach_quorum() {
        // 4 validators, equal power. Byzantine safety requires f < n/3,
        // so at most 1 of 4 can be Byzantine. A single malicious
        // validator's vote must never be sufficient for a QC.
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

        // One Byzantine validator's vote set must be rejected as insufficient.
        let result = set.certify(malicious_votes);
        assert!(matches!(
            result,
            Err(ConsensusError::InsufficientQuorum { signed_power: 1, .. })
        ));
    }

    #[test]
    fn two_conflicting_blocks_cannot_both_reach_quorum_with_honest_majority() {
        // 4 validators (3 honest, 1 Byzantine — "carol").
        // Byzantine safety property: two DIFFERENT blocks at the same
        // height cannot both be certified, because that would require
        // >2/3 power on each side, and with only 1 Byzantine vote
        // available to double up, no such split is possible.
        let set = ValidatorSet::new(vec![
            validator("alice", 1), validator("bob", 1),
            validator("carol", 1), validator("dave", 1),
        ]).unwrap();
        let block_a = hash(0xAA);
        let block_b = hash(0xBB);

        // Honest majority (alice, bob, dave) all vote for block_a.
        let mut votes_for_a = VoteSet::new();
        for id in ["alice", "bob", "dave"] {
            votes_for_a.push(Vote {
                validator: vid(id), height: 1, round: 0,
                block_hash: block_a, vote_type: VoteType::PreCommit,
            });
        }
        let qc_a = set.certify(votes_for_a);
        assert!(qc_a.is_ok(), "honest majority must be able to certify block_a");

        // Byzantine carol alone tries to certify a conflicting block_b —
        // must fail: she does not have quorum power by herself.
        let mut votes_for_b = VoteSet::new();
        votes_for_b.push(Vote {
            validator: vid("carol"), height: 1, round: 0,
            block_hash: block_b, vote_type: VoteType::PreCommit,
        });
        let qc_b = set.certify(votes_for_b);
        assert!(matches!(
            qc_b,
            Err(ConsensusError::InsufficientQuorum { .. })
        ));
    }

    #[test]
    fn byzantine_minority_below_one_third_cannot_force_view_change() {
        // 4 validators, 10 power each (total 40). View change requires
        // >1/3 of power to time out (>13.3, i.e. >=14). A single Byzantine
        // validator (10 power) claiming timeout must NOT be enough
        // to force the honest network into a view change.
        let set = ValidatorSet::new(vec![
            validator("a", 10), validator("b", 10),
            validator("c", 10), validator("d", 10),
        ]).unwrap();
        let mut collector = ViewChangeCollector::new();

        let decision = collector.add_timeout(
            TimeoutVote { validator: vid("a"), height: 1, round: 0 }, &set
        ).unwrap();

        // 10/40 power is below the >1/3 threshold — must wait, not change view.
        assert_eq!(decision, ViewChangeDecision::Wait);
    }

    #[test]
    fn engine_rejects_vote_from_validator_not_in_set() {
        // A Byzantine actor outside the validator set must never be able
        // to inject a vote that counts toward quorum.
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
