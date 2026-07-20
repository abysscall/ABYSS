//! Tendermint-style multi-validator BFT round engine.
//!
//! A [`BftRound`] drives a single (height, round) through the classic phases:
//!
//! ```text
//! Propose → Prevote → Precommit → Committed
//! ```
//!
//! A block is committed for the round once precommits for that block hash reach
//! the validator set's quorum (> 2/3 of voting power). Rounds that fail to
//! commit (a silent or Byzantine proposer, a split vote) are abandoned and a new
//! round is started with the next proposer — the classic view change. This is a
//! deterministic, in-memory engine for devnet simulation; peer-to-peer message
//! transport is a separate later layer.

use std::collections::{BTreeMap, BTreeSet};

use abyss_core::hashing::Hash256;

use crate::{QuorumCertificate, ValidatorId, ValidatorSet, Vote};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Step {
    Propose,
    Prevote,
    Precommit,
    Committed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub height: u64,
    pub round: u64,
    pub block_hash: Hash256,
    pub proposer: ValidatorId,
}

/// Emitted when a round reaches a committed block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitEvent {
    pub height: u64,
    pub round: u64,
    pub block_hash: Hash256,
    pub certificate: QuorumCertificate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RoundError {
    AlreadyCommitted,
    DuplicatePrecommit,
    DuplicatePrevote,
    NotProposed,
    UnknownValidator,
    WrongHeight,
    WrongProposer,
    WrongStep,
}

/// Deterministic round-robin proposer selection over the validator set, indexed
/// by `(height + round)`. Every round of a height picks the next validator, so a
/// silent proposer is skipped by the view change.
pub fn proposer_for(set: &ValidatorSet, height: u64, round: u64) -> ValidatorId {
    let ids: Vec<&ValidatorId> = set.ids().collect();
    // ValidatorSet cannot be empty (its constructor rejects an empty set).
    let index = ((u128::from(height) + u128::from(round)) % ids.len() as u128) as usize;
    ids[index].clone()
}

#[derive(Clone, Debug)]
pub struct BftRound {
    set: ValidatorSet,
    height: u64,
    round: u64,
    step: Step,
    proposer: ValidatorId,
    proposal: Option<Proposal>,
    prevotes: Vec<Vote>,
    precommits: Vec<Vote>,
    seen_prevote: BTreeSet<ValidatorId>,
    seen_precommit: BTreeSet<ValidatorId>,
}

impl BftRound {
    pub fn new(set: ValidatorSet, height: u64, round: u64) -> Self {
        let proposer = proposer_for(&set, height, round);
        Self {
            set,
            height,
            round,
            step: Step::Propose,
            proposer,
            proposal: None,
            prevotes: Vec::new(),
            precommits: Vec::new(),
            seen_prevote: BTreeSet::new(),
            seen_precommit: BTreeSet::new(),
        }
    }

    pub fn height(&self) -> u64 {
        self.height
    }

    pub fn round(&self) -> u64 {
        self.round
    }

    pub fn step(&self) -> Step {
        self.step
    }

    pub fn proposer(&self) -> &ValidatorId {
        &self.proposer
    }

    pub fn proposal(&self) -> Option<&Proposal> {
        self.proposal.as_ref()
    }

    /// The round proposer proposes a block. Only valid in the `Propose` step and
    /// only from the designated proposer; advances the round to `Prevote`.
    pub fn propose(
        &mut self,
        from: &ValidatorId,
        block_hash: Hash256,
    ) -> Result<&Proposal, RoundError> {
        if self.step != Step::Propose {
            return Err(RoundError::WrongStep);
        }
        if from != &self.proposer {
            return Err(RoundError::WrongProposer);
        }
        self.proposal = Some(Proposal {
            height: self.height,
            round: self.round,
            block_hash,
            proposer: self.proposer.clone(),
        });
        self.step = Step::Prevote;
        Ok(self.proposal.as_ref().expect("just set"))
    }

    /// Records a prevote. Valid only in the `Prevote` step, from a known
    /// validator, at this round's height, and at most once per validator.
    pub fn add_prevote(&mut self, vote: Vote) -> Result<(), RoundError> {
        if self.step != Step::Prevote {
            return Err(RoundError::WrongStep);
        }
        self.validate_vote(&vote)?;
        if !self.seen_prevote.insert(vote.validator.clone()) {
            return Err(RoundError::DuplicatePrevote);
        }
        self.prevotes.push(vote);
        Ok(())
    }

    /// If prevotes for a single block hash have reached quorum, advances to the
    /// `Precommit` step and returns that block hash.
    pub fn try_enter_precommit(&mut self) -> Option<Hash256> {
        if self.step != Step::Prevote {
            return None;
        }
        let (block_hash, _) = tally_quorum(&self.set, &self.prevotes)?;
        self.step = Step::Precommit;
        Some(block_hash)
    }

