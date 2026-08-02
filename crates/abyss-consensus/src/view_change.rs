//! View Change — leader failure handling.
//!
//! If a round's leader does not produce a valid proposal in time,
//! validators broadcast `TimeoutVote`s. Once enough voting power has
//! timed out, the round is abandoned and consensus moves to the next
//! round (and therefore the next leader, per validator.rs's rotation).

use std::collections::BTreeMap;

use crate::error::ConsensusError;
use crate::validator::{ValidatorId, ValidatorSet};

/// A signed timeout message — sent when a validator's round timer expires
/// without seeing a valid proposal or reaching quorum.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeoutVote {
    pub validator: ValidatorId,
    pub height: u64,
    pub round: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewChangeDecision {
    Wait,
    ChangeView,
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
        // (at 1/3 honest validators timing out, the round cannot finish).
        let trigger_threshold = validator_set.total_power() / 3 + 1;
        if timed_out_power >= trigger_threshold {
            Ok(ViewChangeDecision::ChangeView)
        } else {
            Ok(ViewChangeDecision::Wait)
        }
    }

    pub fn timeout_count(&self) -> usize { self.timeouts.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;

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
        assert_eq!(t2, ViewChangeDecision::ChangeView); // 20/40 > 13.3
    }

    // ── Byzantine test ────────────────────────────────────────────────────

    #[test]
    fn byzantine_minority_below_one_third_cannot_force_view_change() {
        // A single Byzantine validator (10/40 power) claiming timeout
        // must NOT be enough to force the honest network into a view change.
        let set = ValidatorSet::new(vec![
            validator("a", 10), validator("b", 10),
            validator("c", 10), validator("d", 10),
        ]).unwrap();
        let mut collector = ViewChangeCollector::new();

        let decision = collector.add_timeout(
            TimeoutVote { validator: vid("a"), height: 1, round: 0 }, &set
        ).unwrap();
        assert_eq!(decision, ViewChangeDecision::Wait);
    }
}
