//! BFT consensus engine for ABYSS — Stage 1 (ADR-0016).
//!
//! Implements Tendermint-style consensus: Propose → PreVote → PreCommit → Commit.
//!
//! ## Module map
//! - `validator` — ValidatorId, Validator, ValidatorSet, leader rotation, quorum certification
//! - `vote`      — VoteType, Vote, VoteSet, QuorumCertificate
//! - `round`     — Phase, RoundState (the per-round BFT phase machine)
//! - `view_change` — TimeoutVote, ViewChangeCollector (leader-failure handling)
//! - `slashing`  — SlashingEvidence, SlashingRegistry (evidence API — see ADR-0021
//!                 for why this is explicitly NOT the full slashing economics layer)
//! - `engine`    — ConsensusEngine, the per-node driver tying the above together
//! - `error`     — ConsensusError, shared across all modules
//!
//! See ADR-0017 (Consensus ↔ Execution Interface) for how this crate's
//! output (a PreCommit QuorumCertificate) is intended to connect to
//! `abyss_core::Chain::apply_block()` — that wiring is tracked as
//! follow-up work, not yet implemented.

mod engine;
mod error;
mod round;
mod slashing;
mod validator;
mod view_change;
mod vote;

#[cfg(test)]
pub(crate) mod test_support;

pub use engine::ConsensusEngine;
pub use error::ConsensusError;
pub use round::{Phase, RoundState};
pub use slashing::{SlashingEvidence, SlashingRegistry};
pub use validator::{Validator, ValidatorId, ValidatorSet};
pub use view_change::{TimeoutVote, ViewChangeCollector, ViewChangeDecision};
pub use vote::{QuorumCertificate, Vote, VoteSet, VoteType};