    /// Records a precommit. Valid only in the `Precommit` step, from a known
    /// validator, at this round's height, and at most once per validator.
    pub fn add_precommit(&mut self, vote: Vote) -> Result<(), RoundError> {
        if self.step != Step::Precommit {
            return Err(RoundError::WrongStep);
        }
        self.validate_vote(&vote)?;
        if !self.seen_precommit.insert(vote.validator.clone()) {
            return Err(RoundError::DuplicatePrecommit);
        }
        self.precommits.push(vote);
        Ok(())
    }

    /// If precommits for a single block hash have reached quorum, marks the round
    /// `Committed` and returns the commit event.
    pub fn try_commit(&mut self) -> Option<CommitEvent> {
        if self.step != Step::Precommit {
            return None;
        }
        let (block_hash, signed_power) = tally_quorum(&self.set, &self.precommits)?;
        self.step = Step::Committed;
        Some(CommitEvent {
            height: self.height,
            round: self.round,
            block_hash,
            certificate: QuorumCertificate {
                height: self.height,
                block_hash,
                signed_power,
            },
        })
    }

    fn validate_vote(&self, vote: &Vote) -> Result<(), RoundError> {
        if vote.height != self.height {
            return Err(RoundError::WrongHeight);
        }
        if self.set.voting_power(&vote.validator).is_none() {
            return Err(RoundError::UnknownValidator);
        }
        if self.proposal.is_none() {
            return Err(RoundError::NotProposed);
        }
        Ok(())
    }
}

/// Sums the voting power backing each distinct block hash (counting each
/// validator once per block) and returns the first block hash whose power
/// reaches quorum, along with that power. Conflicting votes for other blocks are
/// ignored — BFT requires quorum on the *same* block.
fn tally_quorum(set: &ValidatorSet, votes: &[Vote]) -> Option<(Hash256, u64)> {
    let mut by_block: BTreeMap<Hash256, (BTreeSet<ValidatorId>, u64)> = BTreeMap::new();
    for vote in votes {
        let Some(power) = set.voting_power(&vote.validator) else {
            continue;
        };
        let entry = by_block.entry(vote.block_hash).or_default();
        if entry.0.insert(vote.validator.clone()) {
            entry.1 = entry.1.saturating_add(power);
        }
    }

    let quorum = set.quorum_power();
    by_block
        .into_iter()
        .find(|(_, (_, power))| *power >= quorum)
        .map(|(hash, (_, power))| (hash, power))
}

/// Drives a full height to finality, honest validators only.
///
/// For each round, the round-robin proposer is checked: if it is not in
/// `honest` the round is abandoned (view change to the next proposer). Otherwise
/// the honest validators prevote and precommit for `block_hash`; the height
/// commits as soon as precommits reach quorum. Returns `None` if no round within
/// `max_rounds` commits (e.g. too few honest validators for quorum).
pub fn finalize_height(
    set: &ValidatorSet,
    height: u64,
    block_hash: Hash256,
    honest: &[ValidatorId],
    max_rounds: u64,
) -> Option<CommitEvent> {
    for round in 0..max_rounds {
        let proposer = proposer_for(set, height, round);
        if !honest.contains(&proposer) {
            continue;
        }

        let mut engine = BftRound::new(set.clone(), height, round);
        if engine.propose(&proposer, block_hash).is_err() {
            continue;
        }

        for id in honest {
            let _ = engine.add_prevote(Vote {
                validator: id.clone(),
                height,
                block_hash,
            });
        }
        if engine.try_enter_precommit().is_none() {
            continue;
        }

        for id in honest {
            let _ = engine.add_precommit(Vote {
                validator: id.clone(),
                height,
                block_hash,
            });
        }
        if let Some(event) = engine.try_commit() {
            return Some(event);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Validator;

    fn set(ids: &[&str]) -> ValidatorSet {
        ValidatorSet::new(
            ids.iter()
                .map(|id| Validator {
                    id: ValidatorId::new(*id).unwrap(),
                    voting_power: 1,
                })
                .collect(),
        )
        .unwrap()
    }

    fn vid(id: &str) -> ValidatorId {
        ValidatorId::new(id).unwrap()
    }

    fn ids(names: &[&str]) -> Vec<ValidatorId> {
        names.iter().map(|n| vid(n)).collect()
    }

    #[test]
    fn proposer_rotates_with_round() {
        let s = set(&["a", "b", "c"]);
        let p0 = proposer_for(&s, 1, 0);
        let p1 = proposer_for(&s, 1, 1);
        let p2 = proposer_for(&s, 1, 2);
        assert_ne!(p0, p1);
        assert_ne!(p1, p2);
        // wraps around after a full rotation
        assert_eq!(proposer_for(&s, 1, 3), p0);
    }

    #[test]
    fn honest_full_set_commits() {
        let s = set(&["a", "b", "c", "d"]);
        let block = [9_u8; 32];
        let event = finalize_height(&s, 5, block, &ids(&["a", "b", "c", "d"]), 4).unwrap();
        assert_eq!(event.height, 5);
        assert_eq!(event.block_hash, block);
        assert_eq!(event.certificate.signed_power, 4);
    }

    #[test]
    fn commits_with_one_byzantine_of_four() {
        // 4 validators, quorum = 3. One silent validator still allows commit.
        let s = set(&["a", "b", "c", "d"]);
        let block = [1_u8; 32];
        let event = finalize_height(&s, 1, block, &ids(&["a", "b", "c"]), 8).unwrap();
        assert_eq!(event.block_hash, block);
        assert!(event.certificate.signed_power >= s.quorum_power());
    }

    #[test]
    fn no_commit_when_two_of_four_are_faulty() {
        // Only 2 honest of 4 (quorum 3): no round can reach quorum.
        let s = set(&["a", "b", "c", "d"]);
        assert!(finalize_height(&s, 1, [2_u8; 32], &ids(&["a", "b"]), 16).is_none());
    }

    #[test]
    fn split_prevotes_do_not_reach_precommit() {
        let s = set(&["a", "b", "c", "d"]);
        let mut round = BftRound::new(s, 7, 0);
        let proposer = round.proposer().clone();
        round.propose(&proposer, [4_u8; 32]).unwrap();
        // two validators prevote one block, two prevote another → no quorum
        round
            .add_prevote(Vote { validator: vid("a"), height: 7, block_hash: [4_u8; 32] })
            .unwrap();
        round
            .add_prevote(Vote { validator: vid("b"), height: 7, block_hash: [4_u8; 32] })
            .unwrap();
        round
            .add_prevote(Vote { validator: vid("c"), height: 7, block_hash: [5_u8; 32] })
            .unwrap();
        round
            .add_prevote(Vote { validator: vid("d"), height: 7, block_hash: [5_u8; 32] })
            .unwrap();
        assert!(round.try_enter_precommit().is_none());
        assert_eq!(round.step(), Step::Prevote);
    }

    #[test]
    fn rejects_wrong_proposer_and_duplicates() {
        let s = set(&["a", "b", "c"]);
        let mut round = BftRound::new(s, 1, 0);
        let proposer = round.proposer().clone();
        let not_proposer = ["a", "b", "c"]
            .iter()
            .map(|n| vid(n))
            .find(|id| id != &proposer)
            .unwrap();

        assert_eq!(
            round.propose(&not_proposer, [1_u8; 32]),
            Err(RoundError::WrongProposer)
        );
        round.propose(&proposer, [1_u8; 32]).unwrap();

        round
            .add_prevote(Vote { validator: vid("a"), height: 1, block_hash: [1_u8; 32] })
            .unwrap();
        assert_eq!(
            round.add_prevote(Vote { validator: vid("a"), height: 1, block_hash: [1_u8; 32] }),
            Err(RoundError::DuplicatePrevote)
        );
    }

    #[test]
    fn rejects_unknown_validator_and_wrong_height() {
        let s = set(&["a", "b", "c"]);
        let mut round = BftRound::new(s, 2, 0);
        let proposer = round.proposer().clone();
        round.propose(&proposer, [1_u8; 32]).unwrap();

        assert_eq!(
            round.add_prevote(Vote { validator: vid("z"), height: 2, block_hash: [1_u8; 32] }),
            Err(RoundError::UnknownValidator)
        );
        assert_eq!(
            round.add_prevote(Vote { validator: vid("a"), height: 99, block_hash: [1_u8; 32] }),
            Err(RoundError::WrongHeight)
        );
    }

    #[test]
    fn weighted_power_reaches_quorum() {
        // a has 3/5 of the power; a + b = 4 >= quorum(5)=4.
        let s = ValidatorSet::new(vec![
            Validator { id: vid("a"), voting_power: 3 },
            Validator { id: vid("b"), voting_power: 1 },
            Validator { id: vid("c"), voting_power: 1 },
        ])
        .unwrap();
        assert_eq!(s.quorum_power(), 4);
        let event = finalize_height(&s, 1, [8_u8; 32], &ids(&["a", "b"]), 8).unwrap();
        assert_eq!(event.certificate.signed_power, 4);
    }
}
